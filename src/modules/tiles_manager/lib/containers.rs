use super::window_animation_player::WindowAnimationPlayer;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::area_tree::tree::WinTree;
use crate::app::structs::point::Point;
use crate::app::structs::{area::Area, direction::Direction};
use crate::win32::window::window_obj::{WindowObjHandler, WindowObjInfo};
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashSet;
use std::{collections::HashMap, hash::Hash};

#[derive(Debug)]
pub struct ContainerEntry<K, V> {
    pub key: K,
    pub value: V,
}

pub trait Container {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animator: &mut WindowAnimationPlayer,
        ignored_wins: &HashSet<WindowRef>,
    ) -> Result<(), ()>;
    fn contains(&self, point: (i32, i32)) -> bool;
}

pub(super) trait Containers<K, U: Clone + Eq + Hash> {
    fn find(&self, win: WindowRef) -> Option<ContainerEntry<K, &WinTree>>;
    fn find_mut(&mut self, win: WindowRef) -> Option<ContainerEntry<K, &mut WinTree>>;
    fn find_at(&self, point: (i32, i32)) -> Option<ContainerEntry<K, &WinTree>>;
    fn find_at_mut(&mut self, point: (i32, i32)) -> Option<ContainerEntry<K, &mut WinTree>>;
    fn find_at_or_near_mut(&mut self, point: (i32, i32)) -> Option<ContainerEntry<K, &mut WinTree>>;
    fn find_closest_at(&self, ref_point: (i32, i32), direction: Direction) -> Option<ContainerEntry<K, &WinTree>>;
    fn find_closest_at_mut(
        &mut self,
        ref_point: (i32, i32),
        direction: Direction,
    ) -> Option<ContainerEntry<K, &mut WinTree>>;
}

impl<K, V> ContainerEntry<K, V> {
    pub fn new(key: K, value: V) -> Self {
        Self { key, value }
    }
}

impl Container for WinTree {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animation_player: &mut WindowAnimationPlayer,
        ignored_wins: &HashSet<WindowRef>,
    ) -> Result<(), ()> {
        let leaves: Vec<AreaLeaf<WindowRef>> = self.leaves(border_pad, Some(ignored_wins));

        for leaf in &leaves {
            if !leaf.id.is_visible() {
                self.remove(leaf.id);
                return self.update(border_pad, tile_pad, animation_player, ignored_wins);
            };
            let area = leaf.viewbox.pad_xy(tile_pad);
            leaf.id.restore(false);
            let borders = leaf.id.get_borders().unwrap_or((0, 0, 0, 0));
            let borders = (
                borders.0.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
                borders.1.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
                borders.2.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
                borders.3.clamp(u16::MIN as i32, u16::MAX as i32) as i16,
            );
            let area = area.shift((-borders.0, -borders.1, borders.2 + borders.0, borders.3 + borders.1));
            animation_player.queue(leaf.id, area);
        }
        Ok(())
    }

    fn contains(&self, point: (i32, i32)) -> bool {
        self.area.contains(point)
    }
}

impl<K: Clone + Eq + Hash> Containers<K, String> for HashMap<K, WinTree> {
    fn find(&self, win: WindowRef) -> Option<ContainerEntry<K, &WinTree>> {
        self.iter()
            .find(|c| c.1.has(win))
            .map(|c| ContainerEntry::new(c.0.clone(), c.1))
    }

    fn find_mut(&mut self, win: WindowRef) -> Option<ContainerEntry<K, &mut WinTree>> {
        self.iter_mut()
            .find(|c| c.1.has(win))
            .map(|c| ContainerEntry::new(c.0.clone(), c.1))
    }

    fn find_at(&self, point: (i32, i32)) -> Option<ContainerEntry<K, &WinTree>> {
        self.iter()
            .find(|c| c.1.contains(point))
            .map(|(k, v)| ContainerEntry::new(k.clone(), v))
    }

    fn find_at_mut(&mut self, point: (i32, i32)) -> Option<ContainerEntry<K, &mut WinTree>> {
        self.iter_mut()
            .find(|c| c.1.contains(point))
            .map(|v| ContainerEntry::new(v.0.clone(), v.1))
    }

    fn find_at_or_near_mut(&mut self, point: (i32, i32)) -> Option<ContainerEntry<K, &mut WinTree>> {
        self.iter_mut()
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.area);
                let dist2 = calc_distance(point, b.1.area);
                dist1.cmp(&dist2)
            })
            .map(|(k, v)| ContainerEntry::new(k.clone(), v))
    }

    // NOTE: finds the closest container in the given direction, ignoring the one in which the ref_point is
    fn find_closest_at(
        &self,
        ref_point: (i32, i32),
        limit_direction: Direction,
    ) -> Option<ContainerEntry<K, &WinTree>> {
        let ref_id = (*self.iter().find(|c| c.1.contains(ref_point))?.0).clone();
        let ref_area = self.get(&ref_id)?.area;
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
                if **k == ref_id {
                    return false;
                };

                // Filter out the ones that are not in the same direction
                let area = c.area;
                match limit_direction {
                    Direction::Right => area.x >= point.0,
                    Direction::Down => area.y >= point.1,
                    Direction::Left => area.x + i32::from(area.width) <= point.0,
                    Direction::Up => area.y + i32::from(area.height) <= point.1,
                }
            })
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.area);
                let dist2 = calc_distance(point, b.1.area);
                dist1.cmp(&dist2)
            });

        closest.map(|c| ContainerEntry::new(c.0.clone(), c.1))
    }

    fn find_closest_at_mut(
        &mut self,
        ref_point: (i32, i32),
        limit_direction: Direction,
    ) -> Option<ContainerEntry<K, &mut WinTree>> {
        let ref_id = (*self.iter().find(|c| c.1.contains(ref_point))?.0).clone();
        let ref_area = self.get(&ref_id)?.area;
        let point = match limit_direction {
            Direction::Right => ref_area.get_ne_corner(),
            Direction::Down => ref_area.get_se_corner(),
            Direction::Left => ref_area.get_sw_corner(),
            Direction::Up => ref_area.get_nw_corner(),
        };

        let closest = self
            .iter_mut()
            .filter(|(k, c)| {
                // Filter out the same monitor
                if **k == ref_id {
                    return false;
                };

                // Filter out the ones that are not in the same direction
                let area = c.area;
                match limit_direction {
                    Direction::Right => area.x >= point.0,
                    Direction::Down => area.y >= point.1,
                    Direction::Left => area.x + i32::from(area.width) <= point.0,
                    Direction::Up => area.y + i32::from(area.height) <= point.1,
                }
            })
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.area);
                let dist2 = calc_distance(point, b.1.area);
                dist1.cmp(&dist2)
            });

        closest.map(|c| ContainerEntry::new(c.0.clone(), c.1))
    }
}

fn calc_distance(ref_point: (i32, i32), area: Area) -> u32 {
    let projection = (
        ref_point.0.clamp(area.x, area.x + i32::from(area.width)),
        ref_point.1.clamp(area.y, area.y + i32::from(area.height)),
    );
    ref_point.distance(projection)
}
