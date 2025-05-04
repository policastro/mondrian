pub mod command;
pub mod configs;
pub mod floating;
pub mod operations;
pub mod public;
pub mod result;

use super::containers::container::Container;
use super::containers::container::ContainerLayer;
use super::containers::map::ContainersMap;
use super::structs::focus_history::FocusHistory;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::area_tree::tree::WinTree;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::area::Area;
use crate::modules::tiles_manager::lib::containers::keys::ActiveContainerKey;
use crate::modules::tiles_manager::lib::containers::keys::ContainerKey;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimationPlayer;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::api::monitor::Monitor;
use crate::win32::api::window::enum_user_manageable_windows;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use configs::TilesManagerConfig;
use floating::FloatingProperties;
use floating::FloatingWindows;
use operations::TilesManagerOperations;
use result::TilesManagerError;
use result::TilesManagerSuccess;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;
use winvd::get_current_desktop;
use winvd::Desktop;

type Error = TilesManagerError;

pub struct TilesManager {
    pub active_trees: HashMap<ActiveContainerKey, Container>,
    pub inactive_trees: HashMap<ContainerKey, Container>,
    pub peeked_containers: HashMap<ActiveContainerKey, Area>,
    pub floating_wins: HashMap<WindowRef, FloatingProperties>,
    pub maximized_wins: HashSet<WindowRef>,
    pub pause_updates: bool,
    pub config: TilesManagerConfig,
    pub animation_player: WindowAnimationPlayer,
    pub focus_history: FocusHistory,
    pub managed_monitors: HashMap<String, Monitor>,
    pub(crate) current_vd: Option<Desktop>,
}

impl TilesManager {
    /// Creates a new [`TilesManager`].
    pub fn create<S, E, C>(
        config: Option<TilesManagerConfig>,
        on_update_start: S,
        on_update_error: E,
        on_update_complete: C,
    ) -> Result<TilesManager, Error>
    where
        S: Fn(HashSet<WindowRef>) + Sync + Send + 'static,
        E: Fn() + Sync + Send + 'static,
        C: Fn() + Sync + Send + 'static,
    {
        let config = config.unwrap_or_default();
        let animation_player = WindowAnimationPlayer::new(
            Duration::from_millis(config.animation.duration.into()),
            config.animation.framerate,
            on_update_start,
            on_update_error,
            on_update_complete,
        );

        let mut tm = TilesManager {
            pause_updates: false,
            floating_wins: HashMap::new(),
            maximized_wins: HashSet::new(),
            inactive_trees: HashMap::new(),
            active_trees: HashMap::new(),
            peeked_containers: HashMap::new(),
            current_vd: None,
            focus_history: FocusHistory::new(),
            managed_monitors: HashMap::new(),
            config,
            animation_player,
        };

        let current_vd = get_current_desktop().map_err(Error::VDError)?;

        tm.managed_monitors = enum_display_monitors()
            .iter()
            .map(|m| (m.id.clone(), m.clone()))
            .collect();

        tm.create_inactive_vd_containers(current_vd)?;
        tm.activate_vd_containers(current_vd)?;

        Ok(tm)
    }

    /// Add all open windows to the tiles manager
    pub fn add_open_windows(&mut self) -> Result<(), Error> {
        let filter = self.config.ignore_filter.clone();
        let wins = enum_user_manageable_windows().into_iter();
        let mut wins: Vec<WindowRef> = wins.filter(|w| !filter.matches(*w)).collect();

        // INFO: bigger windows first
        wins.sort_by(|a, b| {
            let area_1 = a.get_area().unwrap_or_default().calc_area();
            let area_2 = b.get_area().unwrap_or_default().calc_area();
            area_2.cmp(&area_1)
        });

        for w in wins.iter() {
            self.add(*w, None, true).inspect(|s| {
                if let TilesManagerSuccess::Queue { window, area, topmost } = s {
                    self.animation_player.queue(*window, *area, *topmost);
                }
            })?;
        }
        Ok(())
    }

    /// Get the state of a managed window
    /// Returns `None` if the window is not managed (i.e. not in any active container)
    pub fn get_visible_managed_windows(&self) -> HashMap<WindowRef, WindowTileState> {
        let wins = self.active_trees.values().flat_map(|c| c.tree().get_ids());
        let mut tiled: HashMap<WindowRef, WindowTileState> = wins
            .filter_map(|win| self.get_window_state(win).map(|state| (win, state)).ok())
            .collect();

        let floating = self
            .floating_wins
            .enabled_keys()
            .map(|w| (w, WindowTileState::Floating));

        tiled.extend(floating);

        tiled
    }

    /// Cancel the ongoing animation
    pub fn cancel_animation(&mut self) {
        self.animation_player.cancel();
    }

    /// Updates the tiles layout of all active containers.
    /// If `win_in_focus` is `Some`, focus will be moved to that window.
    pub fn update_layout(&mut self, animate: bool, win_in_focus: Option<WindowRef>) -> Result<(), Error> {
        if self.pause_updates {
            return Ok(());
        }

        let anim_player = &mut self.animation_player;
        self.active_trees.iter_mut().for_each(|(k, c)| {
            let tile_pad = match c.current() {
                ContainerLayer::Focalized => (0, 0),
                ContainerLayer::HalfFocalized => self.config.get_half_focalized_tiles_pad_xy(&k.monitor),
                ContainerLayer::Normal => self.config.get_tiles_padding_xy(&k.monitor),
            };
            let _ = update_from_tree(
                c.tree_mut(),
                (-tile_pad.0, -tile_pad.1),
                tile_pad,
                anim_player,
                &self.maximized_wins,
            );
        });

        // INFO: set maximized windows to the front when animation is complete
        let maximized = self.maximized_wins.clone();
        anim_player.play(
            self.config.animation.animation_type.filter(|_| animate),
            win_in_focus,
            Some(Arc::new(move || {
                maximized.iter().for_each(|w| w.to_front());
            })),
        );
        Ok(())
    }

