use crate::
    app::structs::{
        area::Area,
        area_tree::{layout_strategy::LayoutStrategyEnum, tree::WinTree},
    }
;

#[derive(Debug)]
pub(super) struct Container {
    pub tree: WinTree,
    pub workarea: Area,
    pub orig_workarea: Area,
}

impl Container {
    pub fn new(workarea: Area, layout: LayoutStrategyEnum, padding: i16) -> Self {
        Container {
            tree: WinTree::new(workarea, layout.clone()),
            workarea: workarea.pad_full(padding),
            orig_workarea: workarea,
        }
    }

    pub fn contains(&self, point: (i32, i32)) -> bool {
        self.workarea.contains(point)
    }
}
