use std::collections::HashMap;

use winvd::{get_current_desktop, Desktop};

use super::error::TilesManagerError;
use super::manager::{ContainerKey, TilesManager, TilesManagerBase};
use super::operations::{MonitorSearchStrategy, TilesManagerInternalOperations};
use crate::app::area_tree::tree::WinTree;
use crate::app::mondrian_message::{IntermonitorMoveOp, IntramonitorMoveOp, WindowTileState};
use crate::app::structs::direction::Direction;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::utils::get_foreground;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::api::window::{enum_user_manageable_windows, get_foreground_window};
use crate::win32::window::window_obj::{WindowObjHandler, WindowObjInfo};
use crate::win32::window::window_ref::WindowRef;

type IntraOp = IntramonitorMoveOp;
type InterOp = IntermonitorMoveOp;
type Error = TilesManagerError;

pub trait TilesManagerOperations: TilesManagerInternalOperations {
    fn add(&mut self, win: WindowRef) -> Result<(), Error>;
    fn remove(&mut self, win: WindowRef, skip_focalized: bool) -> Result<(), Error>;
    fn swap_focused(&mut self, direction: Direction) -> Result<(), Error>;
    fn release_focused(&mut self, release: Option<bool>) -> Result<(), Error>;
    fn move_focused(&mut self, direction: Direction) -> Result<(), Error>;
    fn resize_focused(&mut self, direction: Direction, size: u8) -> Result<(), Error>;
    fn minimize_focused(&mut self) -> Result<(), Error>;
    fn focalize_focused(&mut self) -> Result<(), Error>;
    fn invert_orientation(&mut self) -> Result<(), Error>;
    fn change_focus(&mut self, direction: Direction) -> Result<(), Error>;
    fn on_move(
        &mut self,
        window: WindowRef,
        target: (i32, i32),
        intra_op: IntraOp,
        inter_op: InterOp,
    ) -> Result<(), Error>;
    fn on_resize(&mut self, window: WindowRef, delta: (i32, i32, i32, i32)) -> Result<(), Error>;
    fn on_maximize(&mut self, window: WindowRef, maximize: bool) -> Result<(), Error>;

    fn check_for_vd_changes(&mut self) -> Result<(), Error>;
    fn on_vd_created(&mut self, desktop: Desktop) -> Result<(), Error>;
    fn on_vd_destroyed(&mut self, destroyed: Desktop, fallback: Desktop) -> Result<(), Error>;
    fn on_vd_changed(&mut self, previous: Desktop, current: Desktop) -> Result<(), Error>;
}

impl TilesManagerOperations for TilesManager {
    fn add(&mut self, win: WindowRef) -> Result<(), Error> {
        TilesManagerInternalOperations::add(self, win)?;
        self.update_layout(true)
    }

    fn remove(&mut self, win: WindowRef, skip_focalized: bool) -> Result<(), Error> {
        match TilesManagerInternalOperations::remove(self, win, skip_focalized)? {
            true => self.update_layout(true),
            false => Ok(()),
        }
    }

    fn swap_focused(&mut self, direction: Direction) -> Result<(), Error> {
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
        self.update_layout(true)
    }

    fn release_focused(&mut self, release: Option<bool>) -> Result<(), Error> {
        self.release(get_foreground().ok_or(Error::NoWindow)?, release)?;
        self.update_layout(true)
    }

    fn move_focused(&mut self, direction: Direction) -> Result<(), Error> {
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
            .area
            .get_center();

        self.move_to(curr, point, false)?;
        self.update_layout(true)
    }

    fn resize_focused(&mut self, direction: Direction, size: u8) -> Result<(), Error> {
        let curr = get_foreground_window().ok_or(Error::NoWindow)?.into();
        if self.get_window_state(curr).is_none() {
            return Err(Error::NoWindow);
        }

        let orig_area = curr.get_window_box().ok_or(Error::NoWindowsInfo)?;
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
        let win_ref = WindowRef::new(get_foreground_window().ok_or(Error::NoWindow)?);
        win_ref.minimize();
        self.find_next_neighbour(win_ref, MonitorSearchStrategy::Any)
            .inspect(|leaf| leaf.id.focus());
        Ok(())
    }

