use super::error::TilesManagerError;
use super::manager::ContainerKey;
use super::manager::TilesManager;
use super::manager::TilesManagerBase;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::configs::general::FloatingWinsSizeStrategy;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::area::Area;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use crate::modules::tiles_manager::lib::containers::Containers;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;

type Error = TilesManagerError;

pub trait TilesManagerInternalOperations: TilesManagerBase {
    fn add(&mut self, win: WindowRef) -> Result<(), Error>;
    fn remove(&mut self, win: WindowRef, skip_focalized: bool) -> Result<bool, Error>;
    fn release(&mut self, window: WindowRef, release: Option<bool>) -> Result<(), Error>;
    fn maximize(&mut self, window: WindowRef, maximize: bool) -> Result<(), Error>;
    fn focalize(&mut self, window: WindowRef, focalize: Option<bool>) -> Result<(), Error>;
    fn find_neighbour(
        &self,
        window: WindowRef,
        direction: Direction,
        search_strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>>;
    fn swap_windows(&mut self, win1: WindowRef, win2: WindowRef) -> Result<(), Error>;
    fn resize(&mut self, window: WindowRef, delta: (i32, i32, i32, i32)) -> Result<(), Error>;
    fn move_to(&mut self, win: WindowRef, point: (i32, i32), free_move: bool) -> Result<(), Error>;
}

impl TilesManagerInternalOperations for TilesManager {
    fn add(&mut self, win: WindowRef) -> Result<(), Error> {
        let tile_state = self.get_window_state(win);
        if tile_state.is_some_and(|s| matches!(s, WindowTileState::Floating | WindowTileState::Maximized)) {
            return Ok(());
        }

        if self.active_trees.find(win).is_some() {
            return Err(Error::WindowAlreadyAdded);
        }

        let center = win.get_area().map(|a| a.get_center());
        let center = center.ok_or(Error::NoWindowsInfo)?;
        let c = self.active_trees.find_at_or_near_mut(center);
        let c = c.ok_or(Error::NoWindowsInfo)?;
        self.focalized_wins.remove(&c.key);

        c.value.insert(win);

        // INFO: if the monitor has a maximized window, restore it
        if let Some(maximized_win) = self.maximized_wins.iter().find(|w| c.value.has(**w)) {
            self.maximize(*maximized_win, false)?;
        }

        Ok(())
    }

    fn remove(&mut self, win: WindowRef, skip_focalized_monitors: bool) -> Result<bool, Error> {
        let tile_state = self.get_window_state(win).ok_or(Error::NoWindow)?;
        if matches!(tile_state, WindowTileState::Floating) {
            return Ok(false);
        }

        let k = self.active_trees.find(win).ok_or(Error::NoWindow)?.key;
        if matches!(tile_state, WindowTileState::Focalized) {
            self.focalized_wins.remove(&k);
        }

        if skip_focalized_monitors && self.focalized_wins.contains_key(&k) {
            return Ok(false);
        }

        let t = self.active_trees.find_mut(win).ok_or(Error::NoWindow)?.value;
        t.remove(win);

        if matches!(tile_state, WindowTileState::Maximized) {
            self.maximized_wins.remove(&win);
        }

        Ok(true)
    }

