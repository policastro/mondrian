use super::error::TilesManagerError;
use super::manager::TilesManager;
use super::manager::TilesManagerBase;
use super::success::TilesManagerSuccess;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use crate::modules::tiles_manager::lib::containers::inactive::InactiveContainers;
use crate::modules::tiles_manager::lib::containers::layer::ContainerLayer;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::modules::tiles_manager::lib::containers::ContainersMut;
use crate::modules::tiles_manager::lib::utils::get_floating_win_area;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;

type Success = TilesManagerSuccess;
type Error = TilesManagerError;
type TileState = WindowTileState;
type TMResult = Result<TilesManagerSuccess, Error>;

pub trait TilesManagerInternalOperations: TilesManagerBase {
    /// Adds a window to the tile manager.
    /// When `prefer_position` is Some, the window will be added to the monitor that contains the
    /// given position. Otherwise, the window will be added to the monitor that contains the center of the window.
    /// If the target monitor has a focalized window or a maximized window, it will be restored.
    fn add(&mut self, win: WindowRef, prefer_position: Option<(i32, i32)>) -> TMResult;

    /// Removes a window from the tile manager.
    fn remove(&mut self, win: WindowRef) -> TMResult;

    fn release(&mut self, window: WindowRef, release: Option<bool>) -> TMResult;
    fn maximize(&mut self, window: WindowRef, maximize: bool) -> TMResult;
    fn focalize(&mut self, window: WindowRef, focalize: Option<bool>) -> TMResult;
    fn half_focalize(&mut self, window: WindowRef, half_focalize: Option<bool>) -> TMResult;
    fn find_neighbour(
        &self,
        window: WindowRef,
        direction: Direction,
        search_strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>>;

    /// Swaps the position of two windows. The new positions are propagated to the inactive
    /// containers if necessary.
    fn swap_windows(&mut self, win1: WindowRef, win2: WindowRef) -> TMResult;
    fn resize(&mut self, window: WindowRef, delta: (i32, i32, i32, i32)) -> TMResult;

    /// Moves a window to the monitor that contains the given position.
    /// If `free_move` is true, the window will be moved to the given position without following
    /// the layout strategy of the tile manager.
    /// If the target monitor is the same as the current monitor, the window will not be moved.
    fn move_window(&mut self, win: WindowRef, point: (i32, i32), free_move: bool) -> TMResult;
}

const C_ERR: Error = Error::ContainerNotFound { refresh: false };

impl TilesManagerInternalOperations for TilesManager {
    fn add(&mut self, win: WindowRef, prefer_position: Option<(i32, i32)>) -> TMResult {
        let tile_state = self.get_window_state(win);
        if tile_state
            .as_ref()
            .is_ok_and(|s| matches!(s, TileState::Floating | TileState::Maximized | TileState::Focalized))
        {
            return Ok(Success::NoChange);
        }

        let center = prefer_position.or_else(|| win.get_area().map(|a| a.get_center()));
        let center = center.ok_or(Error::NoWindow)?;

        let k = self.active_trees.find_at_or_near(center)?.key;
        if k.layer.is_focalized() && tile_state.is_err() {
            self.restore_monitor(&k)?;
            if self.active_trees.find(win).is_ok() {
                return Ok(Success::LayoutChanged);
            }
        }

        if self.active_trees.find(win).is_ok() {
            return Err(Error::WindowAlreadyAdded(win));
        }
        let t = self.active_trees.get_mut(&k).ok_or(C_ERR)?;
        t.insert(win);

        // INFO: if the monitor has a maximized window, restore it
        if let Some(maximized_win) = self.maximized_wins.iter().find(|w| t.has(**w)) {
            self.maximize(*maximized_win, false)?;
        }

        Ok(Success::LayoutChanged)
    }

    fn remove(&mut self, win: WindowRef) -> TMResult {
        let tile_state = self.get_window_state(win)?;

        if matches!(tile_state, WindowTileState::Floating) {
            self.floating_wins.remove(&win);
            return Ok(Success::NoChange);
        }

        let k = self.active_trees.find(win)?.key;
        if matches!(tile_state, WindowTileState::Focalized | WindowTileState::HalfFocalized) {
            self.restore_monitor(&k)?;
        }

        self.active_trees.find_mut(win)?.value.remove(win);

        if matches!(tile_state, WindowTileState::Maximized) {
            self.maximized_wins.remove(&win);
        }

        Ok(Success::LayoutChanged)
    }

