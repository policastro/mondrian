use crate::win32::api::monitor::{get_monitor_info, Monitor};
use widestring::U16CString;
use windows::Win32::{
    Foundation::{LPARAM, RECT},
    Graphics::Gdi::{HDC, HMONITOR, MONITORINFOEXW},
    UI::WindowsAndMessaging::MONITORINFOF_PRIMARY,
};

pub unsafe extern "system" fn enum_monitors_callback(
    monitor: HMONITOR,
    _hdc: HDC,
    lprc_monitor: *mut RECT,
    data: LPARAM,
) -> windows::Win32::Foundation::BOOL {
    let monitors = unsafe { &mut *(data.0 as *mut Vec<Monitor>) };
    let info: MONITORINFOEXW = get_monitor_info(monitor);
    let device_name = unsafe { U16CString::from_ptr_str(info.szDevice.as_ptr()).to_string_lossy() };

    monitors.push(Monitor {
        handle: monitor.0 as isize,
        id: device_name.strip_prefix(r"\\.\").unwrap_or_default().to_string(),
        primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
        resolution: (
            info.monitorInfo.rcMonitor.right - info.monitorInfo.rcMonitor.left,
            info.monitorInfo.rcMonitor.bottom - info.monitorInfo.rcMonitor.top,
        ),
        workspace: (
            info.monitorInfo.rcWork.right - info.monitorInfo.rcWork.left,
            info.monitorInfo.rcWork.bottom - info.monitorInfo.rcWork.top,
        ),
        offset: match lprc_monitor.as_ref() {
            None => (0, 0),
            Some(lprc_monitor) => (lprc_monitor.left, lprc_monitor.top),
        },
    });
    true.into()
}
