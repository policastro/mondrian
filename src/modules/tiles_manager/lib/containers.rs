use super::tm::error::TilesManagerError;
use super::window_animation_player::WindowAnimationPlayer;
use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::area_tree::tree::WinTree;
use crate::app::structs::point::Point;
use crate::app::structs::{area::Area, direction::Direction};
use crate::win32::window::window_obj::{WindowObjHandler, WindowObjInfo};
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashSet;
use std::hash::Hasher;
use std::{collections::HashMap, hash::Hash};

type Result<T> = std::result::Result<T, TilesManagerError>;

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ContainerLayer {
    Normal,
    Focalized,
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct ContainerKey {
    pub vd: u128,
    pub monitor: String,
    pub layer: ContainerLayer, // TODO: support for multiple layers
}

#[derive(Eq, Clone)]
pub struct CrossLayerContainerKey {
    pub vd: u128,
    pub monitor: String,
    pub layer: ContainerLayer, // TODO: support for multiple layers
}

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

pub trait InactiveContainers {
    fn get_normal_mut(&mut self, base_key: &ContainerKey) -> Result<&mut WinTree>;
    fn set_layers_area(&mut self, key: &CrossLayerContainerKey, new_area: Area);
    fn has_vd(&self, vd: u128) -> bool;
}

pub trait Container {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animator: &mut WindowAnimationPlayer,
        ignored_wins: &HashSet<WindowRef>,
    ) -> Result<()>;
    fn contains(&self, point: (i32, i32)) -> bool;
}

pub(super) trait Containers<K, U: Clone + Eq + Hash> {
    fn find(&self, win: WindowRef) -> Result<ContainerEntry<K, &WinTree>>;
    fn find_at(&self, point: (i32, i32)) -> Result<ContainerEntry<K, &WinTree>>;
    fn find_at_or_near(&mut self, point: (i32, i32)) -> Result<ContainerEntry<K, &WinTree>>;
    fn find_closest_at(&self, point: (i32, i32), direction: Direction) -> Result<ContainerEntry<K, &WinTree>>;
    fn find_leaf(&self, win: WindowRef) -> Result<AreaLeaf<WindowRef>>;
    fn find_leaf_at(&self, point: (i32, i32)) -> Result<AreaLeaf<WindowRef>>;
}

pub(super) trait ContainersMut<K, U: Clone + Eq + Hash> {
    fn find_mut(&mut self, win: WindowRef) -> Result<ContainerEntry<K, &mut WinTree>>;
    fn find_at_mut(&mut self, point: (i32, i32)) -> Result<ContainerEntry<K, &mut WinTree>>;
}

impl Container for WinTree {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animation_player: &mut WindowAnimationPlayer,
        ignored_wins: &HashSet<WindowRef>,
    ) -> Result<()> {
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
            animation_player.queue(leaf.id, area, false);
        }
        Ok(())
    }

    fn contains(&self, point: (i32, i32)) -> bool {
        self.get_area().contains(point)
    }
}

impl<K: Clone + Eq + Hash> Containers<K, String> for HashMap<K, WinTree> {
    fn find(&self, win: WindowRef) -> Result<ContainerEntry<K, &WinTree>> {
        self.iter()
            .find(|c| c.1.has(win))
            .map(|c| ContainerEntry::new(c.0.clone(), c.1))
            .ok_or(TilesManagerError::WinNotManaged(win))
    }

    fn find_at(&self, point: (i32, i32)) -> Result<ContainerEntry<K, &WinTree>> {
        self.iter()
            .find(|c| c.1.contains(point))
            .map(|(k, v)| ContainerEntry::new(k.clone(), v))
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }

    fn find_at_or_near(&mut self, point: (i32, i32)) -> Result<ContainerEntry<K, &WinTree>> {
        self.iter()
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.get_area());
                let dist2 = calc_distance(point, b.1.get_area());
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
    ) -> Result<ContainerEntry<K, &WinTree>> {
        let ref_id = (*self
            .iter()
            .find(|c| c.1.contains(ref_point))
            .ok_or(TilesManagerError::NoContainerAtPoint(ref_point))?
            .0)
            .clone();
        let ref_area = self
            .get(&ref_id)
            .ok_or(TilesManagerError::NoContainerAtPoint(ref_point))?
            .get_area();
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
                let area = c.get_area();
                match limit_direction {
                    Direction::Right => area.x >= point.0,
                    Direction::Down => area.y >= point.1,
                    Direction::Left => area.x + i32::from(area.width) <= point.0,
                    Direction::Up => area.y + i32::from(area.height) <= point.1,
                }
            })
            .min_by(|a, b| {
                let dist1 = calc_distance(point, a.1.get_area());
                let dist2 = calc_distance(point, b.1.get_area());
                dist1.cmp(&dist2)
            });

        closest
            .map(|c| ContainerEntry::new(c.0.clone(), c.1))
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }

    fn find_leaf(&self, win: WindowRef) -> Result<AreaLeaf<WindowRef>> {
        self.find(win)?
            .value
            .find_leaf(win, 0)
            .ok_or(TilesManagerError::WinNotManaged(win))
    }

    fn find_leaf_at(&self, point: (i32, i32)) -> Result<AreaLeaf<WindowRef>> {
        self.find_at(point)?
            .value
            .find_leaf_at(point, 0)
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }
}

