use super::floating::FloatingWindows;
use super::floating::TilesManagerFloating;
use super::operations::MonitorSearchStrategy;
use super::operations::TilesManagerOperations;
use super::result::TilesManagerError;
use super::result::TilesManagerSuccess;
use super::TilesManager;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::mondrian_message::IntermonitorMoveOp;
use crate::app::mondrian_message::IntramonitorMoveOp;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::direction::Direction;
use crate::app::structs::point::Point;
use crate::modules::tiles_manager::lib::containers::container::ContainerLayer;
use crate::modules::tiles_manager::lib::containers::keys::ContainerKeyTrait;
use crate::modules::tiles_manager::lib::containers::map::ActiveContainersMap;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::containers::ContainersMut;
use crate::modules::tiles_manager::lib::utils::get_foreground;
use crate::modules::tiles_manager::lib::utils::is_on_current_vd;
use crate::modules::tiles_manager::lib::utils::leaves_limited_by_edge;
use crate::win32::api::cursor::get_cursor_pos;
use crate::win32::api::cursor::set_cursor_pos;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashSet;
use winvd::get_current_desktop;
use winvd::Desktop;

type IntraOp = IntramonitorMoveOp;
type InterOp = IntermonitorMoveOp;
type Success = TilesManagerSuccess;
type Error = TilesManagerError;
type TMOperations = dyn TilesManagerOperations;
type TMFloating = dyn TilesManagerFloating;

pub trait TilesManagerEvents {
    fn on_open(&mut self, window: WindowRef) -> Result<(), Error>;
    fn on_close(&mut self, win: WindowRef) -> Result<(), Error>;
    fn on_restore(&mut self, window: WindowRef) -> Result<(), Error>;
    fn on_minimize(&mut self, win: WindowRef) -> Result<(), Error>;
    fn on_resize(&mut self, window: WindowRef, delta: (i32, i32, i32, i32)) -> Result<(), Error>;
    fn on_move(
        &mut self,
        window: WindowRef,
        target: (i32, i32),
        intra_op: IntraOp,
        inter_op: InterOp,
    ) -> Result<(), Error>;
    fn on_maximize(&mut self, window: WindowRef, maximize: bool) -> Result<(), Error>;
    fn on_focus(&mut self, window: WindowRef) -> Result<(), Error>;
    fn on_desktop_focus(&mut self, focus_position: (i32, i32)) -> Result<(), Error>;
    fn on_vd_destroyed(&mut self, destroyed: Desktop, fallback: Desktop) -> Result<(), Error>;
    fn on_vd_changed(&mut self, previous: Desktop, current: Desktop) -> Result<(), Error>;
    fn on_workarea_changed(&mut self) -> Result<(), Error>;
}

pub trait TilesManagerCommands {
    fn move_focused(&mut self, direction: Direction, floating_increment: u16) -> Result<(), Error>;
    fn release_focused(&mut self, release: Option<bool>) -> Result<(), Error>;
    fn insert_focused(&mut self, direction: Direction) -> Result<(), Error>;
    fn resize_focused(&mut self, direction: Direction, increment: u16, floating_increment: u16) -> Result<(), Error>;
    fn minimize_focused(&mut self) -> Result<(), Error>;
    fn close_focused(&mut self) -> Result<(), Error>;
    fn topmost_focused(&mut self, topmost: Option<bool>) -> Result<(), Error>;
    fn focalize_focused(&mut self) -> Result<(), Error>;
    fn half_focalize_focused(&mut self) -> Result<(), Error>;

