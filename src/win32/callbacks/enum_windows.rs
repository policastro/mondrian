use windows::Win32::Foundation::{BOOL, HWND, LPARAM};

use crate::win32::{api::window::{is_user_managable_window, is_window_cloaked, is_window_visible}, window::window_ref::WindowRef};

pub extern "system" fn user_managed_windows(hwnd: HWND, param: LPARAM) -> BOOL {
    let is_managed = is_user_managable_window(hwnd, false, true, true);
    let is_visible = is_window_visible(hwnd);
    let is_cloaked = is_window_cloaked(hwnd);
    if !is_managed || is_cloaked || !is_visible {
        return true.into();
    }

    let window_info: WindowRef = WindowRef::new(hwnd);
    let windows = unsafe { &mut *(param.0 as *mut Vec<WindowRef>) };
    windows.push(window_info);
    true.into()
}
