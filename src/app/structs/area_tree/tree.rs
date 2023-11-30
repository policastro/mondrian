use crate::app::structs::{area::Area, orientation::Orientation};

use super::{layout::layout_strategy::AreaTreeLayoutStrategy, leaf::AreaLeaf, node::AreaNode};
use std::fmt::Debug;

pub struct AreaTree<T: Copy, L: AreaTreeLayoutStrategy + Copy> {
    root: AreaNode<T>,
    area: Area,
    layout_strategy: L,
}

impl<T: Copy, L: AreaTreeLayoutStrategy + Copy> AreaTree<T, L> {
    pub fn new(area: Area, layout_strategy: L) -> AreaTree<T, L> {
        let (orientation, ratio) = layout_strategy.get_initial_params();
        AreaTree {
            root: AreaNode::new_internal(orientation, ratio),
            area,
            layout_strategy: layout_strategy,
        }
    }

    pub fn insert(&mut self, id: T) {
        self.root.insert(id, self.area, &mut self.layout_strategy);
    }

    pub fn set_parent_orientation(&mut self, point: (u32, u32), orientation: Orientation) {
        let parent = self.root.find_parent_as_mut(point, self.area);
        if let Some(parent) = parent {
            parent.orientation = orientation;
        }
    }

    pub fn get_all_leaves(&self) -> Vec<AreaLeaf<T>> {
        self.root.get_all_leaves(self.area)
    }

    pub fn find_leaf(&self, point: (u32, u32)) -> Option<AreaLeaf<T>> {
        self.root.find_leaf(point, self.area)
    }

    pub fn remove(&mut self, point: (u32, u32)) {
        self.root.remove(point, self.area);
    }

    pub(crate) fn resize_ancestor(&mut self, orig_point1: (u32, u32), orig_point2: (u32, u32), grow_ratio: i32) {
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

    pub(crate) fn swap_ids(&mut self, point1: (u32, u32), point2: (u32, u32)) {
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

    pub fn replace_id(&mut self, point: (u32, u32), id: T) -> Option<T> {
        self.root.set_id(id, point, self.area)
    }
}

impl<T: Debug + Copy, L: AreaTreeLayoutStrategy + Copy> Debug for AreaTree<T, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:#?}", self.root)
    }
}

