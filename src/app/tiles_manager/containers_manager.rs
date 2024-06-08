use crate::app::structs::area_tree::tree::AreaTree;

use super::container::Container;

pub(super) struct ContainersManager {
    containers: Vec<Container>,
}

impl ContainersManager {
    pub fn new(containers: Vec<Container>) -> Self {
        ContainersManager { containers }
    }

    pub fn is_same_container(&self, point1: (i32, i32), point2: (i32, i32)) -> bool {
        let c1 = self.which(point1);
        let c2 = self.which(point2);

        match (c1, c2) {
            (Some(c1), Some(c2)) => c1.monitor.id == c2.monitor.id,
            _ => false,
        }
    }

    pub fn which(&self, point: (i32, i32)) -> Option<&Container> {
        self.containers.iter().find(|c| c.contains(point))
    }

    pub fn which_mut(&mut self, point: (i32, i32)) -> Option<&mut Container> {
        self.containers.iter_mut().find(|c| c.contains(point))
    }

    pub fn which_tree(&mut self, point: (i32, i32)) -> Option<&mut AreaTree<isize>> {
        self.which_mut(point).map(|c| &mut c.tree)
    }

    pub fn get_containers(&mut self) -> Vec<&mut Container> {
        self.containers.iter_mut().collect()
    }
}
