use crate::app::structs::area::Area;

#[derive(Debug, Clone, Copy)]
pub struct AreaLeaf<T> {
    pub id: T,
    pub viewbox: Area,
}

impl<T> AreaLeaf<T> {
    pub fn new(id: T, viewbox: Area) -> AreaLeaf<T> {
        AreaLeaf { id, viewbox }
    }
}
