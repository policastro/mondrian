use crate::win32utils::api::window::get_foreground_window;
use crate::win32utils::window::window_obj::WindowObj;
use crate::win32utils::window::window_ref::WindowRef;
use crate::win32utils::window::window_snapshot::WindowSnapshot;
use log::warn;
use std::collections::HashMap;
use windows::Win32::Foundation::HWND;

use super::structs::area::Area;
use super::structs::area_tree::layout::layout_strategy::AreaTreeLayoutStrategy;
use super::structs::area_tree::leaf::AreaLeaf;
use super::structs::area_tree::tree::AreaTree;
use super::structs::direction::Direction;
use super::structs::monitors_layout::MonitorsLayout;
use super::structs::orientation::Orientation;

pub struct TilesManager<L: AreaTreeLayoutStrategy + Copy> {
    win_trees: HashMap<isize, AreaTree<isize, L>>,
    monitors_layout: MonitorsLayout,
    managed_wins: HashMap<isize, ManagedWindow>,
}

impl<L: AreaTreeLayoutStrategy + Copy> TilesManager<L> {
    pub fn new(monitors_layout: MonitorsLayout, tiles_layout: L) -> Self {
        let windows_trees: HashMap<isize, AreaTree<isize, L>> = monitors_layout
            .get_monitors()
            .iter()
            .map(|m| (m.id, AreaTree::new(m.workarea, tiles_layout)))
            .collect();

        TilesManager {
            managed_wins: HashMap::new(),
            win_trees: windows_trees,
            monitors_layout,
        }
    }

    pub fn add_then_update(&mut self, window: WindowRef) {
        if self.add(window) {
            self.update();
        }
    }

    pub fn add_all(&mut self, windows: Vec<WindowRef>) {
        let mut to_update = false;
        for w in windows.clone() {
            to_update = self.add(w) || to_update;
        }
        if to_update {
            self.update();
        }
    }

    pub fn add(&mut self, win: WindowRef) -> bool {
        if self.has_window(win.hwnd) {
            return false;
        }

        log::info!("ADDING: {:?}", win.snapshot());

        match win.snapshot() {
            Some(snapshot) => {
                let win_tree = self.get_win_tree_mut(snapshot.viewarea.get_center());
                win_tree.insert(win.hwnd.0);
                self.store_window(snapshot, None);
                return true;
            }
            None => return false,
        }
    }

