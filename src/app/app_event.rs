use windows::Win32::Foundation::HWND;

use super::structs::orientation::Orientation;

#[derive(Debug)]
pub enum AppEvent {
    Noop,
    Left,
    Right,
    ListAll,
    UpdateLayout,
    Stop,
    WindowOpened(HWND),
    WindowClosed(HWND),
    WindowMinimized(HWND),
    WindowRestored(HWND),
    WindowMoved(HWND, (i32, i32), Option<Orientation>),
    WindowResized(HWND),
}