/*
#[cfg(test)]
mod tests {
    use crate::app::structs::{area::Area, area_tree::node::AreaNode};

    use super::AreaTree;

    use std::{
        collections::VecDeque,
        fmt::{Debug, Display},
    };

    impl<T: Clone + Debug + Display + Copy> AreaTree<T> {
        fn lx(&self) -> &AreaNode<T> {
            self.root.lx()
        }

        fn rx(&self) -> &AreaNode<T> {
            self.root.rx()
        }

        fn get_height(node: &AreaNode<T>) -> u8 {
            if node.is_leaf(false) {
                return 1;
            }

            let left_depth = match &node.left {
                Some(l) => Self::get_height(l),
                None => 0,
            };
            let right_depth = match &node.right {
                Some(r) => Self::get_height(r),
                None => 0,
            };

            1 + std::cmp::max(left_depth, right_depth)
        }

        fn breadth_first_traversal(&self) {
            let mut queue: VecDeque<Option<&AreaNode<T>>> = VecDeque::new();

            let height = Self::get_height(&self.root);
            let mut tree_matrix: Vec<Vec<Option<T>>> = Vec::with_capacity(height as usize);

            queue.push_back(Some(&self.root));
            let mut iteration = 0;
            let mut split_factor = 1;
            let mut curr_height = 0;

            tree_matrix.push(vec![None; split_factor]);
            while !queue.is_empty() {
                let node = queue.pop_front().unwrap();

                if iteration % split_factor == 0 && iteration != 0 {
                    iteration = 0;
                    curr_height += 1;
                    split_factor *= 2;
                    tree_matrix.push(vec![None; split_factor]);
                }

                iteration += 1;

                if node.is_none() {
                    tree_matrix[curr_height as usize][iteration - 1] = None;
                    continue;
                }

                let node = node.unwrap();
                tree_matrix[curr_height as usize][iteration - 1] = if node.id.is_some() {
                    Some(node.id.unwrap())
                } else {
                    None
                };

                queue.push_back(node.left.as_deref());
                queue.push_back(node.right.as_deref());
            }

            for (i, row) in tree_matrix.iter().enumerate() {
                for (j, col) in row.iter().enumerate() {
                    match j as u32 == u32::pow(2, i as u32) / 2 && i != 0 {
                        true => print!("|| "),
                        false => (),
                    }

                    match col {
                        Some(id) => print!("({})", id),
                        None => print!("(x)"),
                    }

                    print!(" ");
                }
                println!();
            }
        }
    }

    impl<T: Copy> AreaNode<T> {
        pub fn lx(&self) -> &AreaNode<T> {
            self.left.as_deref().unwrap()
        }

        fn rx(&self) -> &AreaNode<T> {
            self.right.as_deref().unwrap()
        }
    }

    #[test]
    fn insert_test() {
        let mut tree = super::AreaTree::new(Area::new(0, 0, 500, 500), super::Orientation::Vertical, 50);
        tree.insert(1, (0, 0));
        assert_eq!(tree.root.id.unwrap(), 1);

        tree.insert(2, (200, 200));
        assert_eq!(tree.rx().id.unwrap(), 1);
        assert_eq!(tree.lx().id.unwrap(), 2);

        tree.insert(3, (300, 300));
        assert_eq!(tree.rx().lx().id.unwrap(), 1);
        assert_eq!(tree.rx().rx().id.unwrap(), 3);
        assert_eq!(tree.lx().id.unwrap(), 2);

        tree.insert(4, (450, 450));
        assert_eq!(tree.rx().lx().id.unwrap(), 1);
        assert_eq!(tree.rx().rx().lx().id.unwrap(), 3);
        assert_eq!(tree.rx().rx().rx().id.unwrap(), 4);
        assert_eq!(tree.lx().id.unwrap(), 2);

        tree.insert(5, (100, 100));
        assert_eq!(tree.rx().lx().id.unwrap(), 1);
        assert_eq!(tree.rx().rx().lx().id.unwrap(), 3);
        assert_eq!(tree.rx().rx().rx().id.unwrap(), 4);
        assert_eq!(tree.lx().lx().id.unwrap(), 5);
        assert_eq!(tree.lx().rx().id.unwrap(), 2);

        tree.insert(6, (175, 175));
        assert_eq!(tree.rx().lx().id.unwrap(), 1);
        assert_eq!(tree.rx().rx().lx().id.unwrap(), 3);
        assert_eq!(tree.rx().rx().rx().id.unwrap(), 4);
        assert_eq!(tree.lx().rx().id.unwrap(), 2);
        assert_eq!(tree.lx().lx().lx().id.unwrap(), 5);
        assert_eq!(tree.lx().lx().rx().id.unwrap(), 6);
    }

    #[test]
    fn find_leaf() {
        let mut tree = super::AreaTree::new(Area::new(0, 0, 500, 500), super::Orientation::Vertical, 50);
        tree.insert(1, (0, 0));
        tree.insert(2, (200, 200));
        tree.insert(3, (300, 300));
        tree.insert(4, (450, 450));
        tree.insert(5, (100, 100));
        tree.insert(6, (175, 175));

        assert_eq!(tree.find_leaf((0, 0)).unwrap().id, 5);
    }

    #[test]
    fn remove_test() {
        let mut tree = super::AreaTree::new(Area::new(0, 0, 500, 500), super::Orientation::Vertical, 50);

        tree.remove((0, 0));
        tree.insert(1, (0, 0));
        tree.breadth_first_traversal();
        tree.remove((0, 0));

        tree.insert(1, (0, 0));
        tree.insert(2, (200, 200));
        tree.insert(3, (300, 300));
        tree.insert(4, (450, 450));
        tree.insert(5, (100, 100));
        tree.insert(6, (175, 175));

        let mut center = Area::new(0, 250, 250, 250).get_center();
        tree.remove(center);

        center = Area::new(250, 250, 125, 250).get_center();
        tree.remove(center);

        assert_eq!(tree.lx().lx().id.unwrap(), 5);
        assert_eq!(tree.lx().rx().id.unwrap(), 6);
        assert_eq!(tree.rx().lx().id.unwrap(), 1);
        assert_eq!(tree.rx().rx().id.unwrap(), 4);
    }
}
*/
