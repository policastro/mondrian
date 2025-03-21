use super::configs::TilesManagerConfig;
use super::error::TilesManagerError;
use super::operations::TilesManagerInternalOperations;
use super::public::FocusHistory;
use crate::app::area_tree::tree::WinTree;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::area::Area;
use crate::modules::tiles_manager::lib::containers::container::Container;
use crate::modules::tiles_manager::lib::containers::inactive::InactiveContainers;
use crate::modules::tiles_manager::lib::containers::keys::ContainerKey;
use crate::modules::tiles_manager::lib::containers::keys::CrossLayerContainerKey;
use crate::modules::tiles_manager::lib::containers::layer::ContainerLayer;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::utils::get_current_time_ms;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimationPlayer;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::api::window::enum_user_manageable_windows;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use winvd::get_current_desktop;
use winvd::Desktop;

type Error = TilesManagerError;

pub struct TilesManager {
    pub active_trees: HashMap<CrossLayerContainerKey, WinTree>,
    pub inactive_trees: HashMap<ContainerKey, (WinTree, u128)>,
    pub peeked_containers: HashMap<CrossLayerContainerKey, Area>,
    pub floating_wins: HashSet<WindowRef>,
    pub maximized_wins: HashSet<WindowRef>,
    pub pause_updates: bool,
    pub config: TilesManagerConfig,
    pub animation_player: WindowAnimationPlayer,
    pub focus_history: FocusHistory,
    pub(crate) current_vd: Option<Desktop>,
}

pub trait TilesManagerBase {
    /// Creates a new [`TilesManager`].
    fn new<S, E, C>(
        config: Option<TilesManagerConfig>,
        on_update_start: S,
        on_update_error: E,
        on_update_complete: C,
    ) -> Self
    where
        S: Fn(HashSet<WindowRef>) + Sync + Send + 'static,
        E: Fn() + Sync + Send + 'static,
        C: Fn() + Sync + Send + 'static;

    /// Initialize the tiles manager. It must be called before any other operation.
    fn init(&mut self) -> Result<(), Error>;

    /// Updates the tiles layout of all active containers
    fn update_layout(&mut self, animate: bool) -> Result<(), Error>;

    /// Get the state of a managed window
    /// Returns `None` if the window is not managed (i.e. not in any active container)
    fn get_managed_windows(&self) -> HashMap<WindowRef, WindowTileState>;

    /// Add all open windows to the tiles manager
    fn add_open_windows(&mut self) -> Result<(), Error>;

    /// Cancel the ongoing animation
    fn cancel_animation(&mut self);

    /// Pause the updates of the tiles manager (i.e. prevents `update_layout` from executing)
    fn pause_updates(&mut self, pause: bool);
}

impl TilesManagerBase for TilesManager {
    fn new<S, E, C>(
        config: Option<TilesManagerConfig>,
        on_update_start: S,
        on_update_error: E,
        on_update_complete: C,
    ) -> Self
    where
        S: Fn(HashSet<WindowRef>) + Sync + Send + 'static,
        E: Fn() + Sync + Send + 'static,
        C: Fn() + Sync + Send + 'static,
    {
        let config = config.unwrap_or_default();
        let animation_duration = Duration::from_millis(config.get_animation_duration().into());
        let animation_player = WindowAnimationPlayer::new(
            animation_duration,
            config.get_framerate(),
            on_update_start,
            on_update_error,
            on_update_complete,
        );

        TilesManager {
            pause_updates: false,
            floating_wins: HashSet::new(),
            maximized_wins: HashSet::new(),
            inactive_trees: HashMap::new(),
            active_trees: HashMap::new(),
            peeked_containers: HashMap::new(),
            current_vd: None,
            focus_history: FocusHistory::new(),
            config,
            animation_player,
        }
    }

    fn add_open_windows(&mut self) -> Result<(), Error> {
        let filter = self.config.filter.clone();
        let mut wins: Vec<WindowRef> = enum_user_manageable_windows()
            .into_iter()
            .filter(|w| !filter.matches(*w))
            .collect();

        // INFO: bigger windows first
        wins.sort_by(|a, b| {
            let area_1 = a.get_area().unwrap_or_default().get_area();
            let area_2 = b.get_area().unwrap_or_default().get_area();
            area_2.cmp(&area_1)
        });

        for w in wins.iter() {
            self.add(*w, None).ok();
        }
        Ok(())
    }

    fn get_managed_windows(&self) -> HashMap<WindowRef, WindowTileState> {
        let mut tiled: HashMap<WindowRef, WindowTileState> = self
            .active_trees
            .values()
            .flat_map(|c| c.get_ids())
            .filter_map(|win| self.get_window_state(win).map(|state| (win, state)).ok())
            .collect();

        tiled.extend(self.floating_wins.iter().map(|w| (*w, WindowTileState::Floating)));

        tiled
    }

    fn cancel_animation(&mut self) {
        self.animation_player.cancel();
    }

    fn init(&mut self) -> Result<(), Error> {
        let current_vd = get_current_desktop().map_err(Error::VDError)?;
        self.inactive_trees.clear();
        self.create_inactive_vd_containers(current_vd)?;
        self.activate_vd_containers(current_vd, Some(ContainerLayer::Normal))
    }

