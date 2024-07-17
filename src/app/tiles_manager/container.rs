use super::managed_monitor::ManagedMonitor;
use crate::{
    app::structs::{
        area::Area,
        area_tree::{layout_strategy::LayoutStrategyEnum, tree::WinTree},
    },
    win32::api::monitor::Monitor,
};

#[derive(Debug)]
pub(super) struct Container {
    pub monitor: ManagedMonitor,
    pub tree: WinTree,
    pub workarea: Area,
}

impl Container {
    pub fn new(monitor: Monitor, layout: LayoutStrategyEnum, padding: i8) -> Self {
        let managed_monitor = ManagedMonitor::from(&monitor);
        Container {
            monitor: managed_monitor,
            tree: WinTree::new(managed_monitor.workarea, layout),
            workarea: managed_monitor.workarea.pad_full(padding),
        }
    }

    pub fn contains(&self, point: (i32, i32)) -> bool {
        self.workarea.contains(point)
    }
}
