use crate::app::structs::area::Area;

use super::node::AreaNodeMarker;

#[derive(Debug, Clone, Copy)]
pub struct AreaLeaf<T> {
    pub id: T,
    pub viewbox: Area,
    pub marker: AreaNodeMarker,
}

impl<T> AreaLeaf<T> {
    pub fn new(id: T, viewbox: Area, marker: AreaNodeMarker) -> AreaLeaf<T> {
        AreaLeaf { id, viewbox, marker }
    }
}
