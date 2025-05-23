pub mod container;
pub mod keys;
pub mod map;

use super::tm::result::TilesManagerError;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::structs::area::Area;
use crate::app::structs::direction::Direction;
use crate::app::structs::point::Point;
use crate::win32::window::window_ref::WindowRef;
use container::Container;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

type Result<T> = std::result::Result<T, TilesManagerError>;

#[derive(Debug)]
pub struct ContainerEntry<K, V> {
    pub key: K,
    pub value: V,
}

impl<K, V> ContainerEntry<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

pub(super) trait Containers<K, U: Clone + Eq + Hash> {
    fn find(&self, win: WindowRef) -> Result<ContainerEntry<K, &Container>>;
    fn find_at(&self, point: (i32, i32)) -> Result<ContainerEntry<K, &Container>>;
    fn find_near(&self, point: (i32, i32)) -> Result<ContainerEntry<K, &Container>>;
    fn find_closest_at(&self, point: (i32, i32), direction: Direction) -> Result<ContainerEntry<K, &Container>>;
    fn find_leaf(&self, win: WindowRef) -> Result<AreaLeaf<WindowRef>>;
    fn find_leaf_at(&self, point: (i32, i32)) -> Result<AreaLeaf<WindowRef>>;
}

pub(super) trait ContainersMut<K, U: Clone + Eq + Hash> {
    fn find_mut(&mut self, win: WindowRef) -> Result<ContainerEntry<K, &mut Container>>;
    fn find_near_mut(&mut self, point: (i32, i32)) -> Result<ContainerEntry<K, &mut Container>>;
}

impl<K: Clone + Eq + Hash + Debug> Containers<K, String> for HashMap<K, Container> {
    fn find(&self, win: WindowRef) -> Result<ContainerEntry<K, &Container>> {
        self.iter()
            .find(|c| c.1.tree().has(win))
            .map(|c| ContainerEntry::new(c.0.clone(), c.1))
            .ok_or(TilesManagerError::WinNotManaged(win))
    }

    fn find_at(&self, point: (i32, i32)) -> Result<ContainerEntry<K, &Container>> {
        self.iter()
            .find(|c| c.1.tree().contains(point, false))
            .map(|(k, v)| ContainerEntry::new(k.clone(), v))
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }

    fn find_near(&self, point: (i32, i32)) -> Result<ContainerEntry<K, &Container>> {
        self.iter()
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.tree().get_area());
                let dist2 = calc_distance(point, b.1.tree().get_area());
                dist1.cmp(&dist2)
            })
            .map(|(k, v)| ContainerEntry::new(k.clone(), v))
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }

    // INFO: finds the closest container in the given direction, ignoring the one in which the ref_point is
    fn find_closest_at(
        &self,
        ref_point: (i32, i32),
        limit_direction: Direction,
    ) -> Result<ContainerEntry<K, &Container>> {
        let ref_entry = self.find_near(ref_point)?;
        let ref_key = ref_entry.key;
        let ref_area = ref_entry.value.tree().get_area();
        let point = match limit_direction {
            Direction::Right => ref_area.get_ne_corner(),
            Direction::Down => ref_area.get_se_corner(),
            Direction::Left => ref_area.get_sw_corner(),
            Direction::Up => ref_area.get_nw_corner(),
        };

        let closest = self
            .iter()
            .filter(|(k, c)| {
                // Filter out the same monitor
                if **k == ref_key {
                    return false;
                };

                // Filter out the ones that are not in the same direction
                let area = c.tree().get_area();
                match limit_direction {
                    Direction::Right => area.x >= point.0,
                    Direction::Down => area.y >= point.1,
                    Direction::Left => area.x + i32::from(area.width) <= point.0,
                    Direction::Up => area.y + i32::from(area.height) <= point.1,
                }
            })
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.tree().get_area());
                let dist2 = calc_distance(point, b.1.tree().get_area());
                dist1.cmp(&dist2)
            });

        closest
            .map(|c| ContainerEntry::new(c.0.clone(), c.1))
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }

    fn find_leaf(&self, win: WindowRef) -> Result<AreaLeaf<WindowRef>> {
        self.find(win)?
            .value
            .tree()
            .find_leaf(win, 0)
            .ok_or(TilesManagerError::WinNotManaged(win))
    }

    fn find_leaf_at(&self, point: (i32, i32)) -> Result<AreaLeaf<WindowRef>> {
        self.find_at(point)?
            .value
            .tree()
            .find_leaf_at(point, 0)
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }
}

impl<K: Clone + Eq + Hash> ContainersMut<K, String> for HashMap<K, Container> {
    fn find_mut(&mut self, win: WindowRef) -> Result<ContainerEntry<K, &mut Container>> {
        self.iter_mut()
            .find(|c| c.1.tree().has(win))
            .map(|c| ContainerEntry::new(c.0.clone(), c.1))
            .ok_or(TilesManagerError::WinNotManaged(win))
    }

    fn find_near_mut(&mut self, point: (i32, i32)) -> Result<ContainerEntry<K, &mut Container>> {
        self.iter_mut()
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.tree().get_area());
                let dist2 = calc_distance(point, b.1.tree().get_area());
                dist1.cmp(&dist2)
            })
            .map(|(k, v)| ContainerEntry::new(k.clone(), v))
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }
}

fn calc_distance(ref_point: (i32, i32), area: Area) -> u32 {
    let projection = (
        ref_point.0.clamp(area.x, area.x + i32::from(area.width)),
        ref_point.1.clamp(area.y, area.y + i32::from(area.height)),
    );
    ref_point.distance(projection)
}
