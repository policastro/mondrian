use super::layout_strategy::LayoutStrategy;
use super::layout_strategy::LayoutStrategyEnum;
use super::leaf::AreaLeaf;
use crate::app::structs::area::Area;
use crate::app::structs::direction::Direction;
use crate::app::structs::orientation::Orientation;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Clone)]
pub(super) struct AreaNode<T: Copy + Clone> {
    pub orientation: Orientation,
    pub ratio: u8,
    pub left: Option<Box<AreaNode<T>>>,
    pub right: Option<Box<AreaNode<T>>>,
    pub id: Option<T>,
    pub marker: AreaNodeMarker,
}

#[derive(Debug, Copy, Clone)]
pub enum AreaNodeMarker {
    None,
    Deleted,
}

type Marker = AreaNodeMarker;
impl<T: Copy> AreaNode<T> {
    pub fn new(id: Option<T>, orientation: Orientation, ratio: u8, marker: Marker) -> AreaNode<T> {
        AreaNode {
            id,
            orientation,
            ratio,
            left: None,
            right: None,
            marker,
        }
    }

    pub fn root() -> AreaNode<T> {
        AreaNode::new(None, Orientation::Horizontal, 50, Marker::None)
    }

    pub fn insert(&mut self, id: T, area: Area, strategy: &mut LayoutStrategyEnum) {
        if self.id.is_none() && self.is_leaf() {
            strategy.complete();
            self.id = Some(id);
            return;
        }

        let (direction, orientation, ratio) = strategy.next();

        let (min_area, max_area) = area.split(self.ratio, self.orientation);
        let (main_tree, cross_tree, area) = match direction {
            Direction::Left | Direction::Up => (&mut self.left, &mut self.right, min_area),
            Direction::Right | Direction::Down => (&mut self.right, &mut self.left, max_area),
        };

        self.orientation = orientation.unwrap_or(self.orientation);
        self.ratio = ratio.unwrap_or(self.ratio);

        if main_tree.is_none() {
            let (orientation, ratio) = strategy.complete();
            *main_tree = Some(Box::new(AreaNode::new(Some(id), orientation, ratio, Marker::None)));
            *cross_tree = Some(Box::new(AreaNode::new(self.id, orientation, ratio, self.marker)));
            (self.orientation, self.ratio, self.id) = (orientation, ratio, None);
            return;
        }

        main_tree.as_mut().unwrap().insert(id, area, strategy)
    }

    pub fn mark_leaf_at(&mut self, point: (i32, i32), area: Area, marker: AreaNodeMarker) {
        let node = self.find_node_mut(point, area);
        if node.is_leaf() && node.id.is_some() {
            node.marker = marker;
        }
    }

    pub fn insert_at(&mut self, id: T, point: (i32, i32), area: Area, vertical_limit: u8) -> bool {
        assert!(vertical_limit <= 50);
        if self.id.is_none() && self.is_leaf() {
            self.id = Some(id);
            return true;
        }

        let mut curr_node = self;
        let mut curr_area = area;
        while !curr_node.is_leaf() {
            let (min_area, max_area) = curr_area.split(curr_node.ratio, curr_node.orientation);
            (curr_node, curr_area) = match (min_area.contains(point), max_area.contains(point)) {
                (true, _) => (curr_node.left.as_mut().unwrap(), min_area),
                (_, true) => (curr_node.right.as_mut().unwrap(), max_area),
                (false, false) => return false,
            };
        }

        let (up_area, _) = curr_area.split(vertical_limit, Orientation::Horizontal);
        let (_, down_area) = curr_area.split(100 - vertical_limit, Orientation::Horizontal);
        let (left_area, _) = curr_area.split(50, Orientation::Vertical);

        let orient = match up_area.contains(point) || down_area.contains(point) {
            true => Orientation::Horizontal,
            false => Orientation::Vertical,
        };

        let (curr_props, new_props) = ((curr_node.id, curr_node.marker), (Some(id), Marker::None));
        let (left, right) = if up_area.contains(point) {
            (new_props, curr_props)
        } else if down_area.contains(point) {
            (curr_props, new_props)
        } else if left_area.contains(point) {
            (new_props, curr_props)
        } else {
            (curr_props, new_props)
        };

        curr_node.left = Some(Box::new(AreaNode::new(left.0, orient, 50, left.1)));
        curr_node.right = Some(Box::new(AreaNode::new(right.0, orient, 50, right.1)));
        (curr_node.orientation, curr_node.ratio, curr_node.id, curr_node.marker) = (orient, 50, None, Marker::None);

        true
    }

