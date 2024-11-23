use super::window_animation_player::WindowAnimationPlayer;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::area_tree::tree::WinTree;
use crate::app::structs::point::Point;
use crate::app::structs::{area::Area, direction::Direction};
use crate::win32::window::window_obj::{WindowObjHandler, WindowObjInfo};
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashSet;
use std::{collections::HashMap, hash::Hash};

pub trait ContainerLayer {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animator: &mut WindowAnimationPlayer,
        ignored_wins: &HashSet<isize>,
    ) -> Result<(), ()>;
    fn contains(&self, point: (i32, i32)) -> bool;
}

impl ContainerLayer for WinTree {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animation_player: &mut WindowAnimationPlayer,
        ignored_wins: &HashSet<isize>,
    ) -> Result<(), ()> {
        let leaves: Vec<AreaLeaf<isize>> = self.leaves(border_pad, ignored_wins);

        for leaf in &leaves {
            let win_ref = WindowRef::from(leaf.id);
            if !win_ref.is_visible() {
                self.remove(win_ref.hwnd.0);
                return self.update(border_pad, tile_pad, animation_player, ignored_wins);
            };
            let area = leaf.viewbox.pad_xy(tile_pad);
            win_ref.restore(false);
            let area = win_ref.adjust_area(area);
            animation_player.queue(win_ref, area);
        }
        Ok(())
    }

    fn contains(&self, point: (i32, i32)) -> bool {
        self.area.contains(point)
    }
}

#[derive(Debug)]
pub(super) struct Container<K: Clone + Eq + Hash> {
    map: HashMap<K, WinTree>,
    active: Option<K>,
}

impl<K: Clone + Eq + Hash> Container<K> {
    pub fn new() -> Self {
        Container {
            map: HashMap::new(),
            active: None,
        }
    }

    pub fn add(&mut self, key: K, tree: WinTree) -> Option<WinTree> {
        self.map.insert(key, tree)
    }

    pub fn get(&self, key: K) -> Option<&WinTree> {
        self.map.get(&key)
    }

    pub fn get_mut(&mut self, key: K) -> Option<&mut WinTree> {
        self.map.get_mut(&key)
    }

    pub fn get_active(&self) -> Option<&WinTree> {
        self.get(self.active.clone()?)
    }

    pub fn get_active_mut(&mut self) -> Option<&mut WinTree> {
        self.get_mut(self.active.clone()?)
    }

    pub fn set_active(&mut self, key: K) -> Result<(), ()> {
        self.map.get(&key).map(|_| self.active = Some(key)).ok_or(())
    }

    pub fn is_active(&self, key: K) -> bool {
        self.active.clone() == Some(key)
    }

    pub fn contains(&self, point: (i32, i32)) -> bool {
        self.get_active().is_some_and(|t| t.contains(point))
    }

    pub fn has(&self, hwnd: isize) -> bool {
        self.get_active().is_some_and(|t| t.has(hwnd))
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<'_, K, WinTree> {
        self.map.iter()
    }

    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<'_, K, WinTree> {
        self.map.iter_mut()
    }
}

pub(super) trait Containers<U: Clone + Eq + Hash> {
    fn find_at(&self, point: (i32, i32)) -> Option<&Container<U>>;
    fn find_at_mut(&mut self, point: (i32, i32)) -> Option<&mut Container<U>>;
    fn find_at_or_near_mut(&mut self, point: (i32, i32)) -> Option<&mut Container<U>>;
    fn find(&self, hwnd: isize, only_active: bool) -> Option<&Container<U>>;
    fn find_mut(&mut self, hwnd: isize, only_active: bool) -> Option<&mut Container<U>>;
    fn is_same_container(&self, point1: (i32, i32), point2: (i32, i32)) -> bool;
    fn find_closest_at(&self, ref_point: (i32, i32), direction: Direction) -> Option<&Container<U>>;
}

impl Containers<String> for HashMap<isize, Container<String>> {
    fn find_at(&self, point: (i32, i32)) -> Option<&Container<String>> {
        self.values().find(|c| c.contains(point))
    }

    fn find_at_mut(&mut self, point: (i32, i32)) -> Option<&mut Container<String>> {
        self.values_mut().find(|c| c.contains(point))
    }

    fn find_at_or_near_mut(&mut self, point: (i32, i32)) -> Option<&mut Container<String>> {
        self.values_mut().min_by(|a, b| {
            let dist1 = calc_distance(point, a.get_active().unwrap().area);
            let dist2 = calc_distance(point, b.get_active().unwrap().area);
            dist1.cmp(&dist2)
        })
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

    // NOTE: finds the closest container in the given direction, ignoring the one in which the ref_point is
    fn find_closest_at(&self, ref_point: (i32, i32), limit_direction: Direction) -> Option<&Container<String>> {
        let ref_id = *self.iter().find(|c| c.1.contains(ref_point))?.0;
        let ref_area = self.get(&ref_id)?.get_active()?.area;
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
                let area = c.get_active().expect("Container must have an active tree").area;
                match limit_direction {
                    Direction::Right => area.x >= point.0,
                    Direction::Down => area.y >= point.1,
                    Direction::Left => area.x + i32::from(area.width) <= point.0,
                    Direction::Up => area.y + i32::from(area.height) <= point.1,
                }
            })
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.get_active().unwrap().area);
                let dist2 = calc_distance(point, b.1.get_active().unwrap().area);
                dist1.cmp(&dist2)
            });

        closest.map(|c| c.1)
    }
}

fn calc_distance(ref_point: (i32, i32), area: Area) -> u32 {
    let projection = (
        ref_point.0.clamp(area.x, area.x + i32::from(area.width)),
        ref_point.1.clamp(area.y, area.y + i32::from(area.height)),
    );
    ref_point.distance(projection)
}
