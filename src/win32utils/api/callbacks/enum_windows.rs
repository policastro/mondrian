use windows::Win32::Foundation::{BOOL, HWND, LPARAM};

use crate::win32utils::{api::window::is_user_managable_window, window::window_ref::WindowRef};

pub extern "system" fn user_managed_windows(hwnd: HWND, param: LPARAM) -> BOOL {
    if !is_user_managable_window(hwnd, true, true) {
        return true.into();
    }

    let window_info: WindowRef = WindowRef::new(hwnd);
    let windows = unsafe { &mut *(param.0 as *mut Vec<WindowRef>) };
    windows.push(window_info);
    true.into()
}