    pub fn find_leaf(&self, point: (i32, i32), area: Area) -> Option<AreaLeaf<T>> {
        let mut curr_node = self;
        let mut curr_area = area;
        while !curr_node.is_leaf() {
            let (min_area, max_area) = curr_node.get_split_area(curr_area);
            let is_left = min_area.contains(point);

            if !is_left && !max_area.contains(point) {
                return None;
            }

            (curr_node, curr_area) = match is_left {
                true => (curr_node.left.as_ref().unwrap(), min_area),
                false => (curr_node.right.as_ref().unwrap(), max_area),
            };
        }
        curr_node
            .id
            .map(|id| AreaLeaf::new(id, curr_area, AreaNodeMarker::None))
    }

    pub fn switch_subtree_orientations(&mut self) {
        let mut leaves = vec![self];
        while let Some(node) = leaves.pop() {
            if !node.is_leaf() {
                node.orientation = node.orientation.opposite();
            }
            if let Some(left) = &mut node.left {
                leaves.push(left);
            }
            if let Some(right) = &mut node.right {
                leaves.push(right);
            }
        }
    }

    pub fn remove(&mut self, point: (i32, i32), area: Area, strategy: &mut LayoutStrategyEnum) {
        if self.is_leaf() {
            strategy.complete();
            self.id = None;
            return;
        }

        let (min_area, max_area) = area.split(self.ratio, self.orientation);
        let is_left = min_area.contains(point);

        let (main_tree, cross_tree, curr_area) = match is_left {
            true => (&mut self.left, &mut self.right, min_area),
            false => (&mut self.right, &mut self.left, max_area),
        };

        let (_, orientation, ratio) = strategy.next();
        self.orientation = orientation.unwrap_or(self.orientation);
        self.ratio = ratio.unwrap_or(self.ratio);

        if main_tree.is_none() || main_tree.as_ref().is_some_and(|t| t.is_leaf()) {
            *main_tree = None;
            let mut temp_node = None;

            std::mem::swap(cross_tree, &mut temp_node);

            if temp_node.as_mut().is_some() {
                std::mem::swap(self, temp_node.as_mut().unwrap());
            }
            strategy.complete();
            return;
        }

        main_tree.as_mut().unwrap().remove(point, curr_area, strategy)
    }

    pub fn find_lowest_common_ancestor(
        &mut self,
        point1: (i32, i32),
        point2: (i32, i32),
        area: Area,
    ) -> Option<(&mut AreaNode<T>, Area)> {
        if self.is_leaf() {
            return None;
        }

        let mut curr_node = self;
        let mut curr_area = area;
        while !curr_node.is_leaf() {
            let (min_area, max_area) = curr_area.split(curr_node.ratio, curr_node.orientation);
            let is_left = (min_area.contains(point1), min_area.contains(point2));
            if is_left.0 != is_left.1 {
                return Some((curr_node, curr_area));
            }

            (curr_node, curr_area) = match is_left.0 {
                true => (curr_node.left.as_mut().unwrap(), min_area),
                false => (curr_node.right.as_mut().unwrap(), max_area),
            };
        }

        Some((curr_node, curr_area))
    }