    fn release(&mut self, window: WindowRef, release: Option<bool>) -> Result<(), Error> {
        let tile_state = self.get_window_state(window).ok_or(Error::NoWindow)?;

        if matches!(tile_state, WindowTileState::Maximized) {
            return Ok(());
        }

        if release.unwrap_or(!matches!(tile_state, WindowTileState::Floating)) {
            let monitor_area = self.active_trees.find(window).ok_or(Error::NoWindow)?.value.get_area();
            let (monitor_x, monitor_y) = monitor_area.get_center();
            let (monitor_w, monitor_h) = monitor_area.get_size();
            self.remove(window, false)?;
            self.floating_wins.insert(window);
            let is_topmost = self.config.floating_wins.topmost;
            let _ = window.set_topmost(is_topmost);
            let area = match self.config.floating_wins.size {
                FloatingWinsSizeStrategy::Preserve => {
                    let a = window.get_area().ok_or(Error::NoWindowsInfo)?;
                    let (x, y) = (monitor_x - (a.width as i32 / 2), monitor_y - (a.height as i32 / 2));
                    Area::new(x, y, a.width, a.height)
                }
                FloatingWinsSizeStrategy::Fixed => {
                    let (w, h) = self.config.floating_wins.size_fixed;
                    let (w, h) = (w.min(monitor_w), h.min(monitor_h));
                    let (x, y) = (monitor_x - (w as i32 / 2), monitor_y - (h as i32 / 2));
                    Area::new(x, y, w, h)
                }
                FloatingWinsSizeStrategy::Relative => {
                    let (rw, rh) = self.config.floating_wins.size_ratio;
                    let (w, h) = (
                        (monitor_w as f32 * rw).round() as u16,
                        (monitor_h as f32 * rh).round() as u16,
                    );
                    let (x, y) = (monitor_x - (w as i32 / 2), monitor_y - (h as i32 / 2));
                    Area::new(x, y, w, h)
                }
            };
            self.animation_player.queue(window, area, is_topmost);
        } else {
            self.floating_wins.remove(&window);
            self.animation_player.dequeue(window);
            self.add(window)?;
            let _ = window.set_topmost(false);
        }

        Ok(())
    }

    fn maximize(&mut self, window: WindowRef, maximize: bool) -> Result<(), Error> {
        let tile_state = self.get_window_state(window).ok_or(Error::NoWindow)?;

        if matches!(tile_state, WindowTileState::Floating) {
            return Ok(());
        }

        if maximize {
            let src_e = self.active_trees.find(window).ok_or(Error::NoWindow)?;
            let trg_e = self
                .active_trees
                .find_at(window.get_area().ok_or(Error::NoWindowsInfo)?.get_center())
                .ok_or(Error::ContainerNotFound { refresh: false })?;

            if src_e.key != trg_e.key {
                self.remove(window, false)?;
                self.add(window)?;
            }

            self.maximized_wins.insert(window);
        } else {
            self.maximized_wins.remove(&window);
        }

        Ok(())
    }

    fn focalize(&mut self, window: WindowRef, focalize: Option<bool>) -> Result<(), Error> {
        let k = self.active_trees.find(window).ok_or(Error::NoWindow)?.key;

        if focalize.is_some_and(|f| f) {
            self.focalized_wins.insert(k.clone(), window);
            let _ = self.release(window, Some(false));
            return Ok(());
        }

        if focalize.is_some_and(|f| !f) {
            if self.focalized_wins.get(&k).is_some_and(|w| *w == window) {
                self.focalized_wins.remove(&k);
            }
            return Ok(());
        }

        if let std::collections::hash_map::Entry::Vacant(e) = self.focalized_wins.entry(k.clone()) {
            e.insert(window);
            let _ = self.release(window, Some(false));
        } else {
            self.focalized_wins.remove(&k);
        }

        Ok(())
    }

