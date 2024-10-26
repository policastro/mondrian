use crate::app::area_tree::leaf::AreaLeaf;
use crate::app::area_tree::tree::WinTree;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::hash::Hash;

use super::window_animation_player::WindowAnimationPlayer;

pub trait ContainerLayer {
    fn update(&mut self, border_pad: i16, tile_pad: (i16, i16), animator: &mut WindowAnimationPlayer)
        -> Result<(), ()>;
    fn contains(&self, point: (i32, i32)) -> bool;
}

impl ContainerLayer for WinTree {
    fn update(
        &mut self,
        border_pad: i16,
        tile_pad: (i16, i16),
        animation_player: &mut WindowAnimationPlayer,
    ) -> Result<(), ()> {
        let leaves: Vec<AreaLeaf<isize>> = self.leaves(border_pad);

        for leaf in &leaves {
            let win_ref = WindowRef::from(leaf.id);
            if !win_ref.is_visible() {
                self.remove(win_ref.hwnd.0);
                return self.update(border_pad, tile_pad, animation_player);
            };
            let area = leaf.viewbox.pad_xy(tile_pad);
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
