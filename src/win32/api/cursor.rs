use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorPos, SetCursorPos};

pub fn get_cursor_pos() -> (i32, i32) {
    let mut lppoint = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut lppoint).expect("GetCursorPos failed") };
    (lppoint.x, lppoint.y)
}

pub fn set_cursor_pos(x: i32, y: i32) {
    let _ = unsafe { SetCursorPos(x, y) };
}