    /// Swaps the currently focalized/half-focalized window with the next/previous window in the same monitor.
    ///
    /// - If `half` is `Some(true)`, the operation applies only to half-focalized windows.
    /// - If `half` is `Some(false)`, the operation applies only to the focalized window.
    /// - If `half` is `None`, the operation cycles both focalized and half-focalized windows without distinction.
    fn cycle_focalized_wins(&mut self, next: bool, half: Option<bool>) -> Result<(), Error>;
    fn invert_orientation(&mut self) -> Result<(), Error>;
    fn focus_workspace(&mut self, workspace_id: &str, monitor_name: Option<&str>) -> Result<(), Error>;
    fn move_focused_to_workspace(
        &mut self,
        workspace_id: &str,
        focus_workspace: bool,
        monitor_name: Option<&str>,
    ) -> Result<(), Error>;
    fn change_focus(&mut self, direction: Direction) -> Result<(), Error>;
    fn change_focus_monitor(&mut self, direction: Direction) -> Result<(), Error>;
    fn switch_focus(&mut self) -> Result<(), Error>;
    fn amplify_focused(&mut self) -> Result<(), Error>;

    /// Limits the tiling to a portion of the screen.
    /// The action is propagated to all inactive containers with:
    /// - vd == currently active virtual desktop
    /// - monitor == monitor where the focused window is located.
    fn peek_current(&mut self, direction: Direction, ratio: f32) -> Result<(), Error>;

    fn check_for_vd_changes(&mut self) -> Result<(), Error>;
}

impl TilesManagerEvents for TilesManager {
    fn on_open(&mut self, win: WindowRef) -> Result<(), Error> {
        self.floating_wins.set_properties(&win, false, false);

        let s = TMOperations::add(self, win, get_cursor_pos().ok(), true, false)?;
        self.success_handler(s, true, Some(win))
    }

    fn on_close(&mut self, win: WindowRef) -> Result<(), Error> {
        self.floating_wins.set_minimized(&win, false);

        // INFO: When floating windows are pinned to all VD, they remain locked (because no
        // open/restore event is fired). In this case, we need to unlock them manually.
        if self.floating_wins.locked(&win).unwrap_or(false) && is_on_current_vd(&win).unwrap_or(true) {
            self.floating_wins.set_locked(&win, false);
        }

        let s = TMOperations::remove(self, win)?;
        self.success_handler(s, true, None)
    }

    fn on_restore(&mut self, win: WindowRef) -> Result<(), Error> {
        self.floating_wins.set_properties(&win, false, false);
        let s = TMOperations::add(self, win, None, false, true)?;
        self.success_handler(s, true, Some(win))
    }

    fn on_minimize(&mut self, win: WindowRef) -> Result<(), Error> {
        self.floating_wins.set_properties(&win, true, false);
        let s = TMOperations::remove(self, win)?;
        self.success_handler(s, true, None)
    }

    fn on_resize(&mut self, win: WindowRef, delta: (i32, i32, i32, i32)) -> Result<(), Error> {
        let s = TMOperations::resize(self, win, delta)?;
        self.success_handler(s, true, None)
    }

    fn on_move(
        &mut self,
        win: WindowRef,
        target: (i32, i32),
        intra_op: IntraOp,
        inter_op: InterOp,
    ) -> Result<(), Error> {
        let tile_state = self.get_window_state(win)?;
        if matches!(tile_state, WindowTileState::Floating | WindowTileState::Maximized) {
            return Ok(());
        }

        let src_win = self.containers.find_leaf(win)?.id;
        let trg_win = self.containers.find_leaf_at(target).map(|l| l.id);

        let src_k = self.containers.find(src_win)?.key;
        let trg_k = match self.containers.find_near(target) {
            Ok(e) => e.key,
            Err(_) => return self.update_layout(true, None), // INFO: update if no container at target
        };

        if src_k == trg_k {
            // If it is in the same monitor
            let c = self.containers.find_mut(src_win)?.value;
            if matches!(intra_op, IntraOp::InsertFreeMove) {
                c.tree_mut().move_to(src_win, target);
            } else if let Ok(trg_win) = trg_win {
                self.swap_windows(src_win, trg_win)?;
            }
        } else if !matches!(inter_op, InterOp::Swap) || trg_win.is_err() {
            // If it is in another monitor and insert
            self.insert_window(win, target, matches!(inter_op, InterOp::InsertFreeMove))?;
        } else {
            // If it is in another monitor and swap
            if trg_win.is_err() {
                self.insert_window(win, target, false)?;
            } else if let Ok(trg_win) = trg_win {
                self.swap_windows(src_win, trg_win)?;
            }
        };

        let switch_orient = match src_k == trg_k {
            true => matches!(intra_op, IntraOp::Invert),
            false => matches!(inter_op, InterOp::Invert),
        };

        if switch_orient {
            let e = self.containers.find_near_mut(target)?;
            e.value.tree_mut().switch_subtree_orientations(target);
        }

        self.update_layout(true, None)
    }