    fn release(&mut self, window: WindowRef, release: Option<bool>) -> TMResult {
        let tile_state = self.get_window_state(window)?;

        if matches!(tile_state, WindowTileState::Maximized) {
            return Ok(Success::NoChange);
        }

        if release.unwrap_or(!matches!(tile_state, WindowTileState::Floating)) {
            let monitor_area = self.active_trees.find(window)?.value.get_area();
            self.remove(window)?;
            self.floating_wins.insert(window);
            let area = get_floating_win_area(&monitor_area, &window, &self.config.floating_wins)?;
            let is_topmost = self.config.floating_wins.topmost;
            self.animation_player.queue(window, area, is_topmost);
        } else {
            self.floating_wins.remove(&window);
            self.animation_player.dequeue(window);
            self.add(window, None)?;
            let _ = window.set_topmost(false);
        }

        Ok(Success::LayoutChanged)
    }

    fn maximize(&mut self, window: WindowRef, maximize: bool) -> TMResult {
        let tile_state = self.get_window_state(window)?;

        if matches!(tile_state, WindowTileState::Floating) {
            return Ok(Success::NoChange);
        }

        if maximize {
            let src_e = self.active_trees.find(window)?;
            let win_center = window.get_area().ok_or(Error::NoWindowsInfo)?.get_center();
            let trg_e = self.active_trees.find_at(win_center)?;

            // NOTE: if the window is maximized to another monitor, remove it and add it again
            if src_e.key != trg_e.key {
                self.remove(window)?;
                self.add(window, None)?;
            }

            self.maximized_wins.insert(window);
        } else {
            self.maximized_wins.remove(&window);
        }

        Ok(Success::LayoutChanged)
    }

    fn focalize(&mut self, window: WindowRef, focalize: Option<bool>) -> TMResult {
        let e = self.active_trees.find(window)?;
        let k = e.key;

        if focalize.is_some_and(|f| f)
            || (focalize.is_none() && matches!(k.layer, ContainerLayer::Normal | ContainerLayer::HalfFocalized))
        {
            let wins = e.value.get_ids().to_vec();
            self.activate_monitor_layer(k.monitor.clone(), ContainerLayer::Focalized)?;
            self.active_trees.get_mut(&k).ok_or(C_ERR)?.insert(window);
            wins.iter().filter(|w| **w != window).for_each(|w| {
                w.minimize();
            });
            return Ok(Success::LayoutChanged);
        }

        if focalize.is_some_and(|f| !f) || (focalize.is_none() && matches!(k.layer, ContainerLayer::Focalized)) {
            self.active_trees.get_mut(&k).ok_or(C_ERR)?.clear();
            self.activate_monitor_layer(k.monitor, ContainerLayer::Normal)?;
            return Ok(Success::LayoutChanged);
        }

        Ok(Success::NoChange)
    }

    fn half_focalize(&mut self, window: WindowRef, half_focalize: Option<bool>) -> TMResult {
        let e = self.active_trees.find_mut(window)?;
        let k = e.key;

        if half_focalize.is_some_and(|f| f) || (half_focalize.is_none() && matches!(k.layer, ContainerLayer::Normal)) {
            let mut new_tree = (*e.value).clone();
            let mut leaves = e.value.leaves(0, None);
            if leaves.len() < 2 {
                return Ok(Success::NoChange);
            }

            leaves.sort_by(|a, b| a.viewbox.get_area().cmp(&b.viewbox.get_area()));
            let biggest_leaf = match leaves.pop().ok_or(Error::Generic)? {
                l if l.id == window => leaves.pop().ok_or(Error::Generic)?,
                l => l,
            };

            for l in leaves.iter().filter(|l| l.id != window && l.id != biggest_leaf.id) {
                new_tree.remove(l.id);
                l.id.minimize();
            }

            self.activate_monitor_layer(k.monitor.clone(), ContainerLayer::HalfFocalized)?;
            self.active_trees.insert(k.clone(), new_tree);
            return Ok(Success::LayoutChanged);
        } else if half_focalize.is_some_and(|f| !f)
            || (half_focalize.is_none() && matches!(k.layer, ContainerLayer::HalfFocalized))
        {
            self.active_trees.get_mut(&k).ok_or(Error::Generic)?.clear();
            self.activate_monitor_layer(k.monitor, ContainerLayer::Normal).ok();
            return Ok(Success::LayoutChanged);
        }

        Ok(Success::NoChange)
    }