    pub fn select_near(&mut self, direction: Direction) {
        if self.current().is_none() || self.managed_wins.len() < 2 {
            return;
        }

        let area = self.get_stored_viewarea(HWND(self.current().unwrap()));
        if let Some(area) = area {
            let selection_point = match direction {
                Direction::Left => area.get_left_center().with_offset((-20, -20)),
                Direction::Right => area.get_right_center().with_offset((20, -20)),
                Direction::Up => area.get_top_center().with_offset((-20, 20)),
                Direction::Down => area.get_bottom_center().with_offset((-20, 20)),
            };

            let windows_tree = self.get_win_tree_mut(area.get_center());
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

    pub fn current(&self) -> Option<isize> {
        match get_foreground_window().0 {
            0 => None,
            id => Some(id),
        }
    }

    pub fn remove(&mut self, hwnd: HWND) {
        if let Some(center) = self.get_stored_viewarea(hwnd).map(|a| a.get_center()) {
            log::info!("REMOVING FROM TILES {}", hwnd.0);

            self.get_win_tree_mut(center).remove(center);

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
        let area = self.get_stored_viewarea(win.hwnd).expect("Area not found");
        let center = area.get_center();

        let win_tree = self.get_win_tree_mut(center);
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

    pub fn change_window_position(&mut self, hwnd: HWND, new_point: (u32, u32), orientation: Option<Orientation>) {
        log::info!("CHANGING POSITION {}", hwnd.0);
        if !self.has_window(hwnd) {
            return;
        }
        let area = self.get_stored_viewarea(hwnd).expect("Area not found");
        let center = area.get_center();

        let monitor_id = match self.monitors_layout.which(center) {
            Some(id) => id,
            None => panic!("Monitor not found"),
        };

        let new_monitor_id = match self.monitors_layout.which(new_point) {
            Some(id) => id,
            None => panic!("Monitor not found"),
        };

        if monitor_id != new_monitor_id {
            if let Some(orientation) = orientation {
                self.win_trees.get_mut(&monitor_id).unwrap().remove(center);
                self.win_trees.get_mut(&new_monitor_id).unwrap().insert(hwnd.0);

                self.win_trees
                    .get_mut(&new_monitor_id)
                    .unwrap()
                    .set_parent_orientation(new_point, orientation)
            } else {
                let new_win_tree = self.win_trees.get_mut(&new_monitor_id).unwrap();
                match new_win_tree.replace_id(new_point, hwnd.0) {
                    Some(other_id) => {
                        let win_tree = self.win_trees.get_mut(&monitor_id).unwrap();
                        win_tree.replace_id(center, other_id);
                    }
                    None => {
                        let win_tree = self.win_trees.get_mut(&monitor_id).unwrap();
                        win_tree.remove(center);
                    }
                }
            }
        } else {
            let win_tree = self.win_trees.get_mut(&monitor_id).unwrap();
            if let Some(orientation) = orientation {
                win_tree.remove(center);
                win_tree.insert(hwnd.0);
                win_tree.set_parent_orientation(new_point, orientation)
            } else {
                win_tree.swap_ids(center, new_point);
            }
        }
        self.update();
    }

    pub fn update(&mut self) {
        let ids: Vec<isize> = self.win_trees.keys().cloned().collect();
        for id in ids {
            self.update_monitor(id);
        }
    }

    pub fn update_monitor(&mut self, monitor_id: isize) {
        let win_tree = self
            .win_trees
            .get(&monitor_id)
            .expect("Monitor not found")
            .get_all_leaves();

        let leaves: Vec<AreaLeaf<isize>> = win_tree
            .iter()
            .cloned()
            .filter(|l| self.has_window(HWND(l.id)))
            .collect();

        for leaf in leaves {
            let window_info = WindowRef::from(leaf.id);
            window_info.resize_and_move(leaf.viewbox.get_origin(), leaf.viewbox.get_size());
            self.store_window(window_info.snapshot().unwrap(), Some(leaf));
        }
    }

    pub fn get_windows(&self) -> Vec<WindowRef> {
        self.managed_wins.values().map(|w| w.to_window_info()).collect()
    }

    pub fn focus_window(&mut self, id: isize) {
        if self.current() != Some(id) {
            WindowRef::new(HWND(id)).focus()
        }
    }

    fn get_stored_viewarea(&self, hwnd: HWND) -> Option<Area> {
        self.managed_wins.get(&hwnd.0).map(|w| w.viewarea)
    }

    pub fn has_window(&self, hwnd: HWND) -> bool {
        self.managed_wins.contains_key(&hwnd.0)
    }

    fn store_window(&mut self, snapshot: WindowSnapshot, leaf: Option<AreaLeaf<isize>>) {
        let mut win = ManagedWindow::from(snapshot);
        win.leaf = leaf;
        self.managed_wins.insert(win.id.0, win);
    }

    fn get_win_tree_mut(&mut self, point: (u32, u32)) -> &mut AreaTree<isize, L> {
        let monitor_id = self.monitors_layout.which(point).expect("Monitor not found");
        self.win_trees.get_mut(&monitor_id).expect("No monitor found")
    }
}

trait Point {
    fn with_offset(&self, offset: (i32, i32)) -> (u32, u32);
}

impl Point for (u32, u32) {
    fn with_offset(&self, offset: (i32, i32)) -> (u32, u32) {
        let x = match offset.0 < 0 {
            true => self.0.saturating_sub(offset.0.unsigned_abs()),
            false => self.0.saturating_add(offset.0 as u32),
        };

        let y = match offset.1 < 0 {
            true => self.1.saturating_sub(offset.1.unsigned_abs()),
            false => self.1.saturating_add(offset.1 as u32),
        };

        (x, y)
    }
}

impl From<[i32; 4]> for Area {
    fn from(array: [i32; 4]) -> Self {
        Area::from_array(array.map(|i| i.try_into().unwrap_or(0)))
    }
}

#[derive(Debug, Clone)]
struct ManagedWindow {
    id: HWND,
    viewarea: Area,
    leaf: Option<AreaLeaf<isize>>,
}

impl From<WindowSnapshot> for ManagedWindow {
    fn from(snapshot: WindowSnapshot) -> Self {
        ManagedWindow::new(snapshot.hwnd, snapshot.viewarea, None)
    }
}

impl ManagedWindow {
    pub fn new(id: HWND, viewarea: Area, leaf: Option<AreaLeaf<isize>>) -> Self {
        ManagedWindow { id, viewarea, leaf }
    }

    pub fn to_window_info(&self) -> WindowRef {
        WindowRef { hwnd: self.id }
    }
}