    fn focalize_focused(&mut self) -> Result<(), Error> {
        let curr_win = get_foreground().ok_or(Error::NoWindow)?;
        self.focalize(curr_win, None)?;
        self.update_layout(true)
    }

    fn invert_orientation(&mut self) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let t = self.active_trees.find_mut(curr).ok_or(Error::Generic)?.value;
        let center = curr.get_window_box().ok_or(Error::NoWindowsInfo)?.get_center();
        t.switch_subtree_orientations(center);

        self.update_layout(true)
    }

    fn change_focus(&mut self, direction: Direction) -> Result<(), Error> {
        let curr = get_foreground().ok_or(Error::NoWindow)?;
        let leaf = self.find_neighbour(curr, direction, MonitorSearchStrategy::Any);
        leaf.ok_or(Error::NoWindow)?.id.focus();

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
        let src_win = src_win.value.find_leaf(win, 0).ok_or(Error::Generic)?.id;
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

    fn check_for_vd_changes(&mut self) -> Result<(), Error> {
        let current_vd = self.current_vd.as_ref().ok_or(Error::Generic)?;
        let active_vd = get_current_desktop().map_err(|_| Error::Generic)?;

        if current_vd
            .get_id()
            .is_ok_and(|id| active_vd.get_id().is_ok_and(|id2| id != id2))
        {
            self.on_vd_changed(*current_vd, active_vd)?;
            let filter = self.config.filter.clone();
            enum_user_manageable_windows()
                .iter()
                .filter(|w| !filter.matches(**w))
                .for_each(|w| {
                    let _ = TilesManagerInternalOperations::add(self, *w);
                });
            return self.update_layout(true);
        }

        Ok(())
    }

    fn on_vd_created(&mut self, desktop: Desktop) -> Result<(), Error> {
        let desktop_id = desktop.get_id().map_err(|_| Error::Generic)?.to_u128();

        let monitors = enum_display_monitors();
        let containers: HashMap<ContainerKey, WinTree> = monitors
            .into_iter()
            .map(|m| {
                let t = WinTree::new(m.clone().into(), self.config.layout_strategy.clone());
                (ContainerKey::new(desktop_id, m.id, String::new()), t)
            })
            .collect();

        self.inactive_trees.extend(containers);
        self.update_layout(true)
    }

    fn on_vd_destroyed(&mut self, destroyed: Desktop, fallback: Desktop) -> Result<(), Error> {
        let destroyed_id = destroyed.get_id().map_err(|_| Error::Generic)?.to_u128();

        self.inactive_trees.retain(|k, _| k.virtual_desktop != destroyed_id);
        self.active_trees.retain(|k, _| k.virtual_desktop != destroyed_id);

        if self.current_vd == Some(destroyed) {
            return self.on_vd_changed(destroyed, fallback);
        }

        self.update_layout(true)
    }

    fn on_vd_changed(&mut self, previous: Desktop, current: Desktop) -> Result<(), Error> {
        let prev_desk_id = previous.get_id().map_err(|_| Error::Generic)?.to_u128();
        let curr_desk_id = current.get_id().map_err(|_| Error::Generic)?.to_u128();

        if self.active_trees.keys().any(|k| k.virtual_desktop != prev_desk_id) {
            return Ok(());
        }

        self.inactive_trees.extend(self.active_trees.drain());

        let active_keys: Vec<ContainerKey> = self
            .inactive_trees
            .keys()
            .filter(|k| k.virtual_desktop == curr_desk_id)
            .cloned()
            .collect();

        self.active_trees.extend(
            active_keys
                .into_iter()
                .map(|k| (k.clone(), self.inactive_trees.remove(&(k.clone())).unwrap())),
        );

        self.current_vd = Some(current);
        self.update_layout(true)
    }
}