    /// Pause the updates of the tiles manager (i.e. prevents `update_layout` from executing)
    pub fn pause_updates(&mut self, pause: bool) {
        self.pause_updates = pause;
        if pause {
            self.cancel_animation();
        };
    }

    /// Returns the state of a managed window, only if it's in one of the active containers.
    /// If a floating window is present (even if minimized), returns [`WindowTileState::Floating`].
    pub fn get_window_state(&self, window: WindowRef) -> Result<WindowTileState, Error> {
        if self.floating_wins.contains_key(&window) {
            return Ok(WindowTileState::Floating);
        }

        if self.maximized_wins.contains(&window) {
            return Ok(WindowTileState::Maximized);
        }

        let container_type = self.active_trees.find(window).map(|e| e.value.current())?;
        match container_type {
            ContainerLayer::Focalized => Ok(WindowTileState::Focalized),
            ContainerLayer::HalfFocalized => Ok(WindowTileState::HalfFocalized),
            ContainerLayer::Normal => Ok(WindowTileState::Normal),
        }
    }

    pub(super) fn create_inactive_vd_containers(&mut self, vd: Desktop) -> Result<(), Error> {
        let vd_id = vd.get_id().map_err(Error::VDError)?.to_u128();
        if self.current_vd.is_some_and(|curr_vd| curr_vd == vd) || self.inactive_trees.has_vd(vd_id) {
            return Err(Error::VDContainersAlreadyCreated);
        }

        self.managed_monitors
            .values()
            .map(|m| {
                let layout = self.config.get_layout_strategy(m.id.as_str());
                let bpad1 = self.config.get_borders_padding(m.id.as_str());
                let bpad2 = self.config.get_focalized_padding(m.id.as_str());
                let bpad3 = self.config.get_half_focalized_borders_pad(m.id.as_str());

                let area = m.get_workspace();
                let t1 = WinTree::new(area, layout.clone(), bpad1);
                let t2 = WinTree::new(area, layout.clone(), bpad2);
                let t3 = WinTree::new(area, layout, bpad3);
                let c = Container::new(t1, t2, t3);

                (ContainerKey::new(vd_id, m.id.clone()), c)
            })
            .for_each(|(k, c)| {
                self.inactive_trees.insert(k, c);
            });

        Ok(())
    }

    pub(super) fn activate_vd_containers(&mut self, vd: Desktop) -> Result<(), Error> {
        let vd_id = vd.get_id().map(|id| id.to_u128()).map_err(Error::VDError)?;

        if self.current_vd.is_some_and(|current_vd| current_vd == vd) {
            return Err(Error::VDContainersAlreadyActivated);
        }

        let to_inactivate = self.active_trees.drain().map(|(k, v)| (k.into(), v));
        self.inactive_trees.extend(to_inactivate);

        let to_activate: Vec<ContainerKey> = self
            .inactive_trees
            .keys()
            .filter(|&k| k.is_vd(vd_id))
            .cloned()
            .collect();

        for k in to_activate {
            if let Some(c) = self.inactive_trees.remove(&k) {
                self.active_trees.insert(k.clone().into(), c);
            }
        }

        self.current_vd = Some(vd);
        Ok(())
    }

    pub(super) fn restore_monitor(&mut self, key: &ActiveContainerKey) -> Result<(), Error> {
        let is_focalized_or_half = self.active_trees.get(key).map(|c| c.current());
        let is_focalized_or_half = is_focalized_or_half.is_some_and(|ct| ct.is_focalized_or_half());

        if is_focalized_or_half {
            let container = self
                .active_trees
                .get_mut(key)
                .ok_or(Error::ContainerNotFound { refresh: false })?;

            container.tree_mut().clear();
            container.set_current(ContainerLayer::Normal);
            return Ok(());
        };
        Ok(())
    }

    pub(super) fn get_maximized_win_in_monitor(&self, key: &ActiveContainerKey) -> Option<WindowRef> {
        let t = self.active_trees.get(key)?;
        self.maximized_wins.iter().find(|w| t.tree().has(**w)).copied()
    }

    pub(super) fn restore_maximized(&mut self, key: &ActiveContainerKey) -> Result<(), Error> {
        if let Some(win) = self.get_maximized_win_in_monitor(key) {
            win.set_normal();
            self.as_maximized(win, false)?;
        }
        Ok(())
    }
}

fn update_from_tree(
    tree: &mut WinTree,
    border_pad: (i16, i16),
    tile_pad: (i16, i16),
    animation_player: &mut WindowAnimationPlayer,
    ignored_wins: &HashSet<WindowRef>,
) -> Result<(), Error> {
    let leaves: Vec<AreaLeaf<WindowRef>> = tree.padded_leaves(border_pad, Some(ignored_wins));

    for leaf in &leaves {
        if !leaf.id.is_visible() {
            tree.remove(leaf.id);
            return update_from_tree(tree, border_pad, tile_pad, animation_player, ignored_wins);
        };
        let area = leaf.viewbox.pad_xy(tile_pad);
        leaf.id.restore(false);
        let borders = leaf.id.get_borders().unwrap_or((0, 0, 0, 0));
        let borders = (
            borders.0.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
            borders.1.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
            borders.2.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
            borders.3.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
        );
        let area = area.shift((-borders.0, -borders.1, borders.2 + borders.0, borders.3 + borders.1));
        animation_player.queue(leaf.id, area, None);
    }
    Ok(())
}
