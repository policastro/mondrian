use super::configs::TilesManagerConfig;
use super::error::TilesManagerError;
use super::operations::FocalizedMap;
use super::operations::TilesManagerInternalOperations;
use super::public::FocusHistory;
use crate::app::area_tree::tree::WinTree;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::area::Area;
use crate::modules::tiles_manager::lib::containers::Container;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimationPlayer;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::api::window::enum_user_manageable_windows;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::Duration;
use winvd::get_current_desktop;
use winvd::Desktop;

type Error = TilesManagerError;

pub struct TilesManager {
    pub inactive_trees: HashMap<ContainerKey, WinTree>,
    pub active_trees: HashMap<ContainerKey, WinTree>,
    pub peeked_containers: HashMap<ContainerKey, Area>,
    pub focalized_wins: HashMap<ContainerKey, WindowRef>,
    pub floating_wins: HashSet<WindowRef>,
    pub maximized_wins: HashSet<WindowRef>,
    pub pause_updates: bool,
    pub config: TilesManagerConfig,
    pub animation_player: WindowAnimationPlayer,
    pub focus_history: FocusHistory,
    pub(crate) current_vd: Option<Desktop>,
}

pub trait TilesManagerBase {
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
    fn add_open_windows(&mut self) -> Result<(), Error>;
    fn get_managed_windows(&self) -> HashMap<WindowRef, WindowTileState>;
    fn cancel_animation(&mut self);
    fn init(&mut self) -> Result<(), Error>;
    fn update_layout(&mut self, animate: bool) -> Result<(), Error>;
    fn pause_updates(&mut self, pause: bool);
}

impl TilesManager {
    pub fn get_window_state(&self, window: WindowRef) -> Option<WindowTileState> {
        let is_managed = self.active_trees.find(window).is_some();
        let is_floating = self.floating_wins.contains(&window);
        let is_maximized = self.maximized_wins.contains(&window);
        let is_focalized = self
            .active_trees
            .find(window)
            .is_some_and(|e| self.focalized_wins.matches(&e.key, window));

        if is_managed && !is_floating && !is_maximized && !is_focalized {
            Some(WindowTileState::Normal)
        } else if is_floating {
            Some(WindowTileState::Floating)
        } else if is_focalized {
            Some(WindowTileState::Focalized)
        } else if is_maximized {
            Some(WindowTileState::Maximized)
        } else {
            None
        }
    }

    pub fn build_vd_containers(&mut self, vd: Desktop) -> Result<HashMap<ContainerKey, WinTree>, Error> {
        let vd_id = vd.get_id().map_err(Error::VirtualDesktopError)?.to_u128();
        let containers = enum_display_monitors()
            .iter()
            .map(|m| {
                let layout = self.config.get_layout_strategy(m.id.as_str());
                let tree = WinTree::new((*m).clone().into(), layout);
                (ContainerKey::new(vd_id, m.id.clone(), String::new()), tree)
            })
            .collect();

        Ok(containers)
    }

    pub fn activate_containers(
        &mut self,
        vd: Option<Desktop>,
        monitor_name: Option<String>,
        layer: Option<String>,
    ) -> Result<(), Error> {
        let vd_id = vd.and_then(|vd| vd.get_id().ok()).map(|id| id.to_u128());

        self.inactive_trees.extend(self.active_trees.drain());

        let active_keys: Vec<_> = self
            .inactive_trees
            .keys()
            .filter(|k| {
                let is_vd = vd_id.as_ref().is_none_or(|id| k.is_virtual_desktop(*id));
                let is_monitor = monitor_name.as_ref().is_none_or(|name| k.is_monitor(name));
                let is_layer = layer.as_ref().is_none_or(|layer| k.is_layer(layer));
                is_vd && is_monitor && is_layer
            })
            .cloned()
            .collect();

        for k in active_keys {
            if let Some(c) = self.inactive_trees.remove(&k) {
                self.active_trees.insert(k.clone(), c);
            }
        }

        Ok(())
    }
}

impl TilesManagerBase for TilesManager {
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
            focalized_wins: HashMap::new(),
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
            self.add(*w, None)?;
        }

        Ok(())
    }

    fn get_managed_windows(&self) -> HashMap<WindowRef, WindowTileState> {
        let mut tiled: HashMap<WindowRef, WindowTileState> = self
            .active_trees
            .values()
            .flat_map(|c| c.get_ids())
            .filter_map(|win| self.get_window_state(win).map(|state| (win, state)))
            .collect();

        tiled.extend(self.floating_wins.iter().map(|w| (*w, WindowTileState::Floating)));

        tiled
    }

    fn cancel_animation(&mut self) {
        self.animation_player.cancel();
    }

    fn init(&mut self) -> Result<(), Error> {
        let current_vd = get_current_desktop().map_err(Error::VirtualDesktopError)?;
        self.current_vd = Some(current_vd);
        self.active_trees = self.build_vd_containers(current_vd)?;
        self.inactive_trees.clear();
        Ok(())
    }

    fn update_layout(&mut self, animate: bool) -> Result<(), Error> {
        if self.pause_updates {
            return Ok(());
        }

        let anim_player = &mut self.animation_player;
        self.active_trees.iter_mut().for_each(|(k, c)| {
            let (border_pad, tile_pad) = match self.focalized_wins.contains_key(k) {
                true => (self.config.get_focalized_padding(&k.monitor_name), (0, 0)),
                false => (
                    self.config.get_borders_padding(&k.monitor_name),
                    self.config.get_tiles_padding_xy(&k.monitor_name),
                ),
            };

            // INFO: prevent updates when the monitor has a maximized window
            if self.maximized_wins.iter().any(|w| c.has(*w)) {
                return;
            }

            let ignored = match self.focalized_wins.get(k) {
                Some(fw) => &c
                    .get_ids()
                    .iter()
                    .filter(|w| w.hwnd != fw.hwnd)
                    .cloned()
                    .inspect(|w| {
                        w.minimize();
                    })
                    .collect(),
                None => &self.maximized_wins,
            };

            let _ = c.update(border_pad, tile_pad, anim_player, ignored);
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

impl Drop for TilesManager {
    fn drop(&mut self) {}
}

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct ContainerKey {
    pub virtual_desktop: u128,
    pub monitor_name: String,
    pub layer: String, // TODO: support for multiple layers
}

impl ContainerKey {
    pub fn new(virtual_desktop: u128, monitor_name: String, layer: String) -> Self {
        ContainerKey {
            virtual_desktop,
            monitor_name,
            layer,
        }
    }

    pub fn is_virtual_desktop(&self, vd: u128) -> bool {
        self.virtual_desktop == vd
    }

    pub fn is_monitor(&self, monitor_name: &str) -> bool {
        self.monitor_name == monitor_name
    }

    pub fn is_layer(&self, layer: &str) -> bool {
        self.layer == layer
    }
}
