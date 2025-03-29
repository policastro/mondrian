use super::floating::FloatingWindows;
use super::floating::TilesManagerFloating;
use super::manager::TilesManager;
use super::manager::TilesManagerBase;
use super::operations::MonitorSearchStrategy;
use super::operations::TilesManagerInternalOperations;
use super::result::TilesManagerError;
use super::result::TilesManagerSuccess;
use crate::app::mondrian_message::IntermonitorMoveOp;
use crate::app::mondrian_message::IntramonitorMoveOp;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::direction::Direction;
use crate::app::structs::point::Point;
use crate::modules::tiles_manager::lib::containers::inactive::InactiveContainers;
use crate::modules::tiles_manager::lib::containers::layer::ContainerLayer;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::containers::ContainersMut;
use crate::modules::tiles_manager::lib::utils::get_foreground;
use crate::modules::tiles_manager::lib::utils::is_on_current_vd;
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

pub trait TilesManagerOperations: TilesManagerInternalOperations {
    // NOTE: Windows events
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
    fn on_vd_created(&mut self, desktop: Desktop) -> Result<(), Error>;
    fn on_vd_destroyed(&mut self, destroyed: Desktop, fallback: Desktop) -> Result<(), Error>;
    fn on_vd_changed(&mut self, previous: Desktop, current: Desktop) -> Result<(), Error>;
    fn on_workarea_changed(&mut self) -> Result<(), Error>;
    // -------------------------

    // NOTE: Actions
    fn move_focused(&mut self, direction: Direction, center_cursor: bool, floating_increment: u16)
        -> Result<(), Error>;
    fn release_focused(&mut self, release: Option<bool>) -> Result<(), Error>;
    fn insert_focused(&mut self, direction: Direction, center_cursor: bool) -> Result<(), Error>;
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
    fn change_focus(&mut self, direction: Direction, center_mouse: bool) -> Result<(), Error>;
    fn switch_focus(&mut self) -> Result<(), Error>;
    fn amplify_focused(&mut self, center_cursor: bool) -> Result<(), Error>;

    /// Limits the tiling to a portion of the screen.
    /// The action is propagated to all inactive containers with:
    /// - vd == currently active virtual desktop
    /// - monitor == monitor where the focused window is located.
    fn peek_current(&mut self, direction: Direction, ratio: f32) -> Result<(), Error>;
    // -------------------------

    fn check_for_vd_changes(&mut self) -> Result<(), Error>;
}

impl TilesManagerOperations for TilesManager {
    fn on_open(&mut self, win: WindowRef) -> Result<(), Error> {
        self.floating_wins.set_properties(&win, false, false);
        match TilesManagerInternalOperations::add(self, win, get_cursor_pos().ok(), true)? {
            Success::LayoutChanged => self.update_layout(true, Some(win)),
            _ => Ok(()),
        }
    }

    fn on_close(&mut self, win: WindowRef) -> Result<(), Error> {
        self.floating_wins.set_minimized(&win, false);

        // INFO: When floating windows are pinned to all VD, they remain locked (because no
        // open/restore event is fired). In this case, we need to unlock them manually.
        if self.floating_wins.locked(&win).unwrap_or(false) && is_on_current_vd(&win).unwrap_or(true) {
            self.floating_wins.set_locked(&win, false);
        }

        match TilesManagerInternalOperations::remove(self, win)? {
            Success::LayoutChanged => self.update_layout(true, None),
            _ => Ok(()),
        }
    }

    fn on_restore(&mut self, win: WindowRef) -> Result<(), Error> {
        self.floating_wins.set_properties(&win, false, false);
        match TilesManagerInternalOperations::add(self, win, None, false)? {
            Success::LayoutChanged => self.update_layout(true, Some(win)),
            _ => Ok(()),
        }
    }

    fn on_minimize(&mut self, win: WindowRef) -> Result<(), Error> {
        self.floating_wins.set_properties(&win, true, false);
        match TilesManagerInternalOperations::remove(self, win)? {
            Success::LayoutChanged => self.update_layout(true, None),
            _ => Ok(()),
        }
    }

