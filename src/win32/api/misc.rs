use windows::Win32::Foundation::HWND;
use windows::Win32::Foundation::LPARAM;
use windows::Win32::Foundation::WPARAM;
use windows::Win32::System::Threading::GetCurrentThreadId;
use windows::Win32::UI::WindowsAndMessaging::PostMessageW;
use windows::Win32::UI::WindowsAndMessaging::PostThreadMessageW;

pub fn get_current_thread_id() -> u32 {
    unsafe { GetCurrentThreadId() }
}

pub fn post_empty_thread_message(thread_id: u32, msg: u32) {
    let _ = unsafe { PostThreadMessageW(thread_id, msg, None, None) };
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
