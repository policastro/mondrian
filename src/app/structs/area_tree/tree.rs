use crate::app::structs::{area::Area, orientation::Orientation};

use super::{layout_strategy::LayoutStrategyEnum, leaf::AreaLeaf, node::AreaNode};
use std::{fmt::Debug, hash::Hash};

pub type WinTree = AreaTree<isize>;

pub struct AreaTree<T: Copy + Eq + Hash> {
    root: AreaNode<T>,
    area: Area,
    strategy: LayoutStrategyEnum,
    ids_map: std::collections::HashMap<T, AreaLeaf<T>>,
}

impl<T: Copy + Eq + Hash> AreaTree<T> {
    pub fn new(area: Area, strategy: LayoutStrategyEnum) -> AreaTree<T> {
        AreaTree {
            root: AreaNode::new(None, Orientation::Horizontal, 50),
            area,
            strategy,
            ids_map: std::collections::HashMap::new(),
        }
    }

    pub fn insert(&mut self, id: T) {
        if self.ids_map.contains_key(&id) {
            return;
        }
        self.root.insert(id, self.area, &mut self.strategy);
        self.update_map();
    }

    pub fn switch_subtree_orientations(&mut self, point: (i32, i32)) {
        if let Some(parent) = self.root.find_parent_mut(point, self.area) {
            parent.switch_subtree_orientations();
            self.update_map();
        }
    }

    pub fn leaves(&self, padding: i16) -> Vec<AreaLeaf<T>> {
        self.root.get_all_leaves(self.area.pad_full(padding))
    }

    pub fn find_leaf(&self, id: T, padding: i16) -> Option<AreaLeaf<T>> {
        let leaf = match self.ids_map.get(&id) {
            Some(l) => l,
            None => return None,
        };
        if padding == 0 {
            return Some(leaf.clone());
        }
        return self
            .root
            .find_leaf(leaf.viewbox.get_center(), self.area.pad_full(padding));
    }

    pub fn find_leaf_at(&self, point: (i32, i32), padding: i16) -> Option<AreaLeaf<T>> {
        self.root.find_leaf(point, self.area.pad_full(padding))
    }

    pub fn remove(&mut self, id: T) {
        if let Some(leaf) = self.ids_map.remove(&id) {
            self.root.remove(leaf.viewbox.get_center(), self.area, &mut self.strategy);
            self.update_map();
        }
    }
    pub fn remove_at(&mut self, point: (i32, i32)) {
        self.root.remove(point, self.area, &mut self.strategy);
        self.update_map();
    }

    pub(crate) fn resize_ancestor(
        &mut self,
        orig_point1: (i32, i32),
        orig_point2: (i32, i32),
        grow_ratio: i32,
        clamp_values: Option<(u8, u8)>,
    ) {
        let clamp_values = clamp_values.unwrap_or((0, 100));
        assert!(clamp_values.0 <= 100 && clamp_values.1 <= 100);
        assert!(clamp_values.0 <= clamp_values.1);
        let ancestor = self
            .root
            .find_lowest_common_ancestor(orig_point1, orig_point2, self.area);

        if let Some(ancestor) = ancestor {
            let is_less = (orig_point1.0 < orig_point2.0, orig_point1.1 < orig_point2.1);

            let ratio_to_consider = match (ancestor.orientation, is_less) {
                (Orientation::Vertical, (true, _)) => ancestor.ratio,
                (Orientation::Vertical, (false, _)) => 100u8.saturating_sub(ancestor.ratio),
                (Orientation::Horizontal, (_, true)) => ancestor.ratio,
                (Orientation::Horizontal, (_, false)) => 100u8.saturating_sub(ancestor.ratio),
            };

            let real_grow_ratio = (grow_ratio as f32 * ratio_to_consider as f32) / 100f32;

            ancestor.ratio = match real_grow_ratio < 0f32 {
                true => ancestor.ratio.saturating_sub(real_grow_ratio.abs() as u8),
                false => ancestor.ratio.saturating_add(real_grow_ratio as u8),
            };

            ancestor.ratio = ancestor.ratio.clamp(clamp_values.0, clamp_values.1);

            self.update_map();
        }
    }

    pub(crate) fn swap_ids_at(&mut self, point1: (i32, i32), point2: (i32, i32)) {
        let id1 = self.root.get_id(point1, self.area);
        if id1.is_none() {
            return;
        }

        let id2 = self.root.set_id(id1.unwrap(), point2, self.area);

        if id2.is_none() {
            self.update_map();
            return;
        }
        self.root.set_id(id2.unwrap(), point1, self.area);

        self.update_map();
    }

    pub fn replace_id(&mut self, point: (i32, i32), id: T) -> Option<T> {
        let v = self.root.set_id(id, point, self.area);
        self.update_map();
        v
    }

    pub fn has_id(&self, id: T) -> bool {
        self.ids_map.contains_key(&id)
    }

    pub fn get_ids(&self) -> Vec<T> {
        self.ids_map.keys().cloned().collect()
    }

    pub fn len(&self) -> usize {
        self.ids_map.len()
    }

    fn update_map(&mut self) {
        self.ids_map = std::collections::HashMap::new();
        self.root.get_all_leaves(self.area).iter().for_each(|leaf| {
            self.ids_map.insert(leaf.id, *leaf);
        })
    }
}

impl<T: Debug + Copy + Eq + Hash> Debug for AreaTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.root)
    }
}