    fn find_neighbour(
        &self,
        window: WindowRef,
        direction: Direction,
        strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>> {
        let src_entry = self.active_trees.find(window).ok()?;
        if src_entry.key.layer == ContainerLayer::Focalized && matches!(strategy, MonitorSearchStrategy::Same) {
            return None;
        };

        if matches!(strategy, MonitorSearchStrategy::Same | MonitorSearchStrategy::Any) {
            if let Some(l) = self.find_neighbour_same_monitor(window, direction) {
                return Some(l);
            }
        }

        if matches!(strategy, MonitorSearchStrategy::Any) {
            if let Some(l) = self.find_neighbour_closest_monitor(window, direction) {
                return Some(l);
            }
        }

        None
    }

    fn swap_windows(&mut self, w1: WindowRef, w2: WindowRef) -> TMResult {
        let src_k = self.active_trees.find(w1)?.key;
        let trg_k = self.active_trees.find(w2)?.key;

        if src_k == trg_k {
            let t = self.active_trees.get_mut(&src_k).ok_or(C_ERR)?;
            t.swap_ids(w1, w2);
            if src_k.layer.is_focalized() {
                self.inactive_trees.get_normal_mut(&src_k.into())?.swap_ids(w1, w2);
            }
        } else {
            let t = self.active_trees.get_mut(&src_k).ok_or(C_ERR)?;
            t.replace_id(w1, w2);
            if src_k.layer.is_focalized() {
                self.inactive_trees.get_normal_mut(&src_k.into())?.replace_id(w1, w2);
            }

            let t = self.active_trees.get_mut(&trg_k).ok_or(C_ERR)?;
            t.replace_id(w2, w1);
            if trg_k.layer.is_focalized() {
                self.inactive_trees.get_normal_mut(&trg_k.into())?.replace_id(w2, w1);
            }
        };

        Ok(Success::LayoutChanged)
    }

    fn resize(&mut self, win: WindowRef, delta: (i32, i32, i32, i32)) -> TMResult {
        let tile_state = self.get_window_state(win)?;
        if matches!(
            tile_state,
            WindowTileState::Floating | WindowTileState::Maximized | WindowTileState::Focalized
        ) {
            return Ok(Success::NoChange);
        }

        let (resize_w, resize_h) = (delta.2 != 0, delta.3 != 0);
        let (resize_left, resize_up) = (delta.0.abs() > 10, delta.1.abs() > 10);
        let has_w_neigh = match resize_w {
            true => self.find_neighbour(
                win,
                if resize_left { Direction::Left } else { Direction::Right },
                MonitorSearchStrategy::Same,
            ),
            false => None,
        };
        let has_h_neigh = match resize_h {
            true => self.find_neighbour(
                win,
                if resize_up { Direction::Up } else { Direction::Down },
                MonitorSearchStrategy::Same,
            ),
            false => None,
        };

        let t = self.active_trees.find_mut(win)?.value;
        let area = t.find_leaf(win, 0).ok_or(Error::NoWindow)?.viewbox;
        let center = area.get_center();

        const CLAMP_VALUES: Option<(u8, u8)> = Some((10, 90));

        if resize_w && has_w_neigh.is_some() {
            let growth = delta.2;
            let (x, growth) = match resize_left {
                true => (area.get_left_center().0.saturating_sub(20), -growth),
                false => (area.get_right_center().0.saturating_add(20), growth),
            };
            t.resize_ancestor(center, (x, center.1), growth, CLAMP_VALUES);
        }

        if resize_h && has_h_neigh.is_some() {
            let growth = delta.3;
            let (y, growth) = match resize_up {
                true => (area.get_top_center().1.saturating_sub(20), -growth),
                false => (area.get_bottom_center().1.saturating_add(20), growth),
            };
            t.resize_ancestor(center, (center.0, y), growth, CLAMP_VALUES);
        }

        Ok(Success::LayoutChanged)
    }

    fn move_window(&mut self, win: WindowRef, point: (i32, i32), free_move: bool) -> TMResult {
        let tile_state = self.get_window_state(win)?;
        if matches!(tile_state, WindowTileState::Floating | WindowTileState::Maximized) {
            return Ok(Success::NoChange);
        }

        let src_k = self.active_trees.find(win)?.key;
        let trg_k = self.active_trees.find_at(point)?.key;
        if src_k == trg_k {
            return Ok(Success::NoChange);
        }

        self.restore_monitor(&src_k)?;
        self.active_trees.get_mut(&src_k).ok_or(Error::NoWindow)?.remove(win);

        self.restore_monitor(&trg_k)?;
        let trg = self.active_trees.get_mut(&trg_k).ok_or(Error::NoWindow)?;
        match free_move {
            true => trg.insert_at(win, point),
            false => trg.insert(win),
        }

        Ok(Success::LayoutChanged)
    }
}

impl TilesManager {
    fn find_neighbour_same_monitor(&self, window: WindowRef, direction: Direction) -> Option<AreaLeaf<WindowRef>> {
        let src_entry = self.active_trees.find(window).ok()?;
        let src_area = src_entry.value.find_leaf(window, 0)?.viewbox;
        let t = self.active_trees.get(&src_entry.key)?;
        let axis_values = src_area.get_corners(direction)[0];
        let pad = match direction.axis() {
            Orientation::Vertical => (1, 0),
            Orientation::Horizontal => (0, 1),
        };

        let corners = src_area.pad_xy(pad).get_corners(direction);
        let corners = match direction.axis() {
            Orientation::Vertical => (corners[0].0, corners[1].0),
            Orientation::Horizontal => (corners[0].1, corners[1].1),
        };
        self.find_best_matching_leaf(window, direction, &t.leaves(0, None), &corners, &axis_values)
    }

