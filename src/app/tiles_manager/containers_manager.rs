use super::container::Container;
use crate::app::structs::point::Point;
use crate::app::structs::{area::Area, direction::Direction};
use std::{collections::HashMap, hash::Hash};

pub(super) trait Containers<U: Clone + Eq + Hash> {
    fn find_at(&self, point: (i32, i32)) -> Option<&Container<U>>;
    fn find_at_mut(&mut self, point: (i32, i32)) -> Option<&mut Container<U>>;
    fn find(&self, hwnd: isize, only_active: bool) -> Option<&Container<U>>;
    fn find_mut(&mut self, hwnd: isize, only_active: bool) -> Option<&mut Container<U>>;
    fn is_same_container(&self, point1: (i32, i32), point2: (i32, i32)) -> bool;
    fn find_nearest(&self, ref_point: (i32, i32), direction: Direction) -> Option<&Container<U>>;
}

impl Containers<String> for HashMap<isize, Container<String>> {
    fn find_at(&self, point: (i32, i32)) -> Option<&Container<String>> {
        self.values().find(|c| c.contains(point))
    }

    fn find_at_mut(&mut self, point: (i32, i32)) -> Option<&mut Container<String>> {
        self.values_mut().find(|c| c.contains(point))
    }

    fn find(&self, hwnd: isize, only_active: bool) -> Option<&Container<String>> {
        let mut v = self.values();
        match only_active {
            true => v.find(|c| c.get_active().is_some_and(|t| t.has(hwnd))),
            false => v.find(|c| c.iter().any(|w| w.1.has(hwnd))),
        }
    }

    fn find_mut(&mut self, hwnd: isize, only_active: bool) -> Option<&mut Container<String>> {
        let mut v = self.values_mut();
        match only_active {
            true => v.find(|c| c.get_active().is_some_and(|t| t.has(hwnd))),
            false => v.find(|c| c.iter().any(|w| w.1.has(hwnd))),
        }
    }

    fn is_same_container(&self, point1: (i32, i32), point2: (i32, i32)) -> bool {
        let c1 = self.iter().find(|c| c.1.contains(point1));
        let c2 = self.iter().find(|c| c.1.contains(point2));
        c1.is_some_and(|c1| c2.is_some_and(|c2| c1.0 == c2.0))
    }

    fn find_nearest(&self, ref_point: (i32, i32), direction: Direction) -> Option<&Container<String>> {
        let ref_id = *self.iter().find(|c| c.1.contains(ref_point))?.0;
        let ref_area = self.get(&ref_id)?.get_active()?.area;
        let point = match direction {
            Direction::Right => ref_area.get_ne_corner(),
            Direction::Down => ref_area.get_se_corner(),
            Direction::Left => ref_area.get_sw_corner(),
            Direction::Up => ref_area.get_nw_corner(),
        };

        let nearest = self
            .iter()
            .filter(|(k, _c)| **k != ref_id) // Filter out the same monitor
            .filter(|(_k, c)| {
                // Filter out the ones that are not in the same direction
                let area = c.get_active().expect("Container must have an active tree").area;
                match direction {
                    Direction::Right => area.x >= point.0,
                    Direction::Down => area.y >= point.1,
                    Direction::Left => area.x + i32::from(area.width) <= point.0,
                    Direction::Up => area.y + i32::from(area.height) <= point.1,
                }
            })
            .min_by(|a, b| {
                let dist1 = nearest_distance_area(point, a.1.get_active().unwrap().area, direction);
                let dist2 = nearest_distance_area(point, b.1.get_active().unwrap().area, direction);
                dist1.cmp(&dist2)
            });

        nearest.map(|c| c.1)
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
