use crate::{app::structs::area::Area, win32::api::monitor::Monitor};


#[derive(Debug, Clone, Copy)]
pub struct ManagedMonitor {
    pub id: isize,
    pub workarea: Area,
}

impl From<&Monitor> for ManagedMonitor {
    fn from(monitor: &Monitor) -> Self {
        let workarea = Area::new(
            monitor.offset.0,
            monitor.offset.1,
            u16::try_from(monitor.workspace.0).expect("Failed to convert i32 to u16"),
            u16::try_from(monitor.workspace.1).expect("Failed to convert i32 to u16"),
        );
        Self {
            id: monitor.id,
            workarea,
        }
    }
}