    fn find_neighbour_closest_monitor(&self, window: WindowRef, direction: Direction) -> Option<AreaLeaf<WindowRef>> {
        let src_area = self.active_trees.find_leaf(window).ok()?.viewbox;
        let e = self
            .active_trees
            .find_closest_at(src_area.get_center(), direction)
            .ok()?;

        let corners = e.value.get_area().get_corners(direction.opposite());
        let axis_values = corners[0];
        let corners = match direction.axis() {
            Orientation::Vertical => (corners[0].0, corners[1].0),
            Orientation::Horizontal => (corners[0].1, corners[1].1),
        };
        self.find_best_matching_leaf(window, direction, &e.value.leaves(0, None), &corners, &axis_values)
    }

    fn find_best_matching_leaf(
        &self,
        window: WindowRef,
        direction: Direction,
        leaves: &[AreaLeaf<WindowRef>],
        cross_axis_limits: &(i32, i32),
        axis_values: &(i32, i32),
    ) -> Option<AreaLeaf<WindowRef>> {
        let dir_axis = direction.axis();
        let (lim1, lim2) = *cross_axis_limits;

        let leaves = leaves.iter().filter(|l| l.id != window).filter(|l| match dir_axis {
            Orientation::Horizontal => l.viewbox.overlaps_y(lim1, lim2) && l.viewbox.contains_x(axis_values.0),
            Orientation::Vertical => l.viewbox.overlaps_x(lim1, lim2) && l.viewbox.contains_y(axis_values.1),
        });

        if self.config.history_based_navigation {
            if let Some((l, _)) = leaves
                .clone()
                .filter_map(|l| self.focus_history.value(l.id).map(|i| (l, i)))
                .max_by_key(|&(_, i)| i)
            {
                return Some(*l);
            }
        }

        leaves
            .min_by_key(|l| match direction.axis() {
                Orientation::Horizontal => l.viewbox.y,
                Orientation::Vertical => l.viewbox.x,
            })
            .copied()
    }
}

#[derive(Copy, Clone, Debug)]
pub enum MonitorSearchStrategy {
    Same,
    Any,
}
