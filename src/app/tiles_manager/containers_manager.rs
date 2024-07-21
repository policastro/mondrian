use std::collections::HashMap;

use crate::app::structs::{area::Area, area_tree::tree::WinTree, direction::Direction};

use super::container::Container;

pub(super) struct ContainersManager {
    containers: HashMap<isize, Container>,
}

impl ContainersManager {
    pub fn new(containers: Vec<Container>) -> Self {
        ContainersManager {
            containers: containers.into_iter().map(|c| (c.monitor.id, c)).collect(),
        }
    }

    pub fn is_same_container(&self, point1: (i32, i32), point2: (i32, i32)) -> bool {
        let c1 = self.which(point1);
        let c2 = self.which(point2);

        match (c1, c2) {
            (Some(c1), Some(c2)) => c1.monitor.id == c2.monitor.id,
            _ => false,
        }
    }

    pub fn which(&self, point: (i32, i32)) -> Option<&Container> {
        self.containers.values().find(|c| c.contains(point))
    }

    pub fn which_mut(&mut self, point: (i32, i32)) -> Option<&mut Container> {
        self.containers.values_mut().find(|c| c.contains(point))
    }

    pub fn which_tree(&mut self, point: (i32, i32)) -> Option<&mut WinTree> {
        self.which_mut(point).map(|c| &mut c.tree)
    }

    pub fn which_nearest_mut(&mut self, ref_point: (i32, i32), direction: Direction) -> Option<&mut Container> {
        let ref_m = self
            .containers
            .values()
            .find(|c| c.contains(ref_point))
            .map(|c| c.monitor)?;

        let point = match direction {
            Direction::Right => ref_m.workarea.get_ne_corner(),
            Direction::Down => ref_m.workarea.get_se_corner(),
            Direction::Left => ref_m.workarea.get_sw_corner(),
            Direction::Up => ref_m.workarea.get_nw_corner(),
        };

        let nearest = self
            .containers
            .values_mut()
            .filter(|c| c.monitor.id != ref_m.id) // Filter out the same monitor
            .filter(|c| {
                // Filter out the ones that are not in the same direction
                let area = c.monitor.workarea;
                match direction {
                    Direction::Right => area.x >= point.0,
                    Direction::Down => area.y >= point.1,
                    Direction::Left => area.x + i32::from(area.width) <= point.0,
                    Direction::Up => area.y + i32::from(area.height) <= point.1,
                }
            })
            .min_by(|a, b| {
                let dist1 = nearest_distance_area(point, a.monitor.workarea, direction);
                let dist2 = nearest_distance_area(point, b.monitor.workarea, direction);
                dist1.cmp(&dist2)
            });

        nearest
    }

    pub fn get_containers_ids(&self) -> Vec<isize> {
        self.containers.keys().cloned().collect()
    }

    pub fn get_container(&self, id: isize) -> Option<&Container> {
        self.containers.get(&id)
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
