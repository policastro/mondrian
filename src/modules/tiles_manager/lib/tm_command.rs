use crate::app::{
    mondrian_message::{MondrianMessage, WindowEvent},
    structs::direction::Direction,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TMCommand {
    WindowEvent(WindowEvent),
    Focus(Direction),
    Move(Direction),
    Resize(Direction, u8),
    Release(Option<bool>),
    Update(bool),
    Focalize,
    Invert,
    ListManagedWindows,
    Minimize,
    Quit,
}

impl TMCommand {
    pub fn require_update(&self) -> bool {
        !matches!(self, TMCommand::Quit | TMCommand::Focus(_))
    }
}

impl From<WindowEvent> for TMCommand {
    fn from(event: WindowEvent) -> Self {
        TMCommand::WindowEvent(event)
    }
}

impl TryFrom<&MondrianMessage> for TMCommand {
    type Error = ();

    fn try_from(msg: &MondrianMessage) -> Result<Self, Self::Error> {
        match msg {
            MondrianMessage::Minimize => Ok(TMCommand::Minimize),
            MondrianMessage::Focus(direction) => Ok(TMCommand::Focus(*direction)),
            MondrianMessage::Move(direction) => Ok(TMCommand::Move(*direction)),
            MondrianMessage::Release(b) => Ok(TMCommand::Release(*b)),
            MondrianMessage::Resize(d, s) => Ok(TMCommand::Resize(*d, *s)),
            MondrianMessage::Focalize => Ok(TMCommand::Focalize),
            MondrianMessage::Invert => Ok(TMCommand::Invert),
            MondrianMessage::ListManagedWindows => Ok(TMCommand::ListManagedWindows),
            MondrianMessage::WindowEvent(event) => Ok(TMCommand::WindowEvent(*event)),
            _ => Err(()),
        }
    }
}
