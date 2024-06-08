use windows::Win32::Foundation::HWND;

use super::structs::orientation::Orientation;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Win32Event {
    Noop,
    Quit,
    WindowOpened(HWND),
    WindowClosed(HWND),
    WindowMinimized(HWND),
    WindowRestored(HWND),
    WindowMoved(HWND, (i32, i32), Option<Orientation>),
    WindowResized(HWND),
}