use super::error::TilesManagerError;
use super::manager::TilesManager;
use super::manager::TilesManagerBase;
use super::operations::MonitorSearchStrategy;
use super::operations::TilesManagerInternalOperations;
use crate::app::mondrian_message::IntermonitorMoveOp;
use crate::app::mondrian_message::IntramonitorMoveOp;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::area::Area;
use crate::app::structs::direction::Direction;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::utils::get_foreground;
use crate::win32::api::cursor::get_cursor_pos;
use crate::win32::api::cursor::set_cursor_pos;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::collections::VecDeque;
use winvd::get_current_desktop;
use winvd::Desktop;

type IntraOp = IntramonitorMoveOp;
type InterOp = IntermonitorMoveOp;
type Error = TilesManagerError;

pub trait TilesManagerOperations: TilesManagerInternalOperations {
    fn on_open(&mut self, window: WindowRef) -> Result<(), Error>;
    fn on_restore(&mut self, window: WindowRef) -> Result<(), Error>;
    fn on_close(&mut self, win: WindowRef) -> Result<(), Error>;
    fn on_minimize(&mut self, win: WindowRef) -> Result<(), Error>;
    fn swap_focused(&mut self, direction: Direction, center_cursor: bool) -> Result<(), Error>;
    fn release_focused(&mut self, release: Option<bool>) -> Result<(), Error>;
    fn move_focused(&mut self, direction: Direction, center_cursor: bool) -> Result<(), Error>;
    fn resize_focused(&mut self, direction: Direction, size: u8) -> Result<(), Error>;
    fn minimize_focused(&mut self) -> Result<(), Error>;
    fn focalize_focused(&mut self) -> Result<(), Error>;
    fn invert_orientation(&mut self) -> Result<(), Error>;
    fn change_focus(&mut self, direction: Direction, center_mouse: bool) -> Result<(), Error>;
    fn on_move(
        &mut self,
        window: WindowRef,
        target: (i32, i32),
        intra_op: IntraOp,
        inter_op: InterOp,
    ) -> Result<(), Error>;
    fn on_resize(&mut self, window: WindowRef, delta: (i32, i32, i32, i32)) -> Result<(), Error>;
    fn on_maximize(&mut self, window: WindowRef, maximize: bool) -> Result<(), Error>;
    fn on_focus(&mut self, window: WindowRef) -> Result<(), Error>;
    fn amplify_focused(&mut self, center_cursor: bool) -> Result<(), Error>;
    fn peek_current(&mut self, direction: Direction, ratio: f32) -> Result<(), Error>;

    fn check_for_vd_changes(&mut self) -> Result<(), Error>;
    fn on_vd_created(&mut self, desktop: Desktop) -> Result<(), Error>;
    fn on_vd_destroyed(&mut self, destroyed: Desktop, fallback: Desktop) -> Result<(), Error>;
    fn on_vd_changed(&mut self, previous: Desktop, current: Desktop) -> Result<(), Error>;

    fn on_workarea_changed(&mut self) -> Result<(), Error>;
}

impl TilesManagerOperations for TilesManager {
    fn on_open(&mut self, win: WindowRef) -> Result<(), Error> {
        TilesManagerInternalOperations::add(self, win, get_cursor_pos().ok())?;
        self.update_layout(true)
    }

    fn on_restore(&mut self, win: WindowRef) -> Result<(), Error> {
        TilesManagerInternalOperations::add(self, win, None)?;
        self.update_layout(true)
    }

    fn on_close(&mut self, win: WindowRef) -> Result<(), Error> {
        match TilesManagerInternalOperations::remove(self, win, false)? {
            true => self.update_layout(true),
            false => Ok(()),
        }
    }

    fn on_minimize(&mut self, win: WindowRef) -> Result<(), Error> {
        match TilesManagerInternalOperations::remove(self, win, true)? {
            true => self.update_layout(true),
            false => Ok(()),
        }
    }

    fn swap_focused(&mut self, direction: Direction, center_cursor: bool) -> Result<(), Error> {
        let src_win = get_foreground().ok_or(Error::NoWindow)?;

        let src_k = self.active_trees.find(src_win).ok_or(Error::NoWindow)?.key;
        let (src_win, search_strategy) = match self.focalized_wins.get(&src_k) {
            Some(w) => (*w, MonitorSearchStrategy::Closest),
            None => (src_win, MonitorSearchStrategy::Any),
        };

        let trg_win = self
            .find_neighbour(src_win, direction, search_strategy)
            .ok_or(Error::NoWindow)?
            .id;
        let trg_k = self.active_trees.find(trg_win).ok_or(Error::NoWindow)?.key;
        let trg_win = *self.focalized_wins.get(&trg_k).unwrap_or(&trg_win);

        self.swap_windows(src_win, trg_win)?;

        if center_cursor {
            let leaf = self.active_trees.find_leaf(src_win).ok_or(Error::NoWindow)?;
            let (x, y) = leaf.viewbox.get_center();
            set_cursor_pos(x, y);
        }

        self.update_layout(true)
    }

