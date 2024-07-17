use crate::app::structs::area::Area;
use crate::app::structs::area_tree::leaf::AreaLeaf;
use crate::app::structs::direction::Direction;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::{WindowObjHandler, WindowObjInfo};
use crate::win32::window::window_ref::WindowRef;
use crate::win32::window::window_snapshot::WindowSnapshot;
use ::windows::Win32::Foundation::HWND;
use log::warn;
use std::collections::HashMap;

use super::config::TilesManagerConfig;
use super::container::Container;
use super::containers_manager::ContainersManager;
use super::managed_window::ManagedWindow;
use super::monitor_layout::MonitorLayout;

pub struct TilesManager {
    containers: ContainersManager,
    managed_wins: HashMap<isize, ManagedWindow>,
    config: TilesManagerConfig,
}

impl TilesManager {
    /// Creates a new [`TilesManager`].
    pub fn new(monitors_layout: Vec<MonitorLayout>, config: Option<TilesManagerConfig>) -> Self {
        let config = config.unwrap_or_default();

        let cointainers: Vec<Container> = monitors_layout
            .into_iter()
            .map(|m| Container::new(m.monitor, m.layout, config.get_border_pad()))
            .collect();

        let containers_manager = ContainersManager::new(cointainers);

        TilesManager {
            managed_wins: HashMap::new(),
            containers: containers_manager,
            config,
        }
    }

    pub fn add(&mut self, win: WindowRef, update: bool) -> bool {
        if self.has_window(win.hwnd) {
            return false;
        }

        let snapshot = match win.snapshot() {
            Some(snapshot) => snapshot,
            None => {
                log::warn!("Could not get snapshot for window: {:?}", win);
                return false;
            }
        };

        let tree = match self
            .containers
            .which_tree(snapshot.viewarea.expect("Could not get area").get_center())
        {
            Some(tree) => tree,
            None => {
                warn!("Could not get tree for window: {:?}", win);
                return false;
            }
        };

        tree.insert(win.hwnd.0);
        self.store_window(snapshot, None);

        if update {
            self.update();
        }

        true
    }

    pub fn find_at(&mut self, direction: Direction) -> Option<AreaLeaf<isize>> {
        let current = get_foreground_window()?;
        if !self.has_window(current) || self.managed_wins.len() < 2 {
            return None;
        }

        let padxy = self.config.get_tile_pad_xy();
        let area = WindowRef::new(current).get_window_box()?.pad_xy((-padxy.0, -padxy.1));

        let selection_point = match direction {
            Direction::Right => area.get_ne_corner().with_offset(20, 5), // TODO Prefer up
            Direction::Down => area.get_se_corner().with_offset(-5, 20), // TODO Prefer left
            Direction::Left => area.get_sw_corner().with_offset(-20, -5), // TODO Prefer up
            Direction::Up => area.get_nw_corner().with_offset(5, -20),   // TODO Prefer left
        };

        if let Some(tree) = self.containers.which_tree(selection_point) {
            // If the point is in a tree
            return tree.find_leaf(selection_point, self.config.get_border_pad());
        } else if let Some(container) = self.containers.which_nearest_mut(area.get_center(), direction) {
            // Otherwise, find the nearest container
            let area = container.workarea;
            let selection_point = match direction {
                Direction::Right => area.get_nw_corner(),
                Direction::Down => area.get_ne_corner(),
                Direction::Left => area.get_se_corner(),
                Direction::Up => area.get_sw_corner(),
            };
            return container.tree.find_leaf(selection_point, self.config.get_border_pad());
        }

        None
    }

    pub fn focus_at(&mut self, direction: Direction) {
        if let Some(leaf) = self.find_at(direction) {
            self.focus_window(leaf.id);
        }
    }

    pub fn focus_next(&mut self) {
        let current = match get_foreground_window() {
            Some(hwnd) => hwnd,
            None => return,
        };

        if !self.has_window(current) || self.managed_wins.len() < 2 {
            return;
        }

        let mut directions = vec![Direction::Right, Direction::Down, Direction::Left, Direction::Up];

        let mut leaf = None;
        while leaf.is_none() && !directions.is_empty() {
            leaf = self.find_at(directions.pop().unwrap());
        }

        if let Some(leaf) = leaf {
            self.focus_window(leaf.id);
        }
    }

    pub fn remove(&mut self, hwnd: HWND) {
        if let Some(center) = self.get_stored_area(hwnd).map(|a| a.get_center()) {
            if let Some(win_tree) = self.containers.which_tree(center) {
                win_tree.remove(center);
            }

            self.managed_wins.remove(&hwnd.0);

            if get_foreground_window().is_none() {
                self.focus_next();
            }

            self.update();
        }
    }