    fn on_resize(&mut self, win: WindowRef, delta: (i32, i32, i32, i32)) -> Result<(), Error> {
        match TilesManagerInternalOperations::resize(self, win, delta)? {
            Success::LayoutChanged => self.update_layout(true, None),
            _ => Ok(()),
        }
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

        let src_win = self.active_trees.find_leaf(win)?.id;
        let trg_win = self.active_trees.find_leaf_at(target).map(|l| l.id);

        let src_k = self.active_trees.find(src_win)?.key;
        let trg_k = match self.active_trees.find_at(target) {
            Ok(e) => e.key,
            Err(_) => return self.update_layout(true, None), // INFO: update if no container at target
        };
        if src_k == trg_k {
            // If it is in the same monitor
            let t = self.active_trees.find_mut(src_win)?.value;
            if matches!(intra_op, IntraOp::InsertFreeMove) {
                t.move_to(src_win, target);
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
            let tree = self.active_trees.find_at_mut(target)?;
            tree.value.switch_subtree_orientations(target);
        }

        self.update_layout(true, None)
    }

    fn on_maximize(&mut self, window: WindowRef, maximize: bool) -> Result<(), Error> {
        match self.maximize(window, maximize)? {
            Success::LayoutChanged => self.update_layout(true, None),
            _ => Ok(()),
        }
    }

    fn on_focus(&mut self, window: WindowRef) -> Result<(), Error> {
        self.focus_history.update(window);
        Ok(())
    }

    fn on_vd_created(&mut self, desktop: Desktop) -> Result<(), Error> {
        let vd_id = desktop.get_id().map_err(Error::VDError)?.to_u128();

        if self.active_trees.keys().any(|k| k.vd == vd_id) || self.inactive_trees.keys().any(|k| k.vd == vd_id) {
            return Ok(());
        }

        self.create_inactive_vd_containers(desktop)?;
        self.update_layout(true, None)
    }

    fn on_vd_destroyed(&mut self, destroyed: Desktop, fallback: Desktop) -> Result<(), Error> {
        let old_vd_id = destroyed.get_id().map_err(Error::VDError)?.to_u128();
        self.active_trees.retain(|k, _| k.vd != old_vd_id);
        self.inactive_trees.retain(|k, _| !k.is_vd(old_vd_id));
        if self.current_vd == Some(destroyed) {
            return self.on_vd_changed(destroyed, fallback);
        }

        self.update_layout(true, None)
    }

    fn on_vd_changed(&mut self, _previous: Desktop, current: Desktop) -> Result<(), Error> {
        let vd_id = current.get_id().map_err(Error::VDError)?.to_u128();

        if self.current_vd == Some(current) {
            return Ok(());
        }

        if !self.inactive_trees.has_vd(vd_id) {
            self.create_inactive_vd_containers(current)?;
        }

        self.activate_vd_containers(current, None)?;
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
        monitors.iter().for_each(|m| {
            self.active_trees
                .iter_mut()
                .filter(|(k, _)| k.monitor == m.id)
                .for_each(|(_, c)| c.set_area(m.workspace_area));

            self.inactive_trees
                .iter_mut()
                .filter(|(k, _)| k.monitor == m.id)
                .for_each(|(_, c)| c.0.set_area(m.workspace_area));
        });
        self.managed_monitors = monitors.iter().map(|m| (m.id.clone(), m.clone())).collect();
        self.update_layout(true, None)
    }

    fn move_focused(
        &mut self,
        direction: Direction,
        center_cursor: bool,
        floating_increment: u16,
    ) -> Result<(), Error> {
        let src_win = get_foreground().ok_or(Error::NoWindow)?;
        if matches!(self.get_window_state(src_win)?, WindowTileState::Floating) {
            return match TilesManagerFloating::move_window(self, src_win, direction, floating_increment)? {
                Success::Queue { window, area, topmost } => {
                    self.animation_player.queue(window, area, topmost);
                    return self.update_layout(true, Some(window));
                }
                _ => Ok(()),
            };
        }

        let trg_win = self.find_neighbour(src_win, direction, MonitorSearchStrategy::Any);
        let trg_win = trg_win.ok_or(Error::NoWindow)?.id;

        self.swap_windows(src_win, trg_win)?;

        if center_cursor {
            let (x, y) = self.active_trees.find_leaf(src_win)?.viewbox.get_center();
            set_cursor_pos(x, y);
        }

        self.update_layout(true, None)
    }

    fn release_focused(&mut self, release: Option<bool>) -> Result<(), Error> {
        match self.release(get_foreground().ok_or(Error::NoWindow)?, release)? {
            Success::LayoutChanged => self.update_layout(true, None),
            _ => Ok(()),
        }
    }

    fn insert_focused(&mut self, direction: Direction, center_cursor: bool) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let src_leaf = self.active_trees.find_leaf(curr)?;
        let point = self
            .active_trees
            .find_closest_at(src_leaf.viewbox.get_center(), direction)?
            .value
            .get_area()
            .get_center();

        self.insert_window(curr, point, false)?;

        if center_cursor {
            let (x, y) = self.active_trees.find_leaf(curr)?.viewbox.get_center();
            set_cursor_pos(x, y);
        }

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
            return match TilesManagerFloating::resize(self, curr, direction.axis(), size)? {
                Success::Queue { window, area, topmost } => {
                    self.animation_player.queue(window, area, topmost);
                    return self.update_layout(true, Some(window));
                }
                _ => Ok(()),
            };
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
        match TilesManagerInternalOperations::resize(self, curr, orig_area.get_shift(&area))? {
            Success::LayoutChanged => self.update_layout(true, None),
            _ => Ok(()),
        }
    }

