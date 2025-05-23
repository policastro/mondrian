use crate::win32::api::monitor::get_monitor_info;
use crate::win32::callbacks::enum_windows::user_managed_windows;
use crate::win32::window::window_ref::WindowRef;
use lazy_static::lazy_static;
use std::ffi::{OsStr, OsString};
use std::mem::size_of;
use std::os::windows::ffi::{OsStrExt, OsStringExt};
use windows::core::PCWSTR;
use windows::Win32::Foundation::{CloseHandle, BOOL, HANDLE, HMODULE, HWND, LPARAM, MAX_PATH, RECT, WPARAM};
use windows::Win32::Graphics::Dwm::{DwmGetWindowAttribute, DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS};
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};
use windows::Win32::System::LibraryLoader::GetModuleHandleExW;
use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
};
use windows::Win32::UI::Controls::STATE_SYSTEM_INVISIBLE;
use windows::Win32::UI::HiDpi::GetDpiForWindow;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, SetFocus, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS, KEYEVENTF_KEYUP, VK_NONAME,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, EnumWindows, GetDesktopWindow, GetForegroundWindow, GetTitleBarInfo, GetWindow,
    GetWindowLongW, GetWindowPlacement, GetWindowRect, GetWindowTextW, GetWindowThreadProcessId, IsIconic,
    IsWindowVisible, RealGetWindowClassW, RegisterClassExW, SendMessageW, SetForegroundWindow, ShowWindow, CS_HREDRAW,
    CS_VREDRAW, GWL_EXSTYLE, GWL_STYLE, GW_OWNER, MINMAXINFO, SHOW_WINDOW_CMD, SW_MAXIMIZE, TITLEBARINFO,
    WINDOWPLACEMENT, WINDOW_EX_STYLE, WINDOW_STYLE, WM_GETMINMAXINFO, WNDCLASSEXW, WNDPROC, WS_CHILD, WS_CHILDWINDOW,
    WS_EX_TOPMOST, WS_POPUP,
};

lazy_static!(
    static ref EMPTY_INPUT_EVT_DOWN: INPUT = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VK_NONAME, // NOTE: seems unused
                dwFlags: KEYBD_EVENT_FLAGS(0),
                ..Default::default()
            },
        },
    };
    static ref EMPTY_INPUT_EVT_UP: INPUT = INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: VK_NONAME,
                dwFlags: KEYEVENTF_KEYUP,
                ..Default::default()
            },
        },
    };
);

pub fn show_window(hwnd: HWND, cmd: SHOW_WINDOW_CMD) -> bool {
    unsafe { ShowWindow(hwnd, cmd).into() }
}

pub fn get_foreground_window() -> Option<HWND> {
    match unsafe { GetForegroundWindow() } {
        hwnd if hwnd.is_invalid() => None,
        hwnd => Some(hwnd),
    }
}

