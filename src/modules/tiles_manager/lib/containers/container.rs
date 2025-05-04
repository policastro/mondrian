use std::fmt::Debug;

use crate::app::area_tree::tree::WinTree;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContainerLayer {
    Normal,
    Focalized,
    HalfFocalized,
}

impl ContainerLayer {
    pub fn is_focalized_or_half(&self) -> bool {
        matches!(self, ContainerLayer::Focalized | ContainerLayer::HalfFocalized)
    }
}

pub struct Container {
    current: ContainerLayer,
    normal: WinTree,
    focalized: WinTree,
    half_focalized: WinTree,
}

impl Container {
    pub fn new(normal: WinTree, focalized: WinTree, half_focalized: WinTree) -> Self {
        Self {
            current: ContainerLayer::Normal,
            normal,
            focalized,
            half_focalized,
        }
    }

    pub fn tree(&self) -> &WinTree {
        match self.current {
            ContainerLayer::Normal => &self.normal,
            ContainerLayer::Focalized => &self.focalized,
            ContainerLayer::HalfFocalized => &self.half_focalized,
        }
    }

    pub fn tree_mut(&mut self) -> &mut WinTree {
        match self.current {
            ContainerLayer::Normal => &mut self.normal,
            ContainerLayer::Focalized => &mut self.focalized,
            ContainerLayer::HalfFocalized => &mut self.half_focalized,
        }
    }

    pub fn get_tree(&self, ct: ContainerLayer) -> &WinTree {
        match ct {
            ContainerLayer::Normal => &self.normal,
            ContainerLayer::Focalized => &self.focalized,
            ContainerLayer::HalfFocalized => &self.half_focalized,
        }
    }

    pub fn get_tree_mut(&mut self, ct: ContainerLayer) -> &mut WinTree {
        match ct {
            ContainerLayer::Normal => &mut self.normal,
            ContainerLayer::Focalized => &mut self.focalized,
            ContainerLayer::HalfFocalized => &mut self.half_focalized,
        }
    }

    pub fn trees_mut(&mut self) -> [(ContainerLayer, &mut WinTree); 3] {
        [
            (ContainerLayer::Normal, &mut self.normal),
            (ContainerLayer::Focalized, &mut self.focalized),
            (ContainerLayer::HalfFocalized, &mut self.half_focalized),
        ]
    }

    pub fn current(&self) -> ContainerLayer {
        self.current
    }

    pub fn set_current(&mut self, current: ContainerLayer) {
        self.current = current;
    }
}
