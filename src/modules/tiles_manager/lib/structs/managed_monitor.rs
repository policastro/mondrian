use crate::win32::{
    api::monitor::Monitor,
    window::{ghost_window::GhostWindow, window_obj::WindowObjHandler, window_ref::WindowRef},
};

pub struct ManagedMonitor {
    pub info: Monitor,
    anchor: GhostWindow,
}

impl From<Monitor> for ManagedMonitor {
    fn from(monitor: Monitor) -> Self {
        let anchor = GhostWindow::create(monitor.monitor_area);
        ManagedMonitor { info: monitor, anchor }
    }
}

impl ManagedMonitor {
    pub fn focus(&self) {
        let _ = WindowRef::try_from(&self.anchor).inspect(|w| w.focus());
    }
}