    fn release_focused(&mut self, release: Option<bool>) -> Result<(), Error> {
        self.release(get_foreground().ok_or(Error::NoWindow)?, release)?;
        self.update_layout(true)
    }

    fn move_focused(&mut self, direction: Direction, center_cursor: bool) -> Result<(), Error> {
        const C_ERR: Error = Error::ContainerNotFound { refresh: false };
        let curr = get_foreground().ok_or(Error::NoWindow)?;

        let src_leaf = self
            .active_trees
            .find(curr)
            .ok_or(C_ERR)?
            .value
            .find_leaf(curr, 0)
            .ok_or(Error::NoWindow)?;

        let point = self
            .active_trees
            .find_closest_at_mut(src_leaf.viewbox.get_center(), direction)
            .ok_or(C_ERR)?
            .value
            .get_area()
            .get_center();

        self.move_to(curr, point, false)?;

        if center_cursor {
            let leaf = self.active_trees.find_leaf(curr).ok_or(Error::NoWindow)?;
            let (x, y) = leaf.viewbox.get_center();
            set_cursor_pos(x, y);
        }

        self.update_layout(true)
    }

    fn resize_focused(&mut self, direction: Direction, size: u8) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        if self.get_window_state(curr).is_none() {
            return Err(Error::NoWindow);
        }