    fn on_maximize(&mut self, window: WindowRef, maximize: bool) -> Result<(), Error> {
        let s = self.as_maximized(window, maximize)?;
        self.success_handler(s, true, None)
    }

    fn on_focus(&mut self, window: WindowRef) -> Result<(), Error> {
        self.focus_history.update(window);
        self.last_focused_monitor = None;
        Ok(())
    }

    fn on_desktop_focus(&mut self, focus_position: (i32, i32)) -> Result<(), Error> {
        self.last_focused_monitor = self
            .managed_monitors
            .iter()
            .find(|(_, m)| m.info.monitor_area.contains(focus_position))
            .map(|m| m.0.clone());
        Ok(())
    }

    fn on_vd_destroyed(&mut self, destroyed: Desktop, fallback: Desktop) -> Result<(), Error> {
        let old_vd_id = destroyed.get_id().map_err(Error::VDError)?.to_u128();
        self.containers.retain(|k, _| k.vd != old_vd_id);
        self.inactive_containers.retain(|k, _| !k.is_vd(old_vd_id));
        if self.current_vd.is_desktop(&destroyed) {
            return self.on_vd_changed(destroyed, fallback);
        }

        self.update_layout(true, None)
    }

    fn on_vd_changed(&mut self, _previous: Desktop, current: Desktop) -> Result<(), Error> {
        if self.current_vd.is_desktop(&current) {
            return Ok(());
        }

        self.activate_vd(current).map(|_| ())?;
        self.floating_wins.set_all_locked(true);
        self.add_open_windows().ok();

        // INFO: floating windows could be tiled in other VD
        let floating_wins: HashSet<WindowRef> = self.floating_wins.keys().copied().collect();
        floating_wins.iter().for_each(|w| {
            let p = self.floating_wins.remove(w).unwrap();
            self.remove(*w).ok();
            self.floating_wins.insert(*w, p);
        });

        self.update_layout(true, None)
    }

    fn on_workarea_changed(&mut self) -> Result<(), Error> {
        let monitors = enum_display_monitors();
        self.peeked_containers.clear();
        monitors.iter().for_each(|m| {
            self.containers
                .iter_mut()
                .filter(|(k, _)| k.monitor == m.id)
                .for_each(|(_, c)| c.tree_mut().set_base_area(m.get_workspace()));

            self.inactive_containers
                .iter_mut()
                .filter(|(k, _)| k.monitor == m.id)
                .for_each(|(_, c)| c.tree_mut().set_base_area(m.get_workspace()));
        });
        self.managed_monitors = monitors.iter().map(|m| (m.id.clone(), m.clone().into())).collect();
        self.update_layout(true, None)
    }
}

impl TilesManagerCommands for TilesManager {
    fn move_focused(&mut self, direction: Direction, floating_increment: u16) -> Result<(), Error> {
        let src_win = get_foreground().ok_or(Error::NoWindow)?;
        if matches!(self.get_window_state(src_win)?, WindowTileState::Floating) {
            let s = TMFloating::move_window(self, src_win, direction, floating_increment)?;
            return self.success_handler(s, true, None);
        }

        let trg_win = self.find_neighbour(src_win, direction, MonitorSearchStrategy::Any);
        let trg_win = trg_win.ok_or(Error::NoWindow)?.id;

        self.swap_windows(src_win, trg_win)?;
        self.cursor_on_leaf(&self.containers.find_leaf(src_win)?);
        self.update_layout(true, None)
    }

