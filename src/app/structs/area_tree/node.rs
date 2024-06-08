use std::fmt::Debug;

use crate::app::structs::{area::Area, direction::Direction, orientation::Orientation};

use super::{
    layout::layout_strategy::{AreaTreeLayoutStrategy, AreaTreeLayoutStrategyEnum},
    leaf::AreaLeaf,
};

pub(super) struct AreaNode<T: Copy> {
    pub orientation: Orientation,
    pub ratio: u8,
    pub left: Option<Box<AreaNode<T>>>,
    pub right: Option<Box<AreaNode<T>>>,
    pub id: Option<T>,
}

impl<T: Copy> AreaNode<T> {
    pub fn new(id: Option<T>, orientation: Orientation, ratio: u8) -> AreaNode<T> {
        AreaNode {
            id,
            orientation,
            ratio,
            left: None,
            right: None,
        }
    }

    pub fn new_internal(orientation: Orientation, ratio: u8) -> AreaNode<T> {
        AreaNode::new(None, orientation, ratio)
    }

    pub fn insert(&mut self, id: T, area: Area, insert_strategy: &mut AreaTreeLayoutStrategyEnum) -> AreaLeaf<T> {
        insert_strategy.reset();
        self.insert_rec(id, area, insert_strategy)
    }

    fn insert_rec(&mut self, id: T, area: Area, insert_strategy: &mut AreaTreeLayoutStrategyEnum) -> AreaLeaf<T> {
        if self.id.is_none() && self.is_leaf() {
            self.id = Some(id);
            return AreaLeaf::new(id, area);
        }

        let (direction, orientation) = insert_strategy.next();

        let (min_area, max_area) = area.split(self.ratio, self.orientation);
        let (main_subtree, cross_subtree, _) = match direction {
            Direction::Left | Direction::Up => (&mut self.left, &mut self.right, min_area),
            Direction::Right | Direction::Down => (&mut self.right, &mut self.left, max_area),
        };

        if main_subtree.is_none() {
            self.orientation = orientation;
            *main_subtree = Some(Box::new(AreaNode::new_internal(orientation, 50)));
            *cross_subtree = Some(Box::new(AreaNode::new(self.id, orientation, 50)));
            self.id = None;
        }

        main_subtree
            .as_mut()
            .expect("This should be impossible")
            .insert_rec(id, area, insert_strategy)
    }

    pub fn find_leaf(&self, point: (i32, i32), area: Area) -> Option<AreaLeaf<T>> {
        let mut curr_node = self;
        let mut curr_area = area;
        while !curr_node.is_leaf() {
            let (min_area, max_area) = curr_node.get_split_area(area);
            let is_left = min_area.contains(point);

            if !is_left && !max_area.contains(point) {
                return None;
            }

            (curr_node, curr_area) = match is_left {
                true => (curr_node.left.as_ref().unwrap(), min_area),
                false => (curr_node.right.as_ref().unwrap(), max_area),
            };
        }

        curr_node.id.map(|id| AreaLeaf::new(id, curr_area))
    }

    pub fn get_all_leaves(&self, area: Area) -> Vec<AreaLeaf<T>> {
        self.get_all_leaves_rec(area)
    }

    fn get_all_leaves_rec(&self, area: Area) -> Vec<AreaLeaf<T>> {
        let mut leaves = Vec::new();

        if self.is_leaf() && self.id.is_some() {
            leaves.push(AreaLeaf::new(self.id.unwrap(), area));
        }

        let (min_area, max_area) = self.get_split_area(area);

        if let Some(left) = &self.left {
            leaves.extend(left.get_all_leaves_rec(min_area));
        }

        if let Some(right) = &self.right {
            leaves.extend(right.get_all_leaves_rec(max_area));
        }

        leaves
    }