    fn update_layout(&mut self, animate: bool) -> Result<(), Error> {
        if self.pause_updates {
            return Ok(());
        }

        let anim_player = &mut self.animation_player;
        self.active_trees.iter_mut().for_each(|(k, c)| {
            let (border_pad, tile_pad) = match k.layer {
                ContainerLayer::Focalized => (self.config.get_focalized_padding(&k.monitor), (0, 0)),
                ContainerLayer::Normal => (
                    self.config.get_borders_padding(&k.monitor),
                    self.config.get_tiles_padding_xy(&k.monitor),
                ),
            };

            // INFO: prevent updates when the monitor has a maximized window
            if self.maximized_wins.iter().any(|w| c.has(*w)) {
                return;
            }

            let _ = c.update(border_pad, tile_pad, anim_player, &self.maximized_wins);
        });
        let animation = self.config.get_animations().filter(|_| animate);
        anim_player.play(animation);
        Ok(())
    }

    fn pause_updates(&mut self, pause: bool) {
        self.pause_updates = pause;
        if pause {
            self.cancel_animation();
        };
    }
}

impl TilesManager {
    pub fn get_window_state(&self, window: WindowRef) -> Result<WindowTileState, Error> {
        if self.floating_wins.contains(&window) {
            return Ok(WindowTileState::Floating);
        }

        if self.maximized_wins.contains(&window) {
            return Ok(WindowTileState::Maximized);
        }

        let key = self.active_trees.find(window).map(|e| e.key)?;
        match key.layer {
            ContainerLayer::Focalized => Ok(WindowTileState::Focalized),
            ContainerLayer::Normal => Ok(WindowTileState::Normal),
        }
    }

    pub fn create_inactive_vd_containers(&mut self, vd: Desktop) -> Result<(), Error> {
        let vd_id = vd.get_id().map_err(Error::VDError)?.to_u128();
        if self.current_vd.is_some_and(|curr_vd| curr_vd == vd) || self.inactive_trees.has_vd(vd_id) {
            return Err(Error::VDContainersAlreadyCreated);
        }

        let curr_time = get_current_time_ms()?;
        enum_display_monitors()
            .iter()
            .flat_map(|m| {
                let layout = self.config.get_layout_strategy(m.id.as_str());
                let t1 = WinTree::new((*m).clone().into(), layout.clone());
                let t2 = WinTree::new((*m).clone().into(), layout);
                [
                    (ContainerKey::normal(vd_id, m.id.clone()), t1, curr_time),
                    (ContainerKey::focalized(vd_id, m.id.clone()), t2, 0),
                ]
            })
            .for_each(|(k, v, ts)| {
                self.inactive_trees.insert(k, (v, ts));
            });

        Ok(())
    }

    pub fn activate_vd_containers(&mut self, vd: Desktop, layer: Option<ContainerLayer>) -> Result<(), Error> {
        let vd_id = vd.get_id().map(|id| id.to_u128()).map_err(Error::VDError)?;

        if self.current_vd.is_some_and(|current_vd| current_vd == vd) {
            return Err(Error::VDContainersAlreadyActivated);
        }

        let current_time = get_current_time_ms()?;
        let to_inactivate = self.active_trees.drain().map(|(k, v)| (k.into(), (v, current_time)));
        self.inactive_trees.extend(to_inactivate);

        let mut latest_keys: HashMap<String, (ContainerKey, u128)> = HashMap::new();
        self.inactive_trees
            .iter()
            .filter(|(k, _)| k.is_vd(vd_id) && layer.as_ref().is_none_or(|layer| k.is_layer(*layer)))
            .for_each(|(k, (_, ts))| {
                let latest = latest_keys.entry(k.monitor.clone()).or_insert_with(|| (k.clone(), *ts));
                if *ts > latest.1 {
                    latest_keys.insert(k.monitor.clone(), (k.clone(), *ts));
                }
            });

        for (k, _) in latest_keys.values() {
            if let Some(c) = self.inactive_trees.remove(k) {
                self.active_trees.insert(k.clone().into(), c.0);
            }
        }

        self.current_vd = Some(vd);
        Ok(())
    }

    pub fn activate_monitor_layer(&mut self, monitor_name: String, layer: ContainerLayer) -> Result<(), Error> {
        let vd_id = self.current_vd.and_then(|vd| vd.get_id().ok()).map(|id| id.to_u128());
        let vd_id = vd_id.ok_or(Error::Generic)?;
        let active_key = ContainerKey::new(vd_id, monitor_name.clone(), layer);

        let inactive_key = self
            .active_trees
            .get_key_value(&active_key.clone().into())
            .map(|v| v.0)
            .cloned();
        if inactive_key.as_ref().is_none_or(|k| k.layer == layer) {
            return Ok(());
        }

        let (active_container, _) = self.inactive_trees.remove(&active_key).ok_or(Error::Generic)?;

        if let Some(inactive_key) = inactive_key {
            let inactive_container = self.active_trees.remove(&inactive_key).ok_or(Error::Generic)?;
            let current_time = get_current_time_ms()?;
            self.inactive_trees
                .insert(inactive_key.into(), (inactive_container, current_time));
        }

        self.active_trees.insert(active_key.into(), active_container);

        Ok(())
    }

    pub fn restore_monitor(&mut self, key: &CrossLayerContainerKey) -> Result<(), Error> {
        if matches!(key.layer, ContainerLayer::Focalized) {
            self.active_trees
                .get_mut(key)
                .ok_or(Error::ContainerNotFound { refresh: false })?
                .clear();
            return self.activate_monitor_layer(key.monitor.clone(), ContainerLayer::Normal);
        };

        Ok(())
    }
}