    fn minimize_focused(&mut self) -> Result<(), Error> {
        get_foreground().ok_or(Error::NoWindow)?.minimize();
        Ok(())
    }

    fn close_focused(&mut self) -> Result<(), Error> {
        get_foreground().ok_or(Error::NoWindow)?.close();
        Ok(())
    }

    fn topmost_focused(&mut self, topmost: Option<bool>) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        curr.set_topmost(topmost.unwrap_or(!curr.is_topmost())).ok();
        Ok(())
    }

    fn focalize_focused(&mut self) -> Result<(), Error> {
        let curr_win = get_foreground().ok_or(Error::NoWindow)?;
        match self.focalize(curr_win, None)? {
            Success::LayoutChanged => self.update_layout(true, Some(curr_win)),
            _ => Ok(()),
        }
    }

    fn half_focalize_focused(&mut self) -> Result<(), Error> {
        let curr_win = get_foreground().ok_or(Error::NoWindow)?;
        match self.half_focalize(curr_win, None)? {
            Success::LayoutChanged => self.update_layout(true, Some(curr_win)),
            _ => Ok(()),
        }
    }

    fn cycle_focalized_wins(&mut self, next: bool, half: Option<bool>) -> Result<(), Error> {
        let f_win = get_foreground().ok_or(Error::NoWindow)?;
        let e = self.active_trees.find_mut(f_win)?;

        let is_right_layer_type = match half {
            Some(true) => e.key.layer == ContainerLayer::HalfFocalized,
            Some(false) => e.key.layer == ContainerLayer::Focalized,
            _ => e.key.layer.is_focalized(),
        };

        if !is_right_layer_type {
            return Ok(());
        }

        let (main_win, others_win): (Vec<_>, Vec<_>) = e.value.get_ids().into_iter().partition(|x| *x == f_win);
        let main_win = match main_win.len() {
            1 => main_win[0],
            _ => return Ok(()),
        };

        let allowed_len = match e.key.layer == ContainerLayer::HalfFocalized {
            true => 1,
            false => 0,
        };
        if others_win.len() != allowed_len {
            return Ok(());
        }

        let wins: Vec<WindowRef> = self
            .inactive_trees
            .get_normal(&e.key.into())?
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

        e.value.replace_id(main_win, *next_win);
        next_win.restore(true);
        main_win.minimize();

        self.update_layout(false, Some(*next_win))
    }

    fn invert_orientation(&mut self) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let t = self.active_trees.find_mut(curr)?.value;
        let center = curr.get_area().ok_or(Error::NoWindowsInfo)?.get_center();
        t.switch_subtree_orientations(center);

        self.update_layout(true, None)
    }

    fn change_focus(&mut self, direction: Direction, center_cursor: bool) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;

        if matches!(self.get_window_state(curr)?, WindowTileState::Floating) {
            return TilesManagerFloating::change_focus(self, curr, direction, center_cursor).map(|_| ());
        }

        let leaf = self.find_neighbour(curr, direction, MonitorSearchStrategy::Any);
        let leaf = leaf.ok_or(Error::NoWindow)?;
        leaf.id.focus();
        if center_cursor {
            let (x, y) = leaf.id.get_area().ok_or(Error::NoWindowsInfo)?.get_center();
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
                .enabled_keys()
                .filter_map(|w| w.get_area().map(|a| (w, a.get_center())))
                .min_by(|a, b| center.distance(a.1).cmp(&center.distance(b.1)))
                .ok_or(Error::NoWindow)?
                .0
                .focus();
        };

        Ok(())
    }

    fn amplify_focused(&mut self, center_cursor: bool) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;

        let tile_state = self.get_window_state(curr)?;
        if matches!(
            tile_state,
            WindowTileState::Focalized | WindowTileState::Maximized | WindowTileState::Floating
        ) {
            return Ok(());
        }

        let leaves = self.active_trees.find_mut(curr)?.value.leaves(0, None);
        let max_leaf = leaves
            .iter()
            .max_by(|a, b| a.viewbox.get_area().cmp(&b.viewbox.get_area()));

        if max_leaf.is_none_or(|l| l.id == curr) {
            return Ok(());
        }

        self.swap_windows(curr, max_leaf.ok_or(Error::NoWindow)?.id)?;

        if center_cursor {
            let (x, y) = self.active_trees.find_leaf(curr)?.viewbox.get_center();
            set_cursor_pos(x, y);
        }

        self.update_layout(true, Some(curr))
    }

    fn peek_current(&mut self, direction: Direction, ratio: f32) -> Result<(), Error> {
        let ratio = ratio.clamp(0.1, 0.9);
        let fw = get_foreground().ok_or(Error::NoWindow)?;

        // NOTE: peek the monitor with the focused window otherwise the one in which the cursor is
        let (k, t) = match self.active_trees.find_mut(fw) {
            Ok(e) => (e.key, e.value),
            Err(_) => {
                let cursor_pos = get_cursor_pos().map_err(|_| Error::Generic)?;
                match self.active_trees.find_at_mut(cursor_pos) {
                    Ok(e) => (e.key, e.value),
                    Err(_) => match self.peeked_containers.iter().find(|(_, v)| v.contains(cursor_pos)) {
                        Some(e) => (e.0.clone(), self.active_trees.get_mut(e.0).ok_or(Error::NoWindow)?),
                        None => return Err(Error::Generic),
                    },
                }
            }
        };

        if let Some(orig_area) = self.peeked_containers.remove(&k) {
            t.set_area(orig_area);
            self.inactive_trees.set_layers_area(&k, orig_area);
        } else {
            let prev_area = t.get_area();
            self.peeked_containers.insert(k.clone(), prev_area);
            let (w, h) = (prev_area.width as f32, prev_area.height as f32);
            let padding = match direction {
                Direction::Left => (((w * ratio).round() as i16, 0), (0, 0)),
                Direction::Right => ((0, (w * ratio).round() as i16), (0, 0)),
                Direction::Up => ((0, 0), ((h * ratio).round() as i16, 0)),
                Direction::Down => ((0, 0), (0, (h * ratio).round() as i16)),
            };
            let new_area = prev_area.pad(padding.0, padding.1);
            t.set_area(new_area);
            self.inactive_trees.set_layers_area(&k, new_area);
        }

        self.update_layout(true, None)
    }

    fn check_for_vd_changes(&mut self) -> Result<(), Error> {
        let current_vd = self.current_vd.as_ref().ok_or(Error::Generic)?;
        let active_vd = get_current_desktop().map_err(Error::VDError)?;

        if current_vd
            .get_id()
            .is_ok_and(|id| active_vd.get_id().is_ok_and(|id2| id != id2))
        {
            self.on_vd_changed(*current_vd, active_vd)?
        }

        Ok(())
    }
}
