use std::ffi::OsString;
use std::os::windows::ffi::OsStringExt;

use windows::Win32::Foundation::{CloseHandle, BOOL, HMODULE, HWND, LPARAM, MAX_PATH, RECT};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::Controls::STATE_SYSTEM_INVISIBLE;
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, GetForegroundWindow, GetTitleBarInfo, GetWindow, GetWindowLongA, GetWindowRect, GetWindowTextW,
    GetWindowThreadProcessId, IsIconic, IsWindowVisible, RealGetWindowClassW, GWL_STYLE, GW_OWNER, TITLEBARINFO,
    WS_POPUP,
};

use crate::win32utils::window::window_ref::WindowRef;

use super::callbacks::enum_windows::user_managed_windows;

pub fn get_foreground_window() -> HWND {
    unsafe { GetForegroundWindow() }
}

pub fn get_class_name(hwnd: HWND) -> String {
    unsafe {
        let mut buffer = [0; MAX_PATH as usize];
        RealGetWindowClassW(hwnd, &mut buffer);
        OsString::from_wide(&buffer)
            .to_string_lossy()
            .trim_matches(char::from(0))
            .to_string()
    }
}

pub fn get_window_style(hwnd: HWND) -> u32 {
    unsafe { GetWindowLongA(hwnd, GWL_STYLE) as u32 }
}

pub fn get_executable_path(hwnd: HWND) -> Option<String> {
    let size;
    let mut buf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe {
        let mut pid: u32 = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));

        let h_process = match OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, BOOL(0), pid) {
            Ok(h_process) => h_process,
            Err(err) => {
                log::error!("OpenProcess failed: {}", err);
                return None;
            }
        };

        size = GetModuleFileNameExW(h_process, HMODULE(0), &mut buf);

        CloseHandle(h_process).expect("CloseHandle failed");
    }

    match size {
        0 => None,
        _ => Some(
            OsString::from_wide(&buf[0..size as usize])
                .to_string_lossy()
                .into_owned(),
        ),
    }
}

pub fn get_executable_name(hwnd: HWND) -> Option<String> {
    match get_executable_path(hwnd) {
        Some(path) => path.split('\\').last().map(|s| s.to_string()),
        None => None,
    }
}

pub fn get_window_box(hwnd: HWND) -> Option<[i32; 4]> {
    get_window_rect(hwnd).map(|[x0, y0, x1, y1]| [x0, y0, x1 - x0, y1 - y0])
}

pub fn get_window_rect(hwnd: HWND) -> Option<[i32; 4]> {
    unsafe {
        let offset = [8, 8];
        let mut rect: RECT = RECT::default();
        match GetWindowRect(hwnd, &mut rect) {
            Ok(_) => Some([rect.left + offset[0], rect.top + offset[1], rect.right, rect.bottom]),
            Err(_) => None,
        }
    }
}

pub fn get_window_title(hwnd: HWND) -> Option<String> {
    let mut buffer: [u16; 1024] = [0; 1024];
    unsafe { GetWindowTextW(hwnd, &mut buffer) };
    String::from_utf16(&buffer).map_or(None, |s| Some(s.trim_matches(char::from(0)).to_string()))
}

pub fn get_title_bar_info(hwnd: HWND) -> TITLEBARINFO {
    let mut titlebar_info: TITLEBARINFO = unsafe { std::mem::zeroed() };
    titlebar_info.cbSize = std::mem::size_of::<TITLEBARINFO>() as u32;
    let _ = unsafe { GetTitleBarInfo(hwnd, &mut titlebar_info) };
    titlebar_info
}

/// Returns true if the window is a managable by the user, i.e. if it is on its viewport
pub fn is_user_managable_window(hwnd: HWND, check_visibility: bool, check_iconic: bool) -> bool {
    unsafe {
        if check_visibility && !IsWindowVisible(hwnd).as_bool() {
            return false;
        }

        if GetWindow(hwnd, GW_OWNER) != HWND(0) {
            return false;
        }

        if IsIconic(hwnd).into() && check_iconic {
            return false;
        }

        if get_window_style(hwnd) & WS_POPUP.0 != 0 {
            return false;
        }

        // the following removes some task tray programs and "Program Manager"
        let titlebar_info = get_title_bar_info(hwnd);
        if (titlebar_info.rgstate[0] & STATE_SYSTEM_INVISIBLE.0) != 0 {
            return false;
        }
    }

    true
}

pub fn enum_user_manageable_windows() -> Vec<WindowRef> {
    let mut windows: Vec<WindowRef> = Vec::new();
    let lparam = LPARAM(windows.as_mut() as *mut Vec<WindowRef> as isize);

    // TODO Handle error
    unsafe {
        EnumWindows(Some(user_managed_windows), lparam).expect("EnumWindows failed");
    }

    windows
}
