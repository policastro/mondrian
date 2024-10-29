use windows::Win32::Foundation::HWND;

use crate::app::{
    mondrian_command::MondrianMessage,
    structs::{area::Area, direction::Direction},
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TMCommand {
    Noop,
    Quit,
    WindowOpened(HWND),
    WindowClosed(HWND),
    WindowMinimized(HWND),
    WindowRestored(HWND),
    WindowMaximized(HWND),
    WindowUnmaximized(HWND),
    WindowStartMoveSize(HWND),
    WindowMoved(HWND, (i32, i32), bool, bool),
    WindowResized(HWND, Area, Area),
    Focus(Direction),
    Move(Direction),
    Resize(Direction, u8),
    Release(Option<bool>),
    Update(bool),
    Focalize,
    Invert,
    ListManagedWindows,
    Minimize,
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
                | TMCommand::WindowStartMoveSize(_)
                | TMCommand::WindowResized(_, _, _)
                | TMCommand::WindowMoved(_, _, _, _)
                | TMCommand::WindowMaximized(_)
                | TMCommand::WindowUnmaximized(_)
        )
    }

    pub fn get_hwnd(&self) -> Option<HWND> {
        match self {
            TMCommand::WindowOpened(hwnd)
            | TMCommand::WindowClosed(hwnd)
            | TMCommand::WindowMinimized(hwnd)
            | TMCommand::WindowRestored(hwnd)
            | TMCommand::WindowStartMoveSize(hwnd)
            | TMCommand::WindowMoved(hwnd, _, _, _)
            | TMCommand::WindowResized(hwnd, _, _)
            | TMCommand::WindowMaximized(hwnd)
            | TMCommand::WindowUnmaximized(hwnd) => Some(*hwnd),
            _ => None,
        }
    }

    pub fn is_noop(&self) -> bool {
        matches!(self, TMCommand::Noop)
    }

    pub fn is_op(&self) -> bool {
        !self.is_noop()
    }
}

impl From<&MondrianMessage> for TMCommand {
    fn from(msg: &MondrianMessage) -> Self {
        match msg {
            MondrianMessage::Minimize => TMCommand::Minimize,
            MondrianMessage::Focus(direction) => TMCommand::Focus(*direction),
            MondrianMessage::Move(direction) => TMCommand::Move(*direction),
            MondrianMessage::Release(b) => TMCommand::Release(*b),
            MondrianMessage::Resize(d, s) => TMCommand::Resize(*d, *s),
            MondrianMessage::Focalize => TMCommand::Focalize,
            MondrianMessage::Invert => TMCommand::Invert,
            MondrianMessage::ListManagedWindows => TMCommand::ListManagedWindows,
            _ => TMCommand::Noop,
        }
    }
}