        let orig_area = curr.get_area().ok_or(Error::NoWindowsInfo)?;
        let size = size as i16;
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
            Direction::Left => (get_pad(has_neigh1, has_neigh2, (size, 0), (0, -size)), (0, 0)),
            Direction::Right => (get_pad(has_neigh1, has_neigh2, (0, size), (-size, 0)), (0, 0)),
            Direction::Up => ((0, 0), get_pad(has_neigh1, has_neigh2, (size, 0), (0, -size))),
            Direction::Down => ((0, 0), get_pad(has_neigh1, has_neigh2, (0, size), (-size, 0))),
        };

        let area = orig_area.pad(padding.0, padding.1);
        self.resize(curr, orig_area.get_shift(&area))?;
        self.update_layout(true)
    }

    fn minimize_focused(&mut self) -> Result<(), Error> {
        get_foreground().ok_or(Error::NoWindow)?.minimize();
        Ok(())
    }

    fn focalize_focused(&mut self) -> Result<(), Error> {
        let curr_win = get_foreground().ok_or(Error::NoWindow)?;
        self.focalize(curr_win, None)?;
        self.update_layout(true)
    }

    fn invert_orientation(&mut self) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let t = self.active_trees.find_mut(curr).ok_or(Error::NoWindow)?.value;
        let center = curr.get_area().ok_or(Error::NoWindowsInfo)?.get_center();
        t.switch_subtree_orientations(center);

        self.update_layout(true)
    }

    fn change_focus(&mut self, direction: Direction, center_cursor: bool) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let leaf = self.find_neighbour(curr, direction, MonitorSearchStrategy::Any);
        let leaf = leaf.ok_or(Error::NoWindow)?;
        leaf.id.focus();
        if center_cursor {
            let (x, y) = leaf.id.get_area().ok_or(Error::NoWindowsInfo)?.get_center();
            set_cursor_pos(x, y);
        }

        Ok(())
    }

    fn on_move(
        &mut self,
        win: WindowRef,
        target: (i32, i32),
        intra_op: IntraOp,
        inter_op: InterOp,
    ) -> Result<(), Error> {
        let tile_state = self.get_window_state(win).ok_or(Error::NoWindow)?;
        if matches!(tile_state, WindowTileState::Floating | WindowTileState::Maximized) {
            return Ok(());
        }

        const C_ERR: Error = Error::ContainerNotFound { refresh: true };

        let src_win = self.active_trees.find(win).ok_or(C_ERR)?;
        let src_win = src_win.value.find_leaf(win, 0).ok_or(Error::NoWindow)?.id;
        let trg_leaf = self.active_trees.find_at(target);
        let trg_leaf = trg_leaf.and_then(|t| t.value.find_leaf_at(target, 0));

        let src_k = self.active_trees.find(src_win).ok_or(C_ERR)?.key;
        let trg_k = self.active_trees.find_at(target).ok_or(C_ERR)?.key;
        if src_k == trg_k {
            // If it is in the same monitor

            // If it is focalized, do nothing
            if self.focalized_wins.contains_key(&src_k) {
                return self.update_layout(true);
            }

            let t = self.active_trees.find_mut(src_win).ok_or(C_ERR)?.value;
            if matches!(intra_op, IntraOp::InsertFreeMove) {
                t.move_to(src_win, target);
            } else if let Some(leaf) = trg_leaf {
                t.swap_ids(src_win, leaf.id);
            }
        } else if matches!(inter_op, InterOp::Insert | InterOp::Invert | InterOp::InsertFreeMove) || trg_leaf.is_none()
        {
            // If it is in another monitor and insert
            self.move_to(win, target, matches!(inter_op, InterOp::InsertFreeMove))?;
        } else {
            // If it is in another monitor and swap
            let src_win = *self.focalized_wins.get(&src_k).unwrap_or(&src_win);
            if trg_leaf.is_none() {
                self.move_to(win, target, false)?;
            } else if let Some(trg_leaf) = trg_leaf {
                let trg_win = *self.focalized_wins.get(&trg_k).unwrap_or(&trg_leaf.id);
                self.swap_windows(src_win, trg_win)?;
            }
        };

        let switch_orient = match src_k == trg_k {
            true => matches!(intra_op, IntraOp::Invert),
            false => matches!(inter_op, InterOp::Invert),
        };

        if switch_orient {
            let tree = self.active_trees.find_at_mut(target);
            tree.ok_or(C_ERR)?.value.switch_subtree_orientations(target);
        }

        self.update_layout(true)
    }

    fn on_resize(&mut self, win: WindowRef, delta: (i32, i32, i32, i32)) -> Result<(), Error> {
        self.resize(win, delta)?;
        self.update_layout(true)
    }

    fn on_maximize(&mut self, window: WindowRef, maximize: bool) -> Result<(), Error> {
        self.maximize(window, maximize)?;
        self.update_layout(true)
    }

    fn on_focus(&mut self, window: WindowRef) -> Result<(), Error> {
        self.focus_history.update(window);
        Ok(())
    }

    fn amplify_focused(&mut self, center_cursor: bool) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;

        let tile_state = self.get_window_state(curr).ok_or(Error::NoWindow)?;
        if matches!(
            tile_state,
            WindowTileState::Focalized | WindowTileState::Maximized | WindowTileState::Floating
        ) {
            return Ok(());
        }

        let t = self.active_trees.find_mut(curr).ok_or(Error::NoWindow)?.value;
        let leaves = t.leaves(0, None);
        let max_leaf = leaves
            .iter()
            .max_by(|a, b| a.viewbox.get_area().cmp(&b.viewbox.get_area()));

        if max_leaf.is_none() || max_leaf.is_some_and(|l| l.id == curr) {
            return Ok(());
        }

        self.swap_windows(curr, max_leaf.ok_or(Error::NoWindow)?.id)?;

        if center_cursor {
            let leaf = self.active_trees.find_leaf(curr).ok_or(Error::NoWindow)?;
            let (x, y) = leaf.viewbox.get_center();
            set_cursor_pos(x, y);
        }

        self.update_layout(true)
    }

    fn peek_current(&mut self, direction: Direction, ratio: f32) -> Result<(), Error> {
        let ratio = ratio.clamp(0.1, 0.9);
        let fw = get_foreground().ok_or(Error::NoWindow)?;

        // NOTE: peek the monitor with the focused window otherwise the one in which the cursor is
        let (k, t) = match self.active_trees.find_mut(fw) {
            Some(e) => (e.key, e.value),
            None => {
                let cursor_pos = get_cursor_pos().map_err(|_| Error::Generic)?;
                match self.active_trees.find_at_mut(cursor_pos) {
                    Some(e) => (e.key, e.value),
                    None => match self.peeked_containers.iter().find(|(_, v)| v.contains(cursor_pos)) {
                        Some(e) => (e.0.clone(), self.active_trees.get_mut(e.0).ok_or(Error::NoWindow)?),
                        None => return Err(Error::Generic),
                    },
                }
            }
        };

        if let Some(orig_area) = self.peeked_containers.remove(&k) {
            t.set_area(orig_area);
        } else {
            let prev_area = t.get_area();
            self.peeked_containers.insert(k, prev_area);
            let (w, h) = (prev_area.width as f32, prev_area.height as f32);
            let padding = match direction {
                Direction::Left => (((w * ratio).round() as i16, 0), (0, 0)),
                Direction::Right => ((0, (w * ratio).round() as i16), (0, 0)),
                Direction::Up => ((0, 0), ((h * ratio).round() as i16, 0)),
                Direction::Down => ((0, 0), (0, (h * ratio).round() as i16)),
            };
            t.set_area(prev_area.pad(padding.0, padding.1));
        }

        self.update_layout(true)
    }

    fn check_for_vd_changes(&mut self) -> Result<(), Error> {
        let current_vd = self.current_vd.as_ref().ok_or(Error::Generic)?;
        let active_vd = get_current_desktop().map_err(Error::VirtualDesktopError)?;

        if current_vd
            .get_id()
            .is_ok_and(|id| active_vd.get_id().is_ok_and(|id2| id != id2))
        {
            self.on_vd_changed(*current_vd, active_vd)?
        }

        Ok(())
    }

    fn on_vd_created(&mut self, desktop: Desktop) -> Result<(), Error> {
        let vd_id = desktop.get_id().map_err(Error::VirtualDesktopError)?.to_u128();

        if self
            .active_trees
            .iter()
            .chain(self.inactive_trees.iter())
            .any(|(k, _)| k.is_virtual_desktop(vd_id))
        {
            return Ok(());
        }

        let containers = self.build_vd_containers(desktop)?;
        self.inactive_trees.extend(containers);
        self.update_layout(true)
    }

    fn on_vd_destroyed(&mut self, destroyed: Desktop, fallback: Desktop) -> Result<(), Error> {
        let old_vd_id = destroyed.get_id().map_err(Error::VirtualDesktopError)?.to_u128();
        self.active_trees.retain(|k, _| !k.is_virtual_desktop(old_vd_id));
        self.inactive_trees.retain(|k, _| !k.is_virtual_desktop(old_vd_id));
        if self.current_vd == Some(destroyed) {
            return self.on_vd_changed(destroyed, fallback);
        }

        self.update_layout(true)
    }

    fn on_vd_changed(&mut self, _previous: Desktop, current: Desktop) -> Result<(), Error> {
        let curr_desk_id = current.get_id().map_err(Error::VirtualDesktopError)?.to_u128();

        if self.current_vd == Some(current) {
            return Ok(());
        }

        if self
            .active_trees
            .iter()
            .chain(self.inactive_trees.iter())
            .all(|(k, _)| !k.is_virtual_desktop(curr_desk_id))
        {
            let containers = self.build_vd_containers(current)?;
            self.inactive_trees.extend(containers);
        }

        self.activate_containers(Some(current), None, None)?;
        self.current_vd = Some(current);
        self.add_open_windows()?;
        self.update_layout(true)
    }

    fn on_workarea_changed(&mut self) -> Result<(), Error> {
        enum_display_monitors().iter().for_each(|m| {
            self.active_trees
                .iter_mut()
                .chain(self.inactive_trees.iter_mut())
                .filter(|(k, _)| k.is_monitor(&m.id))
                .for_each(|(_, c)| c.set_area(Area::from(m.clone())));
        });
        self.update_layout(true)
    }
}

