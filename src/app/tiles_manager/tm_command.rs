use windows::Win32::Foundation::HWND;

use crate::app::structs::direction::Direction;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TMCommand {
    Noop,
    Quit,
    WindowOpened(HWND),
    WindowClosed(HWND),
    WindowMinimized(HWND),
    WindowRestored(HWND),
    WindowMoved(HWND, (i32, i32), bool, bool),
    WindowResized(HWND),
    Focus(Direction),
}

impl TMCommand {
    pub fn require_update(&self) -> bool {
        !matches!(self, TMCommand::Noop | TMCommand::Quit | TMCommand::Focus(_))
    }

    pub fn can_be_filtered(&self) -> bool {
        matches!(
            self,
            TMCommand::WindowOpened(_)
                | TMCommand::WindowMinimized(_)
                | TMCommand::WindowRestored(_)
                | TMCommand::WindowResized(_)
                | TMCommand::WindowMoved(_, _, _, _)
        )
    }

    pub fn get_hwnd(&self) -> Option<HWND> {
        match self {
            TMCommand::WindowOpened(hwnd)
            | TMCommand::WindowClosed(hwnd)
            | TMCommand::WindowMinimized(hwnd)
            | TMCommand::WindowRestored(hwnd)
            | TMCommand::WindowMoved(hwnd, _, _, _)
            | TMCommand::WindowResized(hwnd) => Some(*hwnd),
            _ => None,
        }
    }
}
