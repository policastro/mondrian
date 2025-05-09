use crate::{
    app::structs::area::Area,
    win32::api::monitor::{get_monitor_info, Monitor},
};
use widestring::U16CString;
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{LPARAM, RECT},
        Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW, HDC, HMONITOR, MONITORINFOEXW},
        UI::WindowsAndMessaging::{EDD_GET_DEVICE_INTERFACE_NAME, MONITORINFOF_PRIMARY},
    },
};

pub(crate) unsafe extern "system" fn enum_monitors_callback(
    monitor: HMONITOR,
    _hdc: HDC,
    lprc_monitor: *mut RECT,
    data: LPARAM,
) -> windows::Win32::Foundation::BOOL {
    let monitors = unsafe { &mut *(data.0 as *mut Vec<Monitor>) };
    let info: MONITORINFOEXW = get_monitor_info(monitor);

    let origin = match lprc_monitor.as_ref() {
        None => (0, 0),
        Some(lprc_monitor) => (lprc_monitor.left, lprc_monitor.top),
    };

    let offset_x = info.monitorInfo.rcWork.left - info.monitorInfo.rcMonitor.left;
    let offset_y = info.monitorInfo.rcWork.top - info.monitorInfo.rcMonitor.top;

    let resolution = (
        info.monitorInfo.rcMonitor.right - info.monitorInfo.rcMonitor.left,
        info.monitorInfo.rcMonitor.bottom - info.monitorInfo.rcMonitor.top,
    );

    let workspace = (
        info.monitorInfo.rcWork.right - info.monitorInfo.rcWork.left,
        info.monitorInfo.rcWork.bottom - info.monitorInfo.rcWork.top,
    );

    let monitor_area = Area::new(
        origin.0,
        origin.1,
        u16::try_from(resolution.0).expect("Failed to convert i32 to u16"),
        u16::try_from(resolution.1).expect("Failed to convert i32 to u16"),
    );

    let workspace_area = Area::new(
        origin.0 + offset_x,
        origin.1 + offset_y,
        u16::try_from(workspace.0).expect("Failed to convert i32 to u16"),
        u16::try_from(workspace.1).expect("Failed to convert i32 to u16"),
    );

    monitors.push(Monitor {
        handle: monitor.0 as isize,
        id: String::default(), // NOTE: will be assigned later based on `hw_id`
        hw_id: get_monitor_hw_id(info.szDevice.as_ptr()).unwrap_or_default(),
        primary: info.monitorInfo.dwFlags & MONITORINFOF_PRIMARY != 0,
        resolution,
        monitor_area,
        workspace,
        workspace_area,
        offset: origin,
    });
    true.into()
}

fn get_monitor_hw_id(device_name_ptr: *const u16) -> Option<String> {
    let mut disp_device_id = DISPLAY_DEVICEW {
        cb: std::mem::size_of::<DISPLAY_DEVICEW>() as u32,
        ..unsafe { std::mem::zeroed() }
    };

    let res = unsafe {
        EnumDisplayDevicesW(
            PCWSTR(device_name_ptr),
            0,
            &mut disp_device_id as _,
            EDD_GET_DEVICE_INTERFACE_NAME,
        )
    };

    match res.as_bool() {
        true => Some(unsafe { U16CString::from_ptr_str(disp_device_id.DeviceID.as_ptr()) }.to_string_lossy()),
        false => None,
    }
}
