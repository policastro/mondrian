use std::ffi::OsString;
use std::mem::size_of;
use std::os::windows::ffi::OsStringExt;

use crate::win32::callbacks::enum_windows::user_managed_windows;
use crate::win32::window::window_ref::WindowRef;
use windows::Win32::Foundation::{CloseHandle, BOOL, HMODULE, HWND, LPARAM, MAX_PATH, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED};
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ};
use windows::Win32::UI::Controls::STATE_SYSTEM_INVISIBLE;
use windows::Win32::UI::Input::KeyboardAndMouse::{SendInput, SetFocus, INPUT, INPUT_KEYBOARD};
use windows::Win32::UI::WindowsAndMessaging::{
    DestroyWindow, EnumWindows, GetForegroundWindow, GetTitleBarInfo, GetWindow, GetWindowLongA, GetWindowRect,
    GetWindowTextW, GetWindowThreadProcessId, IsIconic, IsWindowVisible, RealGetWindowClassW, SendMessageW,
    SetForegroundWindow, ShowWindow, GWL_STYLE, GW_OWNER, MINMAXINFO, SHOW_WINDOW_CMD, TITLEBARINFO, WM_GETMINMAXINFO,
    WS_POPUP,
};

pub fn show_window(hwnd: HWND, cmd: SHOW_WINDOW_CMD) -> bool {
    unsafe { ShowWindow(hwnd, cmd).into() }
}

pub fn get_foreground_window() -> Option<HWND> {
    match unsafe { GetForegroundWindow() } {
        HWND(0) => None,
        hwnd => Some(hwnd),
    }
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
            Err(_) => return None,
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
        let mut rect: RECT = RECT::default();
        match GetWindowRect(hwnd, &mut rect) {
            Ok(_) => Some([rect.left, rect.top, rect.right, rect.bottom]),
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

pub fn is_window_visible(hwnd: HWND) -> bool {
    unsafe { IsWindowVisible(hwnd).as_bool() }
}

/// Returns true if the window is a managable by the user, i.e. if it is on its viewport
pub fn is_user_managable_window(hwnd: HWND, check_visibility: bool, check_iconic: bool) -> bool {
    if hwnd.0 == 0 {
        return false;
    }

    // TODO To exclude admin windows
    // TODO Can I manage "mondrian.exe" in another way? It isn't managed even without this check (filters), but I see it in the logs when WindowClosed is fired.
    if get_executable_name(hwnd).is_none() || get_executable_name(hwnd).is_some_and(|s| s == "mondrian.exe") {
        return false;
    }

    if !is_owner_window(hwnd) {
        return false;
    }

    if check_visibility && !is_window_visible(hwnd) {
        return false;
    }

    if check_iconic && is_iconic(hwnd) {
        return false;
    }

    // TODO: Used to enable some windows applications (like microsoft store) to be managable by the user
    // if EXPERIMENTAL_FEATURES { return !is_window_cloaked(hwnd); }

    if get_window_style(hwnd) & WS_POPUP.0 != 0 && is_window_cloaked(hwnd) {
        return false;
    }

    // the following removes some task tray programs and "Program Manager"
    let titlebar_info = get_title_bar_info(hwnd);
    if (titlebar_info.rgstate[0] & STATE_SYSTEM_INVISIBLE.0) != 0 {
        return false;
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

pub fn is_iconic(hwnd: HWND) -> bool {
    unsafe { IsIconic(hwnd).as_bool() }
}

pub fn is_owner_window(hwnd: HWND) -> bool {
    unsafe { GetWindow(hwnd, GW_OWNER) == HWND(0) }
}

pub fn is_window_cloaked(hwnd: HWND) -> bool {
    let cloaked: BOOL = BOOL(0);
    match unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_CLOAKED,
            &cloaked as *const _ as *mut _,
            size_of::<BOOL>() as u32,
        )
    } {
        Ok(_) => cloaked.0 != 0,
        Err(_) => false,
    }
}

pub fn focus(hwnd: HWND) {
    unsafe {
        let event = INPUT {
            r#type: INPUT_KEYBOARD,
            ..Default::default()
        };
        SendInput(&[event], size_of::<INPUT>() as i32);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(hwnd);
    };
}

pub fn get_window_minmax_size(hwnd: HWND) -> ((i32, i32), (i32, i32)) {
    unsafe {
        let mut min_max_info: MINMAXINFO = std::mem::zeroed();
        let _ = SendMessageW(
            hwnd,
            WM_GETMINMAXINFO,
            WPARAM(0),
            LPARAM(&mut min_max_info as *mut MINMAXINFO as isize),
        );
        let min_width = min_max_info.ptMinTrackSize.x;
        let min_height = min_max_info.ptMinTrackSize.y;
        let max_width = min_max_info.ptMaxTrackSize.x;
        let max_height = min_max_info.ptMaxTrackSize.y;

        ((min_width - 8, min_height - 8), (max_width - 8, max_height - 8)) // TODO -8 to remove the border
    }
}

pub fn destroy_window(hwnd: HWND) {
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}
