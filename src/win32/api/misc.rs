use std::{ffi::OsStr, os::windows::ffi::OsStrExt};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        System::Threading::GetCurrentThreadId,
        UI::WindowsAndMessaging::{PostMessageW, PostThreadMessageW},
    },
};

pub fn get_current_thread_id() -> u32 {
    unsafe { GetCurrentThreadId() }
}

pub fn post_empty_thread_message(thread_id: u32, msg: u32) {
    let _ = unsafe { PostThreadMessageW(thread_id, msg, WPARAM(0), LPARAM(0)) };
}

pub fn post_empyt_message(hwnd: HWND, msg: u32) {
    post_message::<usize>(hwnd, msg, None);
}

pub fn post_message<L>(hwnd: HWND, msg: u32, data: Option<L>) {
    let lparam = match data {
        Some(lparam) => LPARAM(Box::into_raw(Box::new(lparam)) as *mut _ as isize),
        None => LPARAM(0),
    };
    let _ = unsafe { PostMessageW(hwnd, msg, WPARAM(0), lparam) };
}

pub fn str_to_pcwstr(s: &str) -> PCWSTR {
    let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
    PCWSTR(wide.as_ptr())
}