pub fn get_desktop_window() -> Option<HWND> {
    match unsafe { GetDesktopWindow() } {
        hwnd if hwnd.is_invalid() => None,
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
    unsafe { GetWindowLongW(hwnd, GWL_STYLE) as u32 }
}

pub fn get_window_exstyle(hwnd: HWND) -> u32 {
    unsafe { GetWindowLongW(hwnd, GWL_EXSTYLE) as u32 }
}

fn get_window_thread_process_id(hwnd: HWND) -> Option<u32> {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
    Some(pid)
}

fn open_process(process_access_rights: PROCESS_ACCESS_RIGHTS, inherit_handle: bool, pid: u32) -> Result<HANDLE, ()> {
    unsafe { OpenProcess(process_access_rights, inherit_handle, pid).map_err(|_| ()) }
}

pub fn get_executable_path(hwnd: HWND) -> Option<String> {
    let size;
    let mut buf: [u16; MAX_PATH as usize] = [0; MAX_PATH as usize];
    unsafe {
        let pid: u32 = get_window_thread_process_id(hwnd)?;
        let h_process = open_process(PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ, false, pid).ok()?;
        size = GetModuleFileNameExW(h_process, HMODULE::default(), &mut buf);
        CloseHandle(h_process).expect("CloseHandle failed");
    }

    if size == 0 {
        return None;
    }

    let path = OsString::from_wide(&buf[0..size as usize])
        .to_string_lossy()
        .into_owned();
    Some(path)
}

pub fn get_executable_name(hwnd: HWND) -> Option<String> {
    match get_executable_path(hwnd) {
        Some(path) => path.split('\\').next_back().map(|s| s.to_string()),
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

pub fn is_program_manager_window(hwnd: HWND) -> bool {
    get_executable_name(hwnd).is_some_and(|s| s == "explorer.exe") && get_class_name(hwnd) == "Progman"
}

/// Returns true if the window is manageable by the tiles manager
pub fn is_user_manageable_window(
    hwnd: HWND,
    check_visibility: bool,
    check_iconic: bool,
    check_title_bar: bool,
) -> bool {
    if hwnd.is_invalid() {
        return false;
    }

    // INFO: to exclude admin windows
    if get_executable_name(hwnd).is_none_or(|s| s == "mondrian.exe") {
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

    if get_window_style(hwnd) & WS_POPUP.0 != 0 && is_window_cloaked(hwnd) {
        return false;
    }

    let titlebar_info = get_title_bar_info(hwnd);
    if check_title_bar && ((titlebar_info.rgstate[0] & STATE_SYSTEM_INVISIBLE.0) != 0) {
        return false;
    }

    true
}

pub fn enum_user_manageable_windows() -> Vec<WindowRef> {
    let mut windows: Vec<WindowRef> = Vec::new();
    let lparam = LPARAM(windows.as_mut() as *mut Vec<WindowRef> as isize);

    unsafe {
        EnumWindows(Some(user_managed_windows), lparam).expect("EnumWindows failed");
    }

    windows
}

pub fn is_iconic(hwnd: HWND) -> bool {
    unsafe { IsIconic(hwnd).as_bool() }
}

pub fn is_owner_window(hwnd: HWND) -> bool {
    unsafe { GetWindow(hwnd, GW_OWNER).is_err() }
}

pub fn is_window_cloaked(hwnd: HWND) -> bool {
    let cloaked: BOOL = false.into();
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

pub fn is_window_topmost(hwnd: HWND) -> bool {
    let exstyle = get_window_exstyle(hwnd);
    (exstyle & WS_EX_TOPMOST.0) != 0
}

pub fn focus(hwnd: HWND) {
    unsafe {
        SendInput(&[*EMPTY_INPUT_EVT_DOWN], size_of::<INPUT>() as i32);
        SendInput(&[*EMPTY_INPUT_EVT_UP], size_of::<INPUT>() as i32);
        let _ = SetForegroundWindow(hwnd);
        let _ = SetFocus(hwnd);
    };
}

pub fn destroy_window(hwnd: HWND) {
    unsafe {
        let _ = DestroyWindow(hwnd);
    }
}

pub fn is_maximized(hwnd: HWND) -> bool {
    unsafe {
        let mut wp: WINDOWPLACEMENT = std::mem::zeroed();
        wp.length = std::mem::size_of::<WINDOWPLACEMENT>() as u32;

        if GetWindowPlacement(hwnd, &mut wp).is_ok() {
            return wp.showCmd == SW_MAXIMIZE.0 as u32;
        }
    }
    false
}

pub fn is_fullscreen(hwnd: HWND) -> bool {
    let win_size = match get_window_rect(hwnd) {
        Some(w) => w,
        None => return false,
    };

    let monitorinfo = unsafe { get_monitor_info(MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST)) };
    let desktop_size = monitorinfo.monitorInfo.rcMonitor;

    win_size[0] == desktop_size.left
        && win_size[1] == desktop_size.top
        && win_size[2] == desktop_size.right
        && win_size[3] == desktop_size.bottom
}

pub fn has_child_window_style(hwnd: HWND) -> bool {
    let ws = get_window_style(hwnd);
    ws & WS_CHILD.0 != 0 || ws & WS_CHILDWINDOW.0 != 0
}

pub fn register_class(class_name: &str, window_proc: WNDPROC) -> u16 {
    let mut hmod: HMODULE = unsafe { std::mem::zeroed() };
    unsafe { GetModuleHandleExW(0, None, &mut hmod).unwrap() };

    let cs_w: Vec<u16> = OsStr::new(class_name).encode_wide().chain(Some(0)).collect();
    let cs_ptr = PCWSTR(cs_w.as_ptr());

    let wc = WNDCLASSEXW {
        cbSize: std::mem::size_of::<WNDCLASSEXW>() as u32,
        hInstance: hmod.into(),
        lpszClassName: cs_ptr,
        lpfnWndProc: window_proc,
        style: CS_HREDRAW | CS_VREDRAW,
        ..Default::default()
    };

    unsafe { RegisterClassExW(&wc) }
}

pub fn create_window<T>(
    ex_style: WINDOW_EX_STYLE,
    cs_ptr: PCWSTR,
    style: WINDOW_STYLE,
    (x, y, w, h): (i32, i32, i32, i32),
    parent: Option<HWND>,
    hmod: HMODULE,
    data: T,
) -> Option<HWND> {
    let data = Some(Box::into_raw(Box::new(data)) as *mut _ as _);
    let parent = parent.unwrap_or_default();
    let hwnd = unsafe { CreateWindowExW(ex_style, cs_ptr, None, style, x, y, w, h, parent, None, hmod, data) };
    hwnd.ok()
}

pub fn get_dwmwa_extended_frame_bounds(hwnd: HWND) -> Option<[i32; 4]> {
    let mut rect: RECT = RECT::default();
    match unsafe {
        DwmGetWindowAttribute(
            hwnd,
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut rect as *mut _ as *mut _,
            size_of::<RECT>() as u32,
        )
    } {
        Ok(_) => Some([rect.left, rect.top, rect.right, rect.bottom]),
        Err(_) => None,
    }
}

pub fn get_dpi_for_window(hwnd: HWND) -> u32 {
    unsafe { GetDpiForWindow(hwnd) }
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

        ((min_width, min_height), (max_width, max_height))
    }
}
