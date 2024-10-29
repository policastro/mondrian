use windows::Win32::Foundation::POINT;
use windows::Win32::UI::WindowsAndMessaging::{GetCursorInfo, GetCursorPos, CURSORINFO};

pub fn get_cursor_pos() -> (i32, i32) {
    let mut lppoint = POINT { x: 0, y: 0 };
    unsafe { GetCursorPos(&mut lppoint).expect("GetCursorPos failed") };
    (lppoint.x, lppoint.y)
}

pub fn get_cursor_info() -> Result<CURSORINFO, windows::core::Error> {
    let mut cursor_info: CURSORINFO = unsafe { std::mem::zeroed() };
    cursor_info.cbSize = std::mem::size_of::<CURSORINFO>() as u32;
    unsafe { GetCursorInfo(&mut cursor_info)? }
    Ok(cursor_info)
}