    fn find_neighbour(
        &self,
        window: WindowRef,
        direction: Direction,
        strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>> {
        let src_entry = self.active_trees.find(window)?;
        let is_monitor_focalized = self.focalized_wins.contains_key(&src_entry.key);

        if is_monitor_focalized && matches!(strategy, MonitorSearchStrategy::Same) {
            return None;
        };

        let strategy = match is_monitor_focalized {
            true => MonitorSearchStrategy::Closest,
            false => strategy,
        };

        if matches!(strategy, MonitorSearchStrategy::Same | MonitorSearchStrategy::Any) {
            if let Some(l) = self.find_neighbour_same_monitor(window, direction) {
                return Some(l);
            }
        }

        if matches!(strategy, MonitorSearchStrategy::Closest | MonitorSearchStrategy::Any) {
            if let Some(l) = self.find_neighbour_closest_monitor(window, direction) {
                return Some(l);
            }
        }

        None
    }

    fn swap_windows(&mut self, w1: WindowRef, w2: WindowRef) -> Result<(), Error> {
        let src_k = self.active_trees.find(w1).ok_or(Error::NoWindow)?.key;
        let trg_k = self.active_trees.find(w2).ok_or(Error::NoWindow)?.key;

        if src_k == trg_k {
            let t = self.active_trees.get_mut(&src_k).ok_or(Error::Generic)?;
            t.swap_ids(w1, w2);
            if self.focalized_wins.matches(&src_k, w1) {
                self.focalized_wins.insert(src_k, w2);
            }
        } else {
            let t = self.active_trees.get_mut(&src_k).ok_or(Error::Generic)?;
            t.replace_id(w1, w2);
            if self.focalized_wins.matches(&src_k, w1) {
                self.focalized_wins.insert(src_k, w2);
            }

            let t = self.active_trees.get_mut(&trg_k).ok_or(Error::Generic)?;
            t.replace_id(w2, w1);
            if self.focalized_wins.matches(&trg_k, w2) {
                self.focalized_wins.insert(trg_k, w1);
            }
        };

        Ok(())
    }

    fn resize(&mut self, win: WindowRef, delta: (i32, i32, i32, i32)) -> Result<(), Error> {
        let tile_state = self.get_window_state(win).ok_or(Error::NoWindow)?;
        if matches!(
            tile_state,
            WindowTileState::Floating | WindowTileState::Maximized | WindowTileState::Focalized
        ) {
            return Ok(());
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

        let t = self.active_trees.find_mut(win).ok_or(Error::NoWindow)?.value;
        let area = t.find_leaf(win, 0).ok_or(Error::Generic)?.viewbox;
        let center = area.get_center();

        let clamp_values = Some((10, 90));

        if resize_w && has_w_neigh.is_some() {
            let growth = delta.2;
            let (x, growth) = match resize_left {
                true => (area.get_left_center().0.saturating_sub(20), -growth),
                false => (area.get_right_center().0.saturating_add(20), growth),
            };
            t.resize_ancestor(center, (x, center.1), growth, clamp_values);
        }

        if resize_h && has_h_neigh.is_some() {
            let growth = delta.3;
            let (y, growth) = match resize_up {
                true => (area.get_top_center().1.saturating_sub(20), -growth),
                false => (area.get_bottom_center().1.saturating_add(20), growth),
            };
            t.resize_ancestor(center, (center.0, y), growth, clamp_values);
        }

        Ok(())
    }

    fn move_to(&mut self, win: WindowRef, point: (i32, i32), free_move: bool) -> Result<(), Error> {
        let tile_state = self.get_window_state(win).ok_or(Error::NoWindow)?;
        if matches!(tile_state, WindowTileState::Floating | WindowTileState::Maximized) {
            return Ok(());
        }

        let src_k = self.active_trees.find(win).ok_or(Error::NoWindow)?.key;
        let trg_k = self.active_trees.find_at(point).ok_or(Error::NoWindow)?.key;
        if src_k == trg_k {
            return Ok(());
        }

        self.active_trees.get_mut(&src_k).ok_or(Error::NoWindow)?.remove(win);
        if self.focalized_wins.matches(&src_k, win) {
            self.focalized_wins.remove(&src_k);
        }

        let trg = self.active_trees.get_mut(&trg_k).ok_or(Error::NoWindow)?;
        match free_move {
            true => trg.insert_at(win, point),
            false => trg.insert(win),
        }
        if self.focalized_wins.contains_key(&trg_k) {
            self.focalized_wins.remove(&trg_k);
        }

        Ok(())
    }
}

impl TilesManager {
    fn find_neighbour_same_monitor(&self, window: WindowRef, direction: Direction) -> Option<AreaLeaf<WindowRef>> {
        let src_entry = self.active_trees.find(window)?;
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
        let src_entry = self.active_trees.find(window)?;
        let src_area = src_entry.value.find_leaf(window, 0)?.viewbox;
        let e = self.active_trees.find_closest_at(src_area.get_center(), direction)?;
        if let Some(fw) = self.focalized_wins.get(&e.key) {
            return e.value.find_leaf(*fw, 0).filter(|w| w.id != window);
        };

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
    Closest,
}

pub trait FocalizedMap {
    fn matches(&self, key: &ContainerKey, window: WindowRef) -> bool;
}

impl FocalizedMap for HashMap<ContainerKey, WindowRef> {
    fn matches(&self, key: &ContainerKey, window: WindowRef) -> bool {
        self.get(key).is_some_and(|w| *w == window)
    }
}
