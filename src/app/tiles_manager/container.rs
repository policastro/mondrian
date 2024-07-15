use super::managed_monitor::ManagedMonitor;
use crate::{
    app::structs::{
        area::Area,
        area_tree::{layout::layout_strategy::AreaTreeLayoutStrategyEnum, tree::AreaTree},
    },
    win32::api::monitor::Monitor,
};

#[derive(Debug)]
pub(super) struct Container {
    pub monitor: ManagedMonitor,
    pub tree: AreaTree<isize>,
    pub workarea: Area,
}

impl Container {
    pub fn new(monitor: Monitor, layout: AreaTreeLayoutStrategyEnum, padding: i8) -> Self {
        let managed_monitor = ManagedMonitor::from(&monitor);
        Container {
            monitor: managed_monitor,
            tree: AreaTree::new(managed_monitor.workarea, layout),
            workarea: managed_monitor.workarea.pad_full(padding),
        }
    }

    pub fn contains(&self, point: (i32, i32)) -> bool {
        self.workarea.contains(point)
    }
}
