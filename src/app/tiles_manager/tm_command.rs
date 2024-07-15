use windows::Win32::Foundation::HWND;

use crate::app::structs::{direction::Direction, orientation::Orientation};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TMCommand {
    Noop,
    Quit,
    WindowOpened(HWND),
    WindowClosed(HWND),
    WindowMinimized(HWND),
    WindowRestored(HWND),
    WindowMoved(HWND, (i32, i32), Option<Orientation>),
    WindowResized(HWND),
    Focus(Direction),
}