    pub fn find_parent_mut(&mut self, point: (i32, i32), area: Area) -> Option<&mut AreaNode<T>> {
        if self.is_leaf() {
            return None;
        }

        let (min_area, max_area) = self.get_split_area(area);

        if min_area.contains(point) {
            if self.left.as_ref().unwrap().is_leaf() {
                Some(self)
            } else {
                self.left.as_mut().unwrap().find_parent_mut(point, min_area)
            }
        } else if self.right.as_ref().unwrap().is_leaf() {
            Some(self)
        } else {
            self.right.as_mut().unwrap().find_parent_mut(point, max_area)
        }
    }

    fn find_node_mut(&mut self, point: (i32, i32), area: Area) -> &mut AreaNode<T> {
        if self.is_leaf() {
            return self;
        }

        let (min_area, max_area) = self.get_split_area(area);
        let (subtree, curr_area) = match min_area.contains(point) {
            true => (&mut self.left, min_area),
            false => (&mut self.right, max_area),
        };

        subtree.as_mut().unwrap().find_node_mut(point, curr_area)
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

        subtree.as_ref().unwrap().find_node(point, curr_area)
    }

    pub fn set_id(&mut self, id: T, point: (i32, i32), area: Area) -> Option<T> {
        let node = self.find_node_mut(point, area);
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

impl<T: Copy + Eq + Hash + Debug> AreaNode<T> {
    pub fn leaves(&self, area: Area, ignored_ids: Option<&HashSet<T>>) -> Vec<AreaLeaf<T>> {
        let mut leaves = Vec::new();

        let leaves_counts = match ignored_ids.is_some() {
            true => {
                let mut leaves_counts = HashMap::new();
                self.get_leaves_counts(&mut leaves_counts, ignored_ids);
                Some(leaves_counts)
            }
            false => None,
        };

        let mut stack = vec![(self, area)];

        while let Some((node, area)) = stack.pop() {
            if node.is_leaf() {
                if let Some(id) = node.id.filter(|&id| !Self::is_ignored(ignored_ids, id)) {
                    leaves.push(AreaLeaf::new(id, area, node.marker));
                }
                continue;
            }

            let (min_area, max_area) = node.get_split_area(area);
            let (lx, rx) = (node.left.as_deref().unwrap(), node.right.as_deref().unwrap());
            let (lx_leaves, rx_leaves) = match leaves_counts {
                Some(ref counts) => (
                    counts.get(&(lx as *const AreaNode<T>)).copied().unwrap_or(0),
                    counts.get(&(rx as *const AreaNode<T>)).copied().unwrap_or(0),
                ),
                _ => (1, 1),
            };

            let (is_lx_ignored, is_rx_ignored) = (
                lx.id.is_none_or(|id| Self::is_ignored(ignored_ids, id)),
                rx.id.is_none_or(|id| Self::is_ignored(ignored_ids, id)),
            );

            let lx_area = match rx_leaves > 0 || !is_rx_ignored {
                true => min_area,
                _ => area,
            };

            let rx_area = match lx_leaves > 0 || !is_lx_ignored {
                true => max_area,
                _ => area,
            };

            stack.push((rx, rx_area));
            stack.push((lx, lx_area));
        }

        leaves
    }

    fn get_leaves_counts(
        &self,
        leaf_counts: &mut HashMap<*const AreaNode<T>, usize>,
        ignored_ids: Option<&HashSet<T>>,
    ) -> usize {
        if self.is_leaf() {
            let ignored = self.id.is_none_or(|id| Self::is_ignored(ignored_ids, id));
            return if ignored { 0 } else { 1 };
        }

        let left_leaves = self.left.as_ref().unwrap().get_leaves_counts(leaf_counts, ignored_ids);
        let right_leaves = self.right.as_ref().unwrap().get_leaves_counts(leaf_counts, ignored_ids);

        let total_leaves = left_leaves + right_leaves;

        // INFO: using the node address as key
        leaf_counts.insert(self as *const AreaNode<T>, total_leaves);

        total_leaves
    }

    fn is_ignored(ignore_ids: Option<&HashSet<T>>, id: T) -> bool {
        ignore_ids.is_some_and(|ids| ids.contains(&id))
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
