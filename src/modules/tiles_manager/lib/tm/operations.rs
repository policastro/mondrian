use super::error::TilesManagerError;
use super::manager::ContainerKey;
use super::manager::TilesManager;
use super::manager::TilesManagerBase;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::mondrian_message::WindowTileState;
use crate::app::structs::direction::Direction;
use crate::app::structs::point::Point;
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
    fn find_next_neighbour(
        &self,
        window: WindowRef,
        search_strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>>; // Focalized
    fn find_neighbour(
        &self,
        window: WindowRef,
        direction: Direction,
        search_strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>>; // Focalized
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

    fn remove(&mut self, win: WindowRef, skip_focalized: bool) -> Result<bool, Error> {
        let tile_state = self.get_window_state(win).ok_or(Error::NoWindow)?;
        if matches!(tile_state, WindowTileState::Floating) {
            return Ok(false);
        }

        let k = self.active_trees.find(win).ok_or(Error::NoWindow)?.key;
        if matches!(tile_state, WindowTileState::Focalized) {
            self.focalized_wins.remove(&k);
        }

        if skip_focalized && self.focalized_wins.contains_key(&k) {
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
            self.remove(window, false)?;
            self.floating_wins.insert(window);
            let _ = window.set_topmost(true);
        } else {
            self.floating_wins.remove(&window);
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

        match maximize {
            true => self.maximized_wins.insert(window),
            false => self.maximized_wins.remove(&window),
        };

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

    fn find_next_neighbour(&self, window: WindowRef, strategy: MonitorSearchStrategy) -> Option<AreaLeaf<WindowRef>> {
        [Direction::Right, Direction::Down, Direction::Left, Direction::Up]
            .iter()
            .find_map(|d| self.find_neighbour(window, *d, strategy))
    }

    fn find_neighbour(
        &self,
        window: WindowRef,
        direction: Direction,
        strategy: MonitorSearchStrategy,
    ) -> Option<AreaLeaf<WindowRef>> {
        let src_entry = self.active_trees.find(window)?;

        if self.focalized_wins.contains_key(&src_entry.key) {
            match strategy {
                MonitorSearchStrategy::Same => return None,
                MonitorSearchStrategy::Any => {
                    return self.find_neighbour(window, direction, MonitorSearchStrategy::Closest)
                }
                _ => (),
            }
        };

        let src_area = src_entry.value.find_leaf(window, 0)?.viewbox;
        let point = match direction {
            Direction::Right => src_area.get_ne_corner().with_offset(1, 1), // INFO: prefer up
            Direction::Down => src_area.get_se_corner().with_offset(-1, 1), // INFO: prefer right
            Direction::Left => src_area.get_sw_corner().with_offset(-1, -1), // INFO: prefer down
            Direction::Up => src_area.get_nw_corner().with_offset(1, -1),   // INFO: prefer left
        };

        let cs = &self.active_trees;
        if matches!(strategy, MonitorSearchStrategy::Any) {
            if let Some(e) = cs.find_at(point) {
                return match self.focalized_wins.get(&e.key) {
                    Some(fw) => e.value.find_leaf(*fw, 0).filter(|w| w.id != window),
                    None => e.value.find_leaf_at(point, 0).filter(|w| w.id != window),
                };
            } else {
                return self.find_neighbour(window, direction, MonitorSearchStrategy::Closest);
            }
        }

        if matches!(strategy, MonitorSearchStrategy::Same) {
            return src_entry.value.find_leaf_at(point, 0).filter(|w| w.id != window);
        };

        if matches!(strategy, MonitorSearchStrategy::Closest) {
            let e = cs.find_closest_at(src_area.get_center(), direction)?;
            let point = match direction {
                Direction::Right => e.value.area.get_nw_corner(),
                Direction::Down => e.value.area.get_ne_corner(),
                Direction::Left => e.value.area.get_se_corner(),
                Direction::Up => e.value.area.get_sw_corner(),
            };

            return match self.focalized_wins.get(&e.key) {
                Some(fw) => e.value.find_leaf(*fw, 0).filter(|w| w.id != window),
                None => e.value.find_leaf_at(point, 0).filter(|w| w.id != window),
            };
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
        if matches!(tile_state, WindowTileState::Floating | WindowTileState::Maximized) {
            return Ok(());
        }

        if matches!(tile_state, WindowTileState::Focalized) {
            return self.update_layout(true);
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
