use crate::app::structs::area::Area;
use crate::app::structs::area_tree::leaf::AreaLeaf;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
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
        let cointainers: Vec<Container> = monitors_layout
            .into_iter()
            .map(|m| Container::new(m.monitor, m.layout))
            .collect();

        let containers_manager = ContainersManager::new(cointainers);

        TilesManager {
            managed_wins: HashMap::new(),
            containers: containers_manager,
            config: config.unwrap_or_default(),
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

    pub fn select_near(&mut self, direction: Direction) {
        if self.current().is_none() || self.managed_wins.len() < 2 {
            return;
        }

        let area = self.get_stored_area(HWND(self.current().unwrap()));
        if let Some(area) = area {
            let selection_point = match direction {
                Direction::Left => area.get_left_center().with_offset((-20, -20)),
                Direction::Right => area.get_right_center().with_offset((20, -20)),
                Direction::Up => area.get_top_center().with_offset((-20, 20)),
                Direction::Down => area.get_bottom_center().with_offset((-20, 20)),
            };

            let windows_tree = match self.containers.which_tree(area.get_center()) {
                Some(tree) => tree,
                None => return,
            };

            if let Some(leaf) = windows_tree.find_leaf(selection_point) {
                self.focus_window(leaf.id);
            }
        }
    }

    pub fn select_next(&mut self) {
        if self.current().is_none() || self.managed_wins.len() < 2 {
            return;
        }

        let prev_current = self.current().unwrap();
        let mut directions = vec![Direction::Left, Direction::Right, Direction::Up, Direction::Down];

        while prev_current == self.current().unwrap() && !directions.is_empty() {
            self.select_near(directions.pop().unwrap());
        }
    }

    fn current(&self) -> Option<isize> {
        match get_foreground_window().0 {
            0 => None,
            id => Some(id),
        }
    }

    pub fn remove(&mut self, hwnd: HWND) {
        if let Some(center) = self.get_stored_area(hwnd).map(|a| a.get_center()) {
            if let Some(win_tree) = self.containers.which_tree(center) {
                win_tree.remove(center);
            }

            self.managed_wins.remove(&hwnd.0);

            if self.current() == Some(hwnd.0) {
                self.select_next();
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

    pub fn change_window_position(&mut self, hwnd: HWND, new_point: (i32, i32), orientation: Option<Orientation>) {
        if !self.has_window(hwnd) {
            return;
        }

        let area = self.get_stored_area(hwnd).expect("Area not found");
        let center = area.get_center();

        if !self.containers.is_same_container(center, new_point) {
            if let Some(orientation) = orientation {
                match self.containers.which_tree(center) {
                    Some(tree) => tree.remove(center),
                    None => panic!("Container not found"),
                }

                match self.containers.which_tree(new_point) {
                    Some(tree) => {
                        tree.insert(hwnd.0);
                        tree.set_parent_orientation(new_point, orientation)
                    }
                    None => panic!("Container not found"),
                }
            } else {
                let replaced_id = self
                    .containers
                    .which_tree(new_point)
                    .expect("Container not found")
                    .replace_id(new_point, hwnd.0);

                let container = self.containers.which_tree(center).expect("Container not found");
                if let Some(other_id) = replaced_id {
                    container.replace_id(center, other_id);
                } else {
                    container.remove(center);
                }
            }
        } else {
            let container = self.containers.which_mut(center).expect("Container not found");
            container.tree.swap_ids(center, new_point);
            if let Some(orientation) = orientation {
                container.tree.set_parent_orientation(new_point, orientation);
            }
        }

        self.update();
    }

    pub fn update(&mut self) {
        let leaves_vec: Vec<(Area, Vec<AreaLeaf<isize>>)> = self
            .containers
            .get_containers()
            .iter()
            .map(|c| (c.monitor.workarea, c.tree.get_all_leaves()))
            .collect();
        leaves_vec.into_iter().for_each(|l| self.update_leaves(l.1, l.0));
    }
    pub fn update_leaves(&mut self, leaves: Vec<AreaLeaf<isize>>, workarea: Area) {
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
            let padding = self.get_padding(leaf.viewbox, workarea);
            let area = leaf.viewbox.pad(Some(padding.0), Some(padding.1));
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
            self.update_leaves(leaves, workarea);
        }
    }

    pub fn focus_window(&mut self, id: isize) {
        if self.current() != Some(id) {
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

    fn get_padding(&self, area: Area, workarea: Area) -> ((i8, i8), (i8, i8)) {
        let ipad = self.config.tiles_padding - 4; // TODO Note: 4 is a magic number
        let epad = self.config.border_padding - 4; // TODO Note: 4 is a magic number

        let x = if area.x == workarea.x { epad } else { ipad };
        let y = if area.y == workarea.y { epad } else { ipad };
        let w = match area.x + i32::from(area.width) == i32::from(workarea.width) {
            true => epad,
            false => ipad,
        };
        let h = match area.y + i32::from(area.height) == i32::from(workarea.height) {
            true => epad,
            false => ipad,
        };

        ((x, w), (y, h))
    }
}

trait Point {
    fn with_offset(&self, offset: (i32, i32)) -> (i32, i32);
}

impl Point for (i32, i32) {
    fn with_offset(&self, offset: (i32, i32)) -> (i32, i32) {
        let x = match offset.0 < 0 {
            true => self.0.saturating_sub(offset.0),
            false => self.0.saturating_add(offset.0),
        };

        let y = match offset.1 < 0 {
            true => self.1.saturating_sub(offset.1),
            false => self.1.saturating_add(offset.1),
        };

        (x, y)
    }
}
