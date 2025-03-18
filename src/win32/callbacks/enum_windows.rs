use windows::Win32::Foundation::{BOOL, HWND, LPARAM};

use crate::win32::{
    api::window::{is_user_manageable_window, is_window_cloaked},
    window::window_ref::WindowRef,
};

pub extern "system" fn user_managed_windows(hwnd: HWND, param: LPARAM) -> BOOL {
    let is_managed = is_user_manageable_window(hwnd, true, true, true);
    let is_cloaked = is_window_cloaked(hwnd);

    if !is_managed || is_cloaked {
        return true.into();
    }

    let window_info: WindowRef = hwnd.into();
    let windows = unsafe { &mut *(param.0 as *mut Vec<WindowRef>) };
    windows.push(window_info);
    true.into()
}