    fn release_focused(&mut self, release: Option<bool>) -> Result<(), Error> {
        let s = self.release(get_foreground().ok_or(Error::NoWindow)?, release, None)?;
        self.success_handler(s, true, None)
    }

    fn insert_focused(&mut self, direction: Direction) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let src_leaf = self.containers.find_leaf(curr)?;
        let point = self
            .containers
            .find_closest_at(src_leaf.viewbox.get_center(), direction)?
            .value
            .tree()
            .get_area()
            .get_center();

        self.insert_window(curr, point, false)?;
        self.cursor_on_leaf(&self.containers.find_leaf(curr)?);
        self.update_layout(true, None)
    }

    fn resize_focused(&mut self, direction: Direction, increment: u16, floating_increment: u16) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let tile_state = self.get_window_state(curr)?;
        if matches!(tile_state, WindowTileState::Focalized | WindowTileState::Maximized) {
            return Ok(());
        }

        if matches!(tile_state, WindowTileState::Floating) {
            let size = match direction {
                Direction::Right | Direction::Down => floating_increment as i16,
                Direction::Left | Direction::Up => -(floating_increment as i16),
            };
            let s = TMFloating::resize(self, curr, direction.axis(), size)?;
            return self.success_handler(s, true, None);
        }

        let orig_area = curr.get_area().ok_or(Error::NoWindowsInfo)?;
        let inc = increment.clamp(0, 500) as i16;
        let has_neigh1 = self
            .find_neighbour(curr, direction, MonitorSearchStrategy::Same)
            .is_some();
        let has_neigh2 = self
            .find_neighbour(curr, direction.opposite(), MonitorSearchStrategy::Same)
            .is_some();

        let get_pad = |neigh1: bool, neigh2: bool, v1: (i16, i16), v2: (i16, i16)| match (neigh1, neigh2) {
            (true, _) => v1,
            (false, true) => v2,
            _ => (0, 0),
        };
        let padding = match direction {
            Direction::Left => (get_pad(has_neigh1, has_neigh2, (inc, 0), (0, -inc)), (0, 0)),
            Direction::Right => (get_pad(has_neigh1, has_neigh2, (0, inc), (-inc, 0)), (0, 0)),
            Direction::Up => ((0, 0), get_pad(has_neigh1, has_neigh2, (inc, 0), (0, -inc))),
            Direction::Down => ((0, 0), get_pad(has_neigh1, has_neigh2, (0, inc), (-inc, 0))),
        };

        let area = orig_area.pad(padding.0, padding.1);
        let s = TMOperations::resize(self, curr, orig_area.get_shift(&area))?;
        self.success_handler(s, true, None)
    }

    fn minimize_focused(&mut self) -> Result<(), Error> {
        get_foreground().ok_or(Error::NoWindow)?.minimize(true);
        Ok(())
    }

    fn close_focused(&mut self) -> Result<(), Error> {
        get_foreground().ok_or(Error::NoWindow)?.close();
        Ok(())
    }

    fn topmost_focused(&mut self, topmost: Option<bool>) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        if matches!(self.get_window_state(curr)?, WindowTileState::Floating) {
            curr.set_topmost(topmost.unwrap_or(!curr.is_topmost())).ok();
        }
        Ok(())
    }

    fn focalize_focused(&mut self) -> Result<(), Error> {
        let curr_win = get_foreground().ok_or(Error::NoWindow)?;
        match self.focalize(curr_win, None)? {
            Success::LayoutChanged => {
                curr_win.focus();
                self.update_layout(true, Some(curr_win))
            }
            s => self.success_handler(s, true, None),
        }
    }

    fn half_focalize_focused(&mut self) -> Result<(), Error> {
        let curr_win = get_foreground().ok_or(Error::NoWindow)?;
        match self.half_focalize(curr_win, None)? {
            Success::LayoutChanged => {
                curr_win.focus();
                self.update_layout(true, Some(curr_win))
            }
            s => self.success_handler(s, true, None),
        }
    }

    fn cycle_focalized_wins(&mut self, next: bool, half: Option<bool>) -> Result<(), Error> {
        let f_win = get_foreground().ok_or(Error::NoWindow)?;
        let e = self.containers.find_mut(f_win)?;

        let container_type = e.value.current();
        let is_right_layer_type = match half {
            Some(true) => container_type == ContainerLayer::HalfFocalized,
            Some(false) => container_type == ContainerLayer::Focalized,
            _ => matches!(
                container_type,
                ContainerLayer::Focalized | ContainerLayer::HalfFocalized
            ),
        };

        if !is_right_layer_type {
            return Ok(());
        }

        let (main_win, others_win): (Vec<_>, Vec<_>) = e.value.tree().get_ids().into_iter().partition(|x| *x == f_win);
        let main_win = match main_win.len() {
            1 => main_win[0],
            _ => return Ok(()),
        };

        let allowed_len = match e.value.current() == ContainerLayer::HalfFocalized {
            true => 1,
            false => 0,
        };

        if others_win.len() != allowed_len {
            return Ok(());
        }

        let wins: Vec<WindowRef> = e
            .value
            .get_tree(ContainerLayer::Normal)
            .get_ids()
            .iter()
            .copied()
            .filter(|w| !others_win.contains(w))
            .collect();

        let next_win = wins
            .iter()
            .position(|w| *w == main_win)
            .and_then(|i| wins.get((i as i8 + if next { 1 } else { -1 }).rem_euclid(wins.len() as i8) as usize))
            .ok_or(Error::Generic)?;

        if main_win == *next_win {
            return Ok(());
        }

        e.value.tree_mut().replace_id(main_win, *next_win);
        next_win.restore(true);
        main_win.minimize(false);

        self.update_layout(false, Some(*next_win))
    }

    fn invert_orientation(&mut self) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let t = self.containers.find_mut(curr)?.value;
        let center = curr.get_area().ok_or(Error::NoWindowsInfo)?.get_center();
        t.tree_mut().switch_subtree_orientations(center);

        self.update_layout(true, None)
    }

    fn focus_workspace(&mut self, workspace_id: &str, monitor_name: Option<&str>) -> Result<(), Error> {
        let bounded = self.config.get_bounded_monitor(workspace_id);
        let monitor_name = if bounded.is_some() {
            bounded.clone()
        } else if let Some(monitor_name) = monitor_name {
            Some(monitor_name.to_uppercase())
        } else if self.last_focused_monitor.is_some() {
            self.last_focused_monitor.clone()
        } else if let Some(c) = get_foreground().and_then(|w| w.get_area().map(|a| a.get_center())) {
            self.managed_monitors
                .iter()
                .find(|(_, m)| m.info.monitor_area.contains(c))
                .map(|m| m.0.clone())
        } else {
            None
        };

        if monitor_name.is_none() {
            return Ok(());
        }

        match self.activate_workspace(&monitor_name.unwrap(), workspace_id, false, bounded.is_none())? {
            Success::UpdateAndFocus { window } => {
                window
                    .filter(|_| self.config.focus_follows_cursor)
                    .and_then(|w| self.containers.find_leaf(w).ok())
                    .inspect(|w| self.cursor_on_leaf(w));
                self.update_layout(false, window)
            }
            s => self.success_handler(s, false, None),
        }
    }

    fn move_focused_to_workspace(
        &mut self,
        workspace_id: &str,
        focus_workspace: bool,
        monitor_name: Option<&str>,
    ) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let tile_state = self.get_window_state(curr)?;
        if matches!(tile_state, WindowTileState::Floating | WindowTileState::Maximized) {
            return Ok(());
        }

        match self.insert_window_to_workspace(curr, workspace_id, monitor_name)? {
            Success::LayoutChanged if !focus_workspace => {
                curr.minimize(true);
                self.update_layout(true, None)
            }
            Success::LayoutChanged if focus_workspace => {
                self.focus_workspace(workspace_id, monitor_name)?;
                self.update_layout(false, Some(curr))
            }
            s => self.success_handler(s, true, None),
        }
    }

    fn change_focus(&mut self, direction: Direction) -> Result<(), Error> {
        if self.last_focused_monitor.is_some() && self.config.focus_on_empty_monitor {
            return self.change_focus_monitor(direction);
        }

        let curr = get_foreground().ok_or(Error::NoWindow)?;
        if matches!(self.get_window_state(curr)?, WindowTileState::Floating) {
            return TMFloating::change_focus(self, curr, direction).map(|_| ());
        }

        let leaf = self.find_neighbour(curr, direction, MonitorSearchStrategy::Any);
        if let Some(leaf) = leaf {
            self.focus_leaf(&leaf);
            Ok(())
        } else if self.config.focus_on_empty_monitor {
            self.change_focus_monitor(direction)
        } else {
            Err(Error::NoWindow)
        }
    }

    fn change_focus_monitor(&mut self, direction: Direction) -> Result<(), Error> {
        let src_area = if let Some(last_focused) = &self.last_focused_monitor {
            let area = self.managed_monitors.get(last_focused).map(|m| m.info.monitor_area);
            area.ok_or(Error::MonitorNotFound(last_focused.clone()))?
        } else {
            let curr = get_foreground().ok_or(Error::NoWindow)?;
            if matches!(self.get_window_state(curr)?, WindowTileState::Floating) {
                return Ok(());
            }
            curr.get_area().ok_or(Error::NoWindowsInfo)?
        };

        let closest = self.containers.find_closest_at(src_area.get_center(), direction)?;
        let monitor = self
            .managed_monitors
            .get(&closest.key.monitor)
            .ok_or(Error::MonitorNotFound(closest.key.monitor.clone()))?;
        self.last_focused_monitor = None;
        if let Some(leaf) = self.get_maximized_leaf_in_monitor(&closest.key) {
            self.focus_leaf(&leaf);
            return Ok(());
        }

        let leaves = closest.value.tree().leaves(None);

        if self.config.history_based_navigation {
            if let Some(leaf) = self.focus_history.latest(&leaves) {
                self.focus_leaf(leaf);
                return Ok(());
            }
        }

        let edge = closest.value.tree().get_area().get_corners(direction.opposite());
        let leaves = leaves_limited_by_edge(&leaves, direction, edge);
        if let Some(leaf) = leaves.first() {
            self.focus_leaf(leaf);
            return Ok(());
        }

        monitor.focus();
        self.last_focused_monitor = Some(closest.key.monitor.clone());
        if self.config.focus_follows_cursor {
            let (x, y) = monitor.info.get_workspace().get_center();
            set_cursor_pos(x, y);
        }

        Ok(())
    }

    fn switch_focus(&mut self) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let tile_state = self.get_window_state(curr)?;
        let center = curr.get_area().ok_or(Error::NoWindowsInfo)?.get_center();
        if matches!(tile_state, WindowTileState::Floating) {
            let wins = self.get_visible_managed_windows();
            wins.iter()
                .filter(|e| *e.0 != curr)
                .filter_map(|e| e.0.get_area().map(|a| (e.0, a.get_center())))
                .min_by(|a, b| center.distance(a.1).cmp(&center.distance(b.1)))
                .ok_or(Error::NoWindow)?
                .0
                .focus();
        } else if !matches!(tile_state, WindowTileState::Maximized) {
            self.floating_wins
                .enabled_keys(&self.current_vd)
                .filter_map(|w| w.get_area().map(|a| (w, a.get_center())))
                .min_by(|a, b| center.distance(a.1).cmp(&center.distance(b.1)))
                .ok_or(Error::NoWindow)?
                .0
                .focus();
        };

        Ok(())
    }

    fn amplify_focused(&mut self) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;

        let tile_state = self.get_window_state(curr)?;
        if matches!(
            tile_state,
            WindowTileState::Focalized | WindowTileState::Maximized | WindowTileState::Floating
        ) {
            return Ok(());
        }

        let leaves = self.containers.find_mut(curr)?.value.tree().leaves(None);
        let max_leaf = leaves
            .iter()
            .max_by(|a, b| a.viewbox.calc_area().cmp(&b.viewbox.calc_area()));

        if max_leaf.is_none_or(|l| l.id == curr) {
            return Ok(());
        }

        self.swap_windows(curr, max_leaf.ok_or(Error::NoWindow)?.id)?;
        self.cursor_on_leaf(&self.containers.find_leaf(curr)?);
        self.update_layout(true, Some(curr))
    }

    fn peek_current(&mut self, direction: Direction, ratio: f32) -> Result<(), Error> {
        let ratio = ratio.clamp(0.1, 0.9);
        let fw = get_foreground().filter(|w| self.containers.find(*w).is_ok());

        let search_point = fw
            .and_then(|w| w.get_area().map(|a| a.get_center()))
            .or(get_cursor_pos().ok())
            .ok_or(Error::Generic)?;

        // NOTE: peek the monitor with the focused window otherwise the one in which the cursor is
        let monitor = self
            .managed_monitors
            .iter()
            .find(|(_, m)| m.info.monitor_area.contains(search_point));
        let monitor = monitor
            .map(|m| m.0.clone())
            .ok_or(Error::NoMonitorAtPoint(search_point))?;

        let k = self.containers.get_key_by_monitor(&monitor)?;
        let c = self.containers.get_mut(&k).ok_or(Error::container_not_found())?;
        let k = k.into();
        if let Some(orig_area) = self.peeked_containers.remove(&k) {
            c.iter_mut().for_each(|t| t.1.set_base_area(orig_area));
        } else {
            let prev_area = c.tree().get_base_area();
            self.peeked_containers.insert(k, prev_area);
            let (w, h) = (prev_area.width as f32, prev_area.height as f32);
            let padding = match direction {
                Direction::Left => (((w * ratio).round() as i16, 0), (0, 0)),
                Direction::Right => ((0, (w * ratio).round() as i16), (0, 0)),
                Direction::Up => ((0, 0), ((h * ratio).round() as i16, 0)),
                Direction::Down => ((0, 0), (0, (h * ratio).round() as i16)),
            };
            let new_area = prev_area.pad(padding.0, padding.1);
            c.iter_mut().for_each(|t| t.1.set_base_area(new_area));
        }

        self.update_layout(true, None)
    }

    fn check_for_vd_changes(&mut self) -> Result<(), Error> {
        let current_vd = self.current_vd;
        let active_vd = get_current_desktop().map_err(Error::VDError)?;

        if !current_vd.is_desktop(&active_vd) {
            self.on_vd_changed(current_vd.get_desktop(), active_vd)?
        }

        Ok(())
    }
}

impl TilesManager {
    fn success_handler(
        &mut self,
        success: Success,
        animate: bool,
        win_to_focus: Option<WindowRef>,
    ) -> Result<(), Error> {
        match success {
            Success::LayoutChanged => self.update_layout(animate, win_to_focus),
            Success::Queue { window, area, topmost } => {
                self.animation_player.queue(window, area, topmost);
                self.update_layout(animate, Some(window))
            }
            Success::Dequeue { window } => {
                self.animation_player.dequeue(window);
                self.update_layout(animate, None)
            }
            Success::UpdateAndFocus { window } => self.update_layout(animate, window),
            _ => Ok(()),
        }
    }

    fn focus_leaf(&mut self, leaf: &AreaLeaf<WindowRef>) {
        leaf.id.focus();
        self.cursor_on_leaf(leaf);
    }

    fn cursor_on_leaf(&self, leaf: &AreaLeaf<WindowRef>) {
        if self.config.focus_follows_cursor {
            let (x, y) = leaf.viewbox.get_center();
            set_cursor_pos(x, y);
        }
    }
}