    pub fn remove(&mut self, point: (i32, i32), area: Area) {
        if self.is_leaf() {
            self.id = None;
            return;
        }

        let (min_area, max_area) = area.split(self.ratio, self.orientation);
        let is_left = min_area.contains(point);

        let (main_subtree, cross_subtree, curr_area) = match is_left {
            true => (&mut self.left, &mut self.right, min_area),
            false => (&mut self.right, &mut self.left, max_area),
        };

        if main_subtree.is_none() || main_subtree.as_ref().unwrap().is_leaf() {
            *main_subtree = None;
            let mut temp_node = None;

            std::mem::swap(cross_subtree, &mut temp_node);

            if temp_node.as_mut().is_some() {
                std::mem::swap(self, temp_node.as_mut().unwrap());
            }
            return;
        }

        main_subtree.as_mut().unwrap().remove(point, curr_area);
    }

    pub fn find_lowest_common_ancestor(
        &mut self,
        point1: (i32, i32),
        point2: (i32, i32),
        area: Area,
    ) -> Option<&mut AreaNode<T>> {
        if self.is_leaf() {
            return None;
        }

        let mut curr_node = self;
        let mut curr_area = area;
        while !curr_node.is_leaf() {
            let (min_area, max_area) = curr_area.split(curr_node.ratio, curr_node.orientation);
            let is_left = (min_area.contains(point1), min_area.contains(point2));
            if is_left.0 != is_left.1 {
                return Some(curr_node);
            }

            (curr_node, curr_area) = match is_left.0 {
                true => (curr_node.left.as_mut().unwrap(), min_area),
                false => (curr_node.right.as_mut().unwrap(), max_area),
            };
        }

        Some(curr_node)
    }

    pub fn find_parent_as_mut(&mut self, point: (i32, i32), area: Area) -> Option<&mut AreaNode<T>> {
        if self.is_leaf() {
            return None;
        }

        let (min_area, max_area) = self.get_split_area(area);

        if min_area.contains(point) {
            if self.left.as_ref().unwrap().is_leaf() {
                Some(self)
            } else {
                self.left.as_mut().unwrap().find_parent_as_mut(point, min_area)
            }
        } else if self.right.as_ref().unwrap().is_leaf() {
            Some(self)
        } else {
            self.right.as_mut().unwrap().find_parent_as_mut(point, max_area)
        }
    }

    fn find_node_as_mut(&mut self, point: (i32, i32), area: Area) -> &mut AreaNode<T> {
        if self.is_leaf() {
            return self;
        }

        let (min_area, max_area) = self.get_split_area(area);
        let (subtree, curr_area) = match min_area.contains(point) {
            true => (&mut self.left, min_area),
            false => (&mut self.right, max_area),
        };

        subtree
            .as_mut()
            .expect("This should be impossible")
            .find_node_as_mut(point, curr_area)
    }

    fn get_split_area(&self, area: Area) -> (Area, Area) {
        area.split(self.ratio, self.orientation)
    }

    fn find_node(&self, point: (i32, i32), area: Area) -> &AreaNode<T> {
        if self.is_leaf() {
            return self;
        }

        let (min_area, max_area) = area.split(self.ratio, self.orientation);
        let (subtree, curr_area) = match min_area.contains(point) {
            true => (&self.left, min_area),
            false => (&self.right, max_area),
        };

        subtree
            .as_ref()
            .expect("This should be impossible")
            .find_node(point, curr_area)
    }

    pub fn set_id(&mut self, id: T, point: (i32, i32), area: Area) -> Option<T> {
        let node = self.find_node_as_mut(point, area);
        let prev_id = node.id;
        node.id = Some(id);
        prev_id
    }

    pub fn get_id(&self, point: (i32, i32), area: Area) -> Option<T> {
        self.find_node(point, area).id
    }

    pub fn is_leaf(&self) -> bool {
        self.left.is_none() || self.right.is_none()
    }
}

impl<T: Debug + Copy> Debug for AreaNode<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WindowsTree")
            .field("id", &self.id)
            .field("orientation", &format!("{:?} | {:?}%", self.orientation, self.ratio))
            .field("left", &self.left)
            .field("right", &self.right)
            .finish()
    }
}