impl<K: Clone + Eq + Hash> ContainersMut<K, String> for HashMap<K, WinTree> {
    fn find_mut(&mut self, win: WindowRef) -> Result<ContainerEntry<K, &mut WinTree>> {
        self.iter_mut()
            .find(|c| c.1.has(win))
            .map(|c| ContainerEntry::new(c.0.clone(), c.1))
            .ok_or(TilesManagerError::WinNotManaged(win))
    }

    fn find_at_mut(&mut self, point: (i32, i32)) -> Result<ContainerEntry<K, &mut WinTree>> {
        self.iter_mut()
            .find(|c| c.1.contains(point))
            .map(|v| ContainerEntry::new(v.0.clone(), v.1))
            .ok_or(TilesManagerError::NoContainerAtPoint(point))
    }
}

impl InactiveContainers for HashMap<ContainerKey, (WinTree, u128)> {
    fn get_normal_mut(&mut self, base_key: &ContainerKey) -> Result<&mut WinTree> {
        self.get_mut(&base_key.to_normal())
            .ok_or(TilesManagerError::ContainerNotFound { refresh: false })
            .map(|(t, _)| t)
    }

    fn set_layers_area(&mut self, key: &CrossLayerContainerKey, new_area: Area) {
        self.iter_mut()
            .filter(|(k, _)| k.is_vd(key.vd) && k.is_monitor(&key.monitor))
            .for_each(|(_, (t, _))| t.set_area(new_area));
    }

    fn has_vd(&self, vd: u128) -> bool {
        self.iter().any(|(k, _)| k.is_vd(vd))
    }
}

impl ContainerKey {
    pub fn new(vd: u128, monitor: String, layer: ContainerLayer) -> Self {
        ContainerKey { vd, monitor, layer }
    }

    pub fn is_vd(&self, vd: u128) -> bool {
        self.vd == vd
    }

    pub fn is_monitor(&self, monitor: &str) -> bool {
        self.monitor == monitor
    }

    pub fn is_layer(&self, layer: ContainerLayer) -> bool {
        self.layer == layer
    }

    pub fn normal(vd: u128, monitor: String) -> Self {
        ContainerKey {
            vd,
            monitor,
            layer: ContainerLayer::Normal,
        }
    }

    pub fn focalized(vd: u128, monitor: String) -> Self {
        ContainerKey {
            vd,
            monitor,
            layer: ContainerLayer::Focalized,
        }
    }

    pub fn to_normal(&self) -> Self {
        ContainerKey {
            vd: self.vd,
            monitor: self.monitor.clone(),
            layer: ContainerLayer::Normal,
        }
    }
}

impl PartialEq for CrossLayerContainerKey {
    fn eq(&self, other: &Self) -> bool {
        self.vd == other.vd && self.monitor == other.monitor
    }
}

impl Hash for CrossLayerContainerKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vd.hash(state);
        self.monitor.hash(state);
    }
}

impl From<ContainerKey> for CrossLayerContainerKey {
    fn from(value: ContainerKey) -> Self {
        CrossLayerContainerKey {
            vd: value.vd,
            monitor: value.monitor,
            layer: value.layer,
        }
    }
}

impl From<CrossLayerContainerKey> for ContainerKey {
    fn from(value: CrossLayerContainerKey) -> Self {
        ContainerKey::new(value.vd, value.monitor, value.layer)
    }
}

fn calc_distance(ref_point: (i32, i32), area: Area) -> u32 {
    let projection = (
        ref_point.0.clamp(area.x, area.x + i32::from(area.width)),
        ref_point.1.clamp(area.y, area.y + i32::from(area.height)),
    );
    ref_point.distance(projection)
}
