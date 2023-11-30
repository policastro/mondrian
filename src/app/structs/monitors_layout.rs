use crate::win32utils::api::monitor::Monitor;
use super::area::Area;



#[derive(Debug)]
pub struct ManagedMonitor {
    pub id: isize,
    pub workarea: Area,
}

impl From<&Monitor> for ManagedMonitor {
    fn from(monitor: &Monitor) -> Self {
        let workarea = Area::new(
            u32::try_from(monitor.offset.0).expect("Failed to convert i32 to u32"),
            u32::try_from(monitor.offset.1).expect("Failed to convert i32 to u32"),
            u32::try_from(monitor.workspace.0).expect("Failed to convert i32 to u32"),
            u32::try_from(monitor.workspace.1).expect("Failed to convert i32 to u32"),
        );
        Self {
            id: monitor.id,
            workarea,
        }
    }
}

pub struct MonitorsLayout {
    monitors: Vec<ManagedMonitor>,
}

impl MonitorsLayout {
    pub fn new(monitors: Vec<Monitor>) -> Self {
        MonitorsLayout {
            monitors: monitors.iter().map(ManagedMonitor::from).collect(),
        }
    }

    pub fn which(&self, point: (u32, u32)) -> Option<isize> {
        let monitor = self.monitors.iter().find(|m| m.workarea.contains(point));
        monitor.map(|m| m.id)
    }

    pub fn get_monitors(&self) -> Vec<&ManagedMonitor> {
        self.monitors.iter().collect()
    }
}
