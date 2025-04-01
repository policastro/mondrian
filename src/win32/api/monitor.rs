use crate::app::structs::area::Area;
use crate::win32::callbacks::enum_monitors::enum_monitors_callback;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Graphics::Gdi::EnumDisplayMonitors;
use windows::Win32::Graphics::Gdi::GetMonitorInfoW;
use windows::Win32::Graphics::Gdi::HMONITOR;
use windows::Win32::Graphics::Gdi::MONITORINFOEXW;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct Monitor {
    pub handle: isize,
    pub id: String,
    pub hw_id: String,
    pub primary: bool,
    pub resolution: (i32, i32),
    pub workspace: (i32, i32),
    pub monitor_area: Area,
    pub workspace_area: Area,
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

    monitors.sort_by(|a, b| a.hw_id.cmp(&b.hw_id));
    monitors
        .iter_mut()
        .enumerate()
        .for_each(|(i, m)| m.id = format!("MONITOR{}", i + 1));
    for m in monitors.iter() {
        println!("{:?}", m);
    }
    monitors
}
