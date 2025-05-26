pub mod command;
pub mod configs;
pub mod floating;
pub mod operations;
pub mod public;
pub mod result;

use super::containers::container::Container;
use super::containers::container::ContainerLayer;
use super::containers::keys::ContainerKeyTrait;
use super::containers::map::ContainersMap;
use super::structs::focus_history::FocusHistory;
use super::structs::managed_monitor::ManagedMonitor;
use super::structs::virtual_desktop::VirtualDesktop;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::area_tree::tree::WinTree;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::area::Area;
use crate::modules::tiles_manager::lib::containers::keys::ActiveContainerKey;
use crate::modules::tiles_manager::lib::containers::keys::ContainerKey;
use crate::modules::tiles_manager::lib::containers::map::ActiveContainersMap;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimationPlayer;
use crate::win32::api::monitor::enum_display_monitors;
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
    containers: HashMap<ActiveContainerKey, Container>,
    inactive_containers: HashMap<ContainerKey, Container>,
    floating_wins: HashMap<WindowRef, FloatingProperties>,
    maximized_wins: HashSet<WindowRef>,
    peeked_containers: HashMap<ContainerKey, Area>,
    pause_updates: bool,
    animation_player: WindowAnimationPlayer,
    focus_history: FocusHistory,
    managed_monitors: HashMap<String, ManagedMonitor>,
    last_focused_monitor: Option<String>,
    last_workspaces: HashMap<(u128, String), String>,
    config: TilesManagerConfig,
    current_vd: VirtualDesktop,
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

        let current_vd = get_current_desktop().map_err(Error::VDError)?;
        let mut tm = TilesManager {
            pause_updates: false,
            floating_wins: HashMap::new(),
            maximized_wins: HashSet::new(),
            inactive_containers: HashMap::new(),
            containers: HashMap::new(),
            peeked_containers: HashMap::new(),
            focus_history: FocusHistory::new(),
            managed_monitors: HashMap::new(),
            config,
            animation_player,
            last_focused_monitor: None,
            last_workspaces: HashMap::new(),
            current_vd: current_vd.try_into()?,
        };

        tm.managed_monitors = enum_display_monitors()
            .iter()
            .map(|m| (m.id.clone(), m.clone().into()))
            .collect();

        tm.activate_vd(current_vd)?;

        Ok(tm)
    }

    /// Add all open windows to the tiles manager
    pub fn add_open_windows(&mut self) -> Result<(), Error> {
        let filter = self.config.ignore_filter.clone();

        // INFO: filter out windows that are already managed
        let wins = enum_user_manageable_windows().into_iter().filter(|w| {
            !self
                .containers
                .iter()
                .any(|(_, c)| c.get_tree(ContainerLayer::Normal).has(*w))
        });
        let mut wins: Vec<WindowRef> = wins.filter(|w| !filter.matches(*w)).collect();

        // INFO: bigger windows first
        wins.sort_by(|a, b| {
            let area_1 = a.get_area().unwrap_or_default().calc_area();
            let area_2 = b.get_area().unwrap_or_default().calc_area();
            area_2.cmp(&area_1)
        });

        for w in wins.iter() {
            self.add(*w, None, true, true).inspect(|s| {
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
        let wins = self.containers.values().flat_map(|c| c.tree().get_ids());
        let mut tiled: HashMap<WindowRef, WindowTileState> = wins
            .filter_map(|win| self.get_window_state(win).map(|state| (win, state)).ok())
            .collect();

        let floating = self
            .floating_wins
            .enabled_keys(&self.current_vd)
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
        self.containers.iter_mut().for_each(|(k, c)| {
            let tile_pad = match c.current() {
                ContainerLayer::Focalized => (0, 0),
                ContainerLayer::HalfFocalized => self.config.get_half_focalized_tiles_pad_xy(&k.monitor, &k.workspace),
                ContainerLayer::Normal => self.config.get_tiles_padding_xy(&k.monitor, &k.workspace),
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

        let container_type = self.containers.find(window).map(|e| e.value.current())?;
        match container_type {
            ContainerLayer::Focalized => Ok(WindowTileState::Focalized),
            ContainerLayer::HalfFocalized => Ok(WindowTileState::HalfFocalized),
            ContainerLayer::Normal => Ok(WindowTileState::Normal),
        }
    }

    fn activate_vd(&mut self, vd: Desktop) -> Result<TilesManagerSuccess, Error> {
        let vd_id = vd.get_id().map(|id| id.to_u128()).map_err(Error::VDError)?;
        if self.containers.has_vd(vd_id) {
            return Ok(TilesManagerSuccess::NoChange);
        }

        let to_inactivate = self.containers.drain().map(|(k, v)| {
            let (vd, monitor, ws) = (k.vd, k.monitor.clone(), k.workspace.clone());
            self.last_workspaces.insert((vd, monitor), ws);
            (k.into(), v)
        });

        self.inactive_containers.extend(to_inactivate);

        if !self.inactive_containers.has_vd(vd_id) {
            self.create_inactive_vd_containers(vd)?;
        }

        let to_activate: Vec<ContainerKey> = self
            .inactive_containers
            .keys()
            .filter(|&k| {
                let last_ws = self.last_workspaces.get(&(vd_id, k.monitor.clone()));
                k.is_vd(vd_id) && last_ws.is_none_or(|ws| *ws == k.workspace)
            })
            .cloned()
            .collect();

        for k in to_activate {
            if let Some(c) = self.inactive_containers.remove(&k) {
                self.containers.insert(k.clone().into(), c);
            }
        }

        self.current_vd = vd.try_into()?;
        Ok(TilesManagerSuccess::LayoutChanged)
    }

    fn activate_workspace(
        &mut self,
        monitor_name: &str,
        workspace: &str,
        silent: bool,
        move_focus: bool,
    ) -> Result<TilesManagerSuccess, Error> {
        let prev_k = self.containers.keys().find(|k| k.monitor == monitor_name);
        let prev_k = prev_k.ok_or(Error::MonitorNotFound(monitor_name.to_string()))?.clone();

        if prev_k.workspace == workspace {
            return Ok(TilesManagerSuccess::NoChange);
        }

        let vd_id = self.current_vd.get_id();
        if !self.inactive_containers.has(vd_id, monitor_name, workspace) {
            self.create_inactive_workspace_container(self.current_vd, monitor_name, workspace)?;
        }

        let new_key = self
            .inactive_containers
            .get_key_with_workspace(vd_id, monitor_name, workspace)
            .ok_or(Error::Generic)?;
        let new_tree = self.inactive_containers.remove(&new_key).ok_or(Error::Generic)?;
        let wins_leaves = new_tree.tree().leaves(None);
        let old_tree = self.containers.replace(new_key.into(), new_tree);
        let old_tree = old_tree.ok_or(Error::container_not_found())?;

        // INFO: most recently focused window otherwise top left window
        let win_to_focus = self
            .focus_history
            .latest(&wins_leaves)
            .or(wins_leaves
                .iter()
                .min_by(|l1, l2| l1.viewbox.x.cmp(&l2.viewbox.x).then(l1.viewbox.y.cmp(&l2.viewbox.y)))
                .iter()
                .next()
                .copied())
            .filter(|_| move_focus);
        let win_to_focus = win_to_focus.map(|l| l.id);

        if !silent {
            if let Some(m) = &self.managed_monitors.get(monitor_name) {
                m.focus();
                if win_to_focus.is_none() {
                    self.last_focused_monitor = Some(monitor_name.to_string());
                }
            }
            old_tree.tree().leaves(None).iter().for_each(|l| {
                l.id.minimize(false);
            });
        }

        self.inactive_containers.insert(prev_k.into(), old_tree);
        Ok(TilesManagerSuccess::UpdateAndFocus { window: win_to_focus })
    }

    fn create_inactive_vd_containers(&mut self, vd: Desktop) -> Result<(), Error> {
        let vd_id = vd.get_id().map_err(Error::VDError)?.to_u128();
        if self.inactive_containers.has_vd(vd_id) {
            return Err(Error::VDContainersAlreadyCreated);
        }

        let containers: Vec<(ContainerKey, Container)> = self
            .managed_monitors
            .values()
            .map(|m| {
                let ws = self.config.get_default_workspace(&m.info.id);
                let key = ContainerKey::new(vd_id, &m.info.id, &ws);
                (key, m)
            })
            .filter_map(|(k, m)| self.create_container(&m.info.id, &k.workspace).ok().map(|c| (k, c)))
            .collect();

        for (k, t) in containers {
            self.inactive_containers.insert(k, t);
        }

        Ok(())
    }

    fn create_inactive_workspace_container(
        &mut self,
        vd: VirtualDesktop,
        monitor_name: &str,
        ws: &str,
    ) -> Result<(), Error> {
        let vd_id = vd.get_id();
        if self.inactive_containers.has(vd_id, monitor_name, ws) {
            return Err(Error::WorkspaceAlreadyCreated);
        }

        let container = self.create_container(monitor_name, ws)?;
        let key = ContainerKey::new(vd_id, monitor_name, ws);
        self.inactive_containers.insert(key, container);

        Ok(())
    }

    fn create_container(&self, monitor_name: &str, workspace: &str) -> Result<Container, Error> {
        let monitor = &self
            .managed_monitors
            .get(monitor_name)
            .ok_or(Error::MonitorNotFound(monitor_name.to_string()))?
            .info;
        let monitor_id = monitor.id.clone();

        let layout = self.config.get_layout_strategy(&monitor_id, workspace);
        let bpad1 = self.config.get_borders_padding(&monitor_id, workspace);
        let bpad2 = self.config.get_focalized_padding(&monitor_id, workspace);
        let bpad3 = self.config.get_half_focalized_borders_pad(&monitor_id, workspace);

        let area = monitor.get_workspace();
        let t1 = WinTree::new(area, layout.clone(), bpad1);
        let t2 = WinTree::new(area, layout.clone(), bpad2);
        let t3 = WinTree::new(area, layout, bpad3);

        Ok(Container::new(t1, t2, t3))
    }

    fn restore_monitor(&mut self, key: &ActiveContainerKey) -> Result<(), Error> {
        let is_focalized_or_half = self.containers.get(key).map(|c| c.current());
        let is_focalized_or_half = is_focalized_or_half.is_some_and(|ct| ct.is_focalized_or_half());

        if is_focalized_or_half {
            let container = self.containers.get_mut(key).ok_or(Error::container_not_found())?;

            container.tree_mut().clear();
            container.set_current(ContainerLayer::Normal);
            return Ok(());
        };
        Ok(())
    }

    fn get_maximized_win_in_monitor(&self, key: &ActiveContainerKey) -> Option<WindowRef> {
        let t = self.containers.get(key)?;
        self.maximized_wins.iter().find(|w| t.tree().has(**w)).copied()
    }

    fn get_maximized_leaf_in_monitor(&self, key: &ActiveContainerKey) -> Option<AreaLeaf<WindowRef>> {
        if let Some(win) = self.get_maximized_win_in_monitor(key) {
            return self.containers.get(key)?.tree().find_leaf(win, 0);
        }
        None
    }

    fn restore_maximized(&mut self, key: &ActiveContainerKey) -> Result<(), Error> {
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