#[derive(Default, Clone, Debug)]
pub struct FocusHistory {
    map: HashMap<WindowRef, u64>,
    order: VecDeque<WindowRef>,
    order_map: HashMap<WindowRef, usize>,
    current_max: u64,
}

impl FocusHistory {
    const MAX_ENTRIES: usize = 100; // TODO: maybe should be configurable?

    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            order_map: HashMap::new(),
            current_max: 0,
        }
    }

    pub fn value(&self, window: WindowRef) -> Option<u64> {
        self.map.get(&window).copied()
    }

    pub fn update(&mut self, window: WindowRef) {
        // INFO: this should pratically never happen
        if self.current_max == u64::MAX {
            let min_v = *self.map.values().min().unwrap_or(&0);
            self.map.values_mut().for_each(|v| *v -= min_v);
            self.current_max -= min_v;
        }

        if let Some(p) = self.order_map.get(&window) {
            self.order.remove(*p);
        }

        self.current_max += 1;
        self.order.push_back(window);
        self.order_map.insert(window, self.order.len() - 1);
        self.map.insert(window, self.current_max);

        // INFO: prevent map from growing indefinitely
        if self.order.len() > Self::MAX_ENTRIES {
            let w = self.order.pop_front().unwrap();
            self.map.remove(&w);
            self.order_map.remove(&w);
        }
    }
}
