use windows::Win32::{
    Foundation::LPARAM,
    Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HMONITOR, MONITORINFOEXW},
};

use crate::win32::callbacks::enum_monitors::enum_monitors_callback;


#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Monitor {
    pub id: isize,
    pub name: String,
    pub primary: bool,
    pub resolution: (i32, i32),
    pub workspace: (i32, i32),
    pub offset: (i32, i32),
}

pub fn get_monitor_info(monitor: HMONITOR) -> MONITORINFOEXW {
    let mut info: MONITORINFOEXW = unsafe { std::mem::zeroed() };
    info.monitorInfo.cbSize = std::mem::size_of::<MONITORINFOEXW>() as u32;
    let info_ptr = <*mut _>::cast(&mut info);

    unsafe { GetMonitorInfoW(monitor, info_ptr).expect("GetMonitorInfo failed") };
    info
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
