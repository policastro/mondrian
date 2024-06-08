use crate::{app::structs::area_tree::{layout::layout_strategy::AreaTreeLayoutStrategyEnum, tree::AreaTree}, win32::api::monitor::Monitor};
use super::managed_monitor::ManagedMonitor;

pub(super) struct Container {
    pub monitor: ManagedMonitor,
    pub tree: AreaTree<isize>,
}

impl Container {
    pub fn new(monitor: Monitor, layout: AreaTreeLayoutStrategyEnum) -> Self {
        let managed_monitor = ManagedMonitor::from(&monitor);
        Container {
            monitor: managed_monitor,
            tree: AreaTree::new(managed_monitor.workarea, layout),
        }
    }

    pub fn contains(&self, point: (i32, i32)) -> bool {
        self.monitor.workarea.contains(point)
    }
}
