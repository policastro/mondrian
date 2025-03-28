use super::layout_strategy::LayoutStrategy;
use super::layout_strategy::LayoutStrategyEnum;
use super::layout_strategy::TreeOperation;
use super::leaf::AreaLeaf;
use super::node::AreaNode;
use crate::app::structs::area::Area;
use crate::app::structs::orientation::Orientation;
use crate::win32::window::window_ref::WindowRef;
use std::{collections::HashSet, fmt::Debug, hash::Hash};

pub type WinTree = AreaTree<WindowRef>;

#[derive(Clone)]
pub struct AreaTree<T: Copy + Clone + Eq + Hash> {
    root: AreaNode<T>,
    area: Area,
    strategy: LayoutStrategyEnum,
    ids_map: std::collections::HashMap<T, AreaLeaf<T>>,
}

impl<T: Copy + Clone + Eq + Hash + Debug> AreaTree<T> {
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
        self.strategy.init(self.ids_map.len() as u8, TreeOperation::Insert);
        self.root.insert(id, self.area, &mut self.strategy);
        self.update_map();
    }

    pub fn insert_at(&mut self, id: T, point: (i32, i32)) {
        if self.ids_map.contains_key(&id) {
            return;
        }
        self.root.insert_at(id, point, self.area, 20);
        self.update_map();
    }

    pub fn move_to(&mut self, id: T, point: (i32, i32)) {
        if let Some(leaf) = self.ids_map.remove(&id) {
            self.root.insert_at(id, point, self.area, 20);
            // WARNING: not sure if this is correct
            self.remove_at(leaf.viewbox.get_center());
            self.update_map();
        }
    }

    pub fn switch_subtree_orientations(&mut self, point: (i32, i32)) {
        if let Some(parent) = self.root.find_parent_mut(point, self.area) {
            parent.switch_subtree_orientations();
            self.update_map();
        }
    }

    pub fn leaves(&self, padding: i16, ignored_wins: Option<&HashSet<T>>) -> Vec<AreaLeaf<T>> {
        self.root.leaves(self.area.pad_full(padding), ignored_wins)
    }

    pub fn find_leaf(&self, id: T, padding: i16) -> Option<AreaLeaf<T>> {
        let leaf = self.ids_map.get(&id)?;
        if padding == 0 {
            return Some(*leaf);
        }

        self.root
            .find_leaf(leaf.viewbox.get_center(), self.area.pad_full(padding))
    }

    pub fn find_leaf_at(&self, point: (i32, i32), padding: i16) -> Option<AreaLeaf<T>> {
        self.root.find_leaf(point, self.area.pad_full(padding))
    }

    pub fn remove(&mut self, id: T) {
        if let Some(leaf) = self.ids_map.remove(&id) {
            self.remove_at(leaf.viewbox.get_center());
        }
    }

    pub fn remove_at(&mut self, point: (i32, i32)) {
        self.strategy.init(self.ids_map.len() as u8, TreeOperation::Remove);
        self.root.remove(point, self.area, &mut self.strategy);
        self.update_map();
    }

    pub(crate) fn resize_ancestor(
        &mut self,
        orig_point1: (i32, i32),
        orig_point2: (i32, i32),
        growth: i32,
        clamp_values: Option<(u8, u8)>,
    ) {
        let growth = growth.clamp(i16::MIN.into(), i16::MAX.into()) as i16;
        let clamp_values = clamp_values.unwrap_or((0, 100));
        assert!(clamp_values.0 <= 100 && clamp_values.1 <= 100);
        assert!(clamp_values.0 <= clamp_values.1);
        let ancestor = self
            .root
            .find_lowest_common_ancestor(orig_point1, orig_point2, self.area);

        if let Some((node, area)) = ancestor {
            let min_area = area.split(node.ratio, node.orientation).0;
            let min_area_pad = min_area.pad((0, -growth), (0, -growth));

            let new_ratio = match node.orientation {
                Orientation::Horizontal => (min_area_pad.height as f32 / area.height as f32) * 100f32,
                Orientation::Vertical => (min_area_pad.width as f32 / area.width as f32) * 100f32,
            };

            node.ratio = new_ratio.clamp(clamp_values.0 as f32, clamp_values.1 as f32) as u8;
            self.update_map();
        }
    }

    pub(crate) fn swap_ids(&mut self, id1: T, id2: T) {
        let (p1, p2) = (self.ids_map.get(&id1), self.ids_map.get(&id2));
        if let (Some(l1), Some(l2)) = (p1, p2) {
            self.swap_ids_at(l1.viewbox.get_center(), l2.viewbox.get_center());
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

    pub fn replace_id(&mut self, id: T, new_id: T) -> Option<T> {
        let point = self.ids_map.get(&id)?.viewbox.get_center();
        self.replace_id_at(point, new_id)
    }

    pub fn replace_id_at(&mut self, point: (i32, i32), id: T) -> Option<T> {
        let v = self.root.set_id(id, point, self.area);
        self.update_map();
        v
    }

    pub fn has(&self, id: T) -> bool {
        self.ids_map.contains_key(&id)
    }

    pub fn get_ids(&self) -> Vec<T> {
        self.ids_map.keys().cloned().collect()
    }

    pub fn set_area(&mut self, area: Area) {
        self.area = area;
        self.update_map();
    }

    pub fn get_area(&self) -> Area {
        self.area
    }

    fn update_map(&mut self) {
        self.ids_map = std::collections::HashMap::new();
        self.root.leaves(self.area, None).iter().for_each(|leaf| {
            self.ids_map.insert(leaf.id, *leaf);
        })
    }

    pub fn clear(&mut self) {
        self.ids_map.clear();
        self.root = AreaNode::new(None, Orientation::Horizontal, 50);
    }
}

impl<T: Debug + Copy + Eq + Hash> Debug for AreaTree<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.root)
    }
}