    pub fn refresh_window_size(&mut self, hwnd: HWND) {
        if !self.has_window(hwnd) {
            warn!("Window not found: {}", hwnd.0);
            return;
        }

        let win = WindowRef::new(hwnd);
        let area = self.get_stored_area(win.hwnd).expect("Area not found");
        let center = area.get_center();

        let win_tree = match self.containers.which_tree(center) {
            Some(win_tree) => win_tree,
            None => return,
        };

        let area_shift = win.get_window_box().expect("Snapshot not found").get_shift(&area);

        if area_shift.2 != 0 {
            let growth = (area_shift.2 as f32 / area.width as f32) * 100f32;
            let (x, growth) = match area_shift.0.abs() > 10 {
                true => (area.get_left_center().0.saturating_sub(20), -growth),
                false => (area.get_right_center().0.saturating_add(20), growth),
            };

            win_tree.resize_ancestor(center, (x, center.1), growth as i32);
        }

        if area_shift.3 != 0 {
            let growth = (area_shift.3 as f32 / area.height as f32) * 100f32;
            let (y, grow_perc) = match area_shift.1.abs() > 10 {
                true => (area.get_top_center().1.saturating_sub(20), -growth),
                false => (area.get_bottom_center().1.saturating_add(20), growth),
            };

            win_tree.resize_ancestor(center, (center.0, y), grow_perc as i32);
        }
        self.update();
    }

    pub fn move_window(&mut self, hwnd: HWND, new_point: (i32, i32), invert_monitor_op: bool, switch_orient: bool) {
        if !self.has_window(hwnd) {
            return;
        }

        let area = self.get_stored_area(hwnd).expect("Area not found");
        let center = area.get_center();

        let containers = &mut self.containers;
        if containers.is_same_container(center, new_point) {
            // If it is in the same monitor
            let tree = containers.which_tree(center).expect("Container not found");
            tree.swap_ids(center, new_point);
            if switch_orient {
                tree.switch_subtree_orientations(new_point);
            }
        } else if self.config.is_insert_in_monitor(invert_monitor_op) || switch_orient {
            // If it is in another monitor and insert
            match containers.which_tree(center) {
                Some(tree) => tree.remove(center),
                None => panic!("Container not found"),
            }

            let tree = containers.which_tree(new_point).expect("Container not found");
            tree.insert(hwnd.0);
            if switch_orient {
                tree.switch_subtree_orientations(new_point);
            }
        } else {
            // If it is in another monitor and swap
            let replaced_id = containers
                .which_tree(new_point)
                .map(|t| {
                    let id = t.replace_id(new_point, hwnd.0);
                    if switch_orient {
                        t.switch_subtree_orientations(new_point);
                    }
                    id
                })
                .expect("Container not found");

            let tree = containers.which_tree(center).expect("Container not found");
            if let Some(id) = replaced_id {
                tree.replace_id(center, id);
            } else {
                tree.remove(center);
            }
        }

        self.update();
    }

    pub fn update(&mut self) {
        let leaves_vec: Vec<Vec<AreaLeaf<isize>>> = self
            .containers
            .get_containers()
            .iter()
            .map(|l| l.tree.get_all_leaves(self.config.get_border_pad()))
            .collect();
        leaves_vec.into_iter().for_each(|l| self.update_leaves(l));
    }

    pub fn update_leaves(&mut self, leaves: Vec<AreaLeaf<isize>>) {
        let managed_wins = self.managed_wins.clone();

        managed_wins
            .into_iter()
            .map(|w| WindowRef::from(w.0))
            .filter(|wr| wr.get_exe_name().is_none() || !wr.is_window())
            .for_each(|wr| self.remove(wr.hwnd));

        let leaves: Vec<AreaLeaf<isize>> = leaves.into_iter().filter(|l| self.has_window(HWND(l.id))).collect();

        let mut errors = 0;
        leaves.clone().into_iter().for_each(|leaf| {
            let win_ref = WindowRef::from(leaf.id);
            let area = leaf.viewbox.pad_xy(self.config.get_tile_pad_xy());
            let res = win_ref.resize_and_move(area.get_origin(), area.get_size());

            if res.is_err() {
                warn!("Failed to resize and move window: {}", win_ref.hwnd.0);
                errors += 1;
            };

            if let Some(snapshot) = win_ref.snapshot() {
                self.store_window(snapshot, Some(leaf));
            }
        });

        if errors > 0 {
            self.update_leaves(leaves);
        }
    }

    pub fn focus_window(&mut self, id: isize) {
        if !get_foreground_window().is_some_and(|hwnd| hwnd.0 == id) {
            WindowRef::new(HWND(id)).focus()
        }
    }

    pub fn has_window(&self, hwnd: HWND) -> bool {
        self.managed_wins.contains_key(&hwnd.0)
    }

    fn get_stored_area(&self, hwnd: HWND) -> Option<Area> {
        self.managed_wins.get(&hwnd.0).map(|w| w.viewarea)
    }

    fn store_window(&mut self, snapshot: WindowSnapshot, leaf: Option<AreaLeaf<isize>>) {
        let mut win = ManagedWindow::from(snapshot);
        win.leaf = leaf;
        self.managed_wins.insert(win.id.0, win);
    }
}

trait Point {
    fn with_offset(&self, offset_x: i32, offset_y: i32) -> (i32, i32);
}

impl Point for (i32, i32) {
    fn with_offset(&self, offset_x: i32, offset_y: i32) -> (i32, i32) {
        (self.0.saturating_add(offset_x), self.1.saturating_add(offset_y))
    }
}
