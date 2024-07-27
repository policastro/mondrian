use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::app::structs::{
    area::Area,
    area_tree::{leaf::AreaLeaf, tree::WinTree},
    direction::Direction,
};

use super::container::Container;

pub(super) struct ContainersManager<T: Clone + Copy + Eq + Hash> {
    containers: HashMap<T, Container>,
}

impl<T: Clone + Copy + Eq + Hash> ContainersManager<T> {
    pub fn new(containers: HashMap<T, Container>) -> Self {
        ContainersManager { containers }
    }

    pub fn is_same_container(&self, point1: (i32, i32), point2: (i32, i32)) -> bool {
        let id1 = self.which_id(point1);
        let id2 = self.which_id(point2);

        match (id1, id2) {
            (Some(id1), Some(id2)) => id1 == id2,
            _ => false,
        }
    }

    pub fn which_id(&self, point: (i32, i32)) -> Option<T> {
        self.containers
            .iter()
            .find(|(_k, c)| c.contains(point))
            .map(|(k, _)| *k)
    }

    pub fn which_at_mut(&mut self, point: (i32, i32)) -> Option<&mut Container> {
        self.containers.values_mut().find(|c| c.contains(point))
    }

    pub fn which_mut(&mut self, hwnd: isize) -> Option<&mut Container> {
        self.containers.values_mut().find(|c| (*c).tree.has_id(hwnd))
    }

    pub fn which_tree_at_mut(&mut self, point: (i32, i32)) -> Option<&mut WinTree> {
        self.which_at_mut(point).map(|c| &mut c.tree)
    }

    pub fn which_tree_mut(&mut self, hwnd: isize) -> Option<&mut WinTree> {
        self.which_mut(hwnd).map(|c| &mut c.tree)
    }

    pub fn which_nearest_mut(&mut self, ref_point: (i32, i32), direction: Direction) -> Option<&mut Container> {
        let ref_id = self.which_id(ref_point)?;
        let ref_c = self.which_by_id(ref_id)?;

        let point = match direction {
            Direction::Right => ref_c.orig_workarea.get_ne_corner(),
            Direction::Down => ref_c.orig_workarea.get_se_corner(),
            Direction::Left => ref_c.orig_workarea.get_sw_corner(),
            Direction::Up => ref_c.orig_workarea.get_nw_corner(),
        };

        let nearest = self
            .containers
            .iter_mut()
            .filter(|(k, _c)| **k != ref_id) // Filter out the same monitor
            .map(|(_k, c)| c)
            .filter(|c| {
                // Filter out the ones that are not in the same direction
                let area = c.orig_workarea;
                match direction {
                    Direction::Right => area.x >= point.0,
                    Direction::Down => area.y >= point.1,
                    Direction::Left => area.x + i32::from(area.width) <= point.0,
                    Direction::Up => area.y + i32::from(area.height) <= point.1,
                }
            })
            .min_by(|a, b| {
                let dist1 = nearest_distance_area(point, a.orig_workarea, direction);
                let dist2 = nearest_distance_area(point, b.orig_workarea, direction);
                dist1.cmp(&dist2)
            });

        nearest
    }

    pub fn get_containers_ids(&self) -> Vec<T> {
        self.containers.keys().cloned().collect()
    }

    pub fn which_by_id(&self, id: T) -> Option<&Container> {
        self.containers.get(&id)
    }

    pub fn which_by_id_mut(&mut self, id: T) -> Option<&mut Container> {
        self.containers.get_mut(&id)
    }

    pub fn has_window(&self, hwnd: isize) -> bool {
        self.containers.iter().any(|(_k, c)| c.tree.has_id(hwnd))
    }

    pub fn get_windows(&self) -> HashSet<isize> {
        self.containers.iter().flat_map(|(_k, c)| c.tree.get_ids()).collect()
    }

    pub fn get_windows_len(&self) -> usize {
        self.containers.iter().map(|(_k, c)| c.tree.len()).sum()
    }

    pub fn get_leaf(&self, hwnd: isize, padding: i16) -> Option<AreaLeaf<isize>> {
        self.containers
            .iter()
            .find_map(|(_k, c)| c.tree.find_leaf(hwnd, padding))
    }
}

fn nearest_distance_area(ref_point: (i32, i32), area: Area, direction: Direction) -> u32 {
    let point = match direction {
        Direction::Right => match (ref_point.1 >= area.y, ref_point.1 <= area.y + i32::from(area.height)) {
            (true, true) => ref_point,
            (true, false) => area.get_sw_corner(),
            (false, true) => area.get_nw_corner(),
            _ => area.get_left_center(),
        },
        Direction::Down => match (ref_point.0 >= area.x, ref_point.0 <= area.x + i32::from(area.width)) {
            (true, true) => ref_point,
            (true, false) => area.get_ne_corner(),
            (false, true) => area.get_nw_corner(),
            _ => area.get_top_center(),
        },
        Direction::Left => match (ref_point.1 >= area.y, ref_point.1 <= area.y + i32::from(area.height)) {
            (true, true) => ref_point,
            (true, false) => area.get_se_corner(),
            (false, true) => area.get_ne_corner(),
            _ => area.get_right_center(),
        },
        Direction::Up => match (ref_point.0 >= area.x, ref_point.0 <= area.x + i32::from(area.width)) {
            (true, true) => ref_point,
            (true, false) => area.get_se_corner(),
            (false, true) => area.get_sw_corner(),
            _ => area.get_bottom_center(),
        },
    };
    point.distance(ref_point)
}

trait Point {
    fn distance(&self, other: (i32, i32)) -> u32;
}

impl Point for (i32, i32) {
    fn distance(&self, other: (i32, i32)) -> u32 {
        let (x1, y1) = self;
        let (x2, y2) = other;
        ((x2 - x1).pow(2) + (y2 - y1).pow(2)) as u32
    }
}
