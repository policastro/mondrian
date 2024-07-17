use std::{ffi::OsStr, os::windows::ffi::OsStrExt};

use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{COLORREF, HWND, RECT},
        Graphics::Gdi::{
            BeginPaint, CreatePen, DeleteObject, EndPaint, Rectangle, SelectObject, PAINTSTRUCT, PS_SOLID,
        },
        UI::WindowsAndMessaging::GetClientRect,
    },
};

use super::color::Color;

pub fn to_pcwstr(s: &str) -> PCWSTR {
    let wide: Vec<u16> = OsStr::new(s).encode_wide().chain(Some(0)).collect();
    PCWSTR(wide.as_ptr())
}

pub fn create_border(hwnd: HWND, thickness: i32, color: Color) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        let mut rc: RECT = std::mem::zeroed();
        let _ = GetClientRect(hwnd, &mut rc);

        let h_pen = CreatePen(PS_SOLID, thickness, COLORREF(color.into()));
        let old_pen = SelectObject(hdc, h_pen);

        let _ = Rectangle(hdc, rc.left, rc.top, rc.right, rc.bottom);

        SelectObject(hdc, old_pen);
        let _ = DeleteObject(h_pen);

        let _ = EndPaint(hwnd, &ps);
    }
}
