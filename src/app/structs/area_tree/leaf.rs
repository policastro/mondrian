use crate::app::structs::area::Area;

#[derive(Debug, Clone)]
pub struct AreaLeaf<T> {
    pub id: T,
    pub viewbox: Area,
    pub orig_viewbox: Area,
}

impl<T> AreaLeaf<T> {
    pub fn new(id: T, viewbox: Area, orig_viewbox: Area) -> AreaLeaf<T> {
        AreaLeaf {
            id,
            viewbox,
            orig_viewbox,
        }
    }
}
