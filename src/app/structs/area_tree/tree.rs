use crate::app::structs::{area::Area, orientation::Orientation};

use super::{
    layout_strategy::{LayoutStrategy, LayoutStrategyEnum},
    leaf::AreaLeaf,
    node::AreaNode,
};
use std::fmt::Debug;

pub type WinTree = AreaTree<isize>;

pub struct AreaTree<T: Copy> {
    root: AreaNode<T>,
    area: Area,
    layout_strategy: LayoutStrategyEnum,
}

impl<T: Copy> AreaTree<T> {
    pub fn new(area: Area, layout_strategy: LayoutStrategyEnum) -> AreaTree<T> {
        let (orientation, ratio) = layout_strategy.get_initial_params();
        AreaTree {
            root: AreaNode::new_internal(orientation, ratio),
            area,
            layout_strategy,
        }
    }

    pub fn insert(&mut self, id: T) {
        self.root.insert(id, self.area, &mut self.layout_strategy);
    }

    pub fn switch_subtree_orientations(&mut self, point: (i32, i32)) {
        if let Some(parent) = self.root.find_parent_mut(point, self.area) {
            parent.switch_subtree_orientations();
        }
    }

    pub fn get_all_leaves(&self, padding: i8) -> Vec<AreaLeaf<T>> {
        self.root.get_all_leaves(self.area.pad_full(padding))
    }

    pub fn find_leaf(&self, point: (i32, i32), padding: i8) -> Option<AreaLeaf<T>> {
        self.root.find_leaf(point, self.area.pad_full(padding))
    }

    pub fn remove(&mut self, point: (i32, i32)) {
        self.root.remove(point, self.area);
    }

    pub(crate) fn resize_ancestor(&mut self, orig_point1: (i32, i32), orig_point2: (i32, i32), grow_ratio: i32) {
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
        }
    }

    pub(crate) fn swap_ids(&mut self, point1: (i32, i32), point2: (i32, i32)) {
        let id1 = self.root.get_id(point1, self.area);
        if id1.is_none() {
            return;
        }

        let id2 = self.root.set_id(id1.unwrap(), point2, self.area);

        if id2.is_none() {
            return;
        }
        self.root.set_id(id2.unwrap(), point1, self.area);
    }

    pub fn replace_id(&mut self, point: (i32, i32), id: T) -> Option<T> {
        self.root.set_id(id, point, self.area)
    }
}

impl<T: Debug + Copy> Debug for AreaTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.root)
    }
}
