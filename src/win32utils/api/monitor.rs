use windows::Win32::{
    Foundation::{LPARAM, RECT},
    Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFO},
    UI::WindowsAndMessaging::MONITORINFOF_PRIMARY,
};

#[derive(Debug)]
pub struct Monitor {
    pub id: isize,
    pub primary: bool,
    pub resolution: (i32, i32),
    pub workspace: (i32, i32),
    pub offset: (i32, i32),
}

pub fn get_monitor_info(monitor: HMONITOR) -> MONITORINFO {
    unsafe {
        let mut monitor_info: MONITORINFO = std::mem::zeroed();
        monitor_info.cbSize = std::mem::size_of::<MONITORINFO>() as u32;
        let _ = GetMonitorInfoW(monitor, &mut monitor_info as *mut MONITORINFO);
        monitor_info
    }
}

unsafe extern "system" fn enum_monitors_callback(
    monitor: HMONITOR,
    _hdc: HDC,
    lprc_monitor: *mut RECT,
    data: LPARAM,
) -> windows::Win32::Foundation::BOOL {
    let monitors = unsafe { &mut *(data.0 as *mut Vec<Monitor>) };
    let monitor_info: MONITORINFO = get_monitor_info(monitor);

    monitors.push(Monitor {
        id: monitor.0,
        primary: monitor_info.dwFlags & MONITORINFOF_PRIMARY != 0,
        resolution: (
            monitor_info.rcMonitor.right - monitor_info.rcMonitor.left,
            monitor_info.rcMonitor.bottom - monitor_info.rcMonitor.top,
        ),
        workspace: (
            monitor_info.rcWork.right - monitor_info.rcWork.left,
            monitor_info.rcWork.bottom - monitor_info.rcWork.top,
        ),
        offset: match lprc_monitor.as_ref() {
            None => (0, 0),
            Some(lprc_monitor) => (lprc_monitor.left, lprc_monitor.top),
        },
    });
    true.into()
}

pub fn enum_display_monitors() -> Vec<Monitor> {
    let mut monitors: Vec<Monitor> = Vec::new();

    unsafe {
        let _ = EnumDisplayMonitors(
            None,
            None,
            Some(enum_monitors_callback),
            LPARAM(&mut monitors as *mut Vec<Monitor> as isize),
        );
    }

    monitors
}

