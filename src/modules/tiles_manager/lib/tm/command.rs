use crate::app::{
    mondrian_message::{MondrianMessage, SystemEvent, WindowEvent},
    structs::direction::Direction,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TMCommand {
    WindowEvent(WindowEvent),
    SystemEvent(SystemEvent),
    Focus(Direction),
    Insert(Direction),
    Move(Direction, bool),
    Resize(Direction, u8),
    Release(Option<bool>),
    Peek(Direction, f32),
    Update(bool),
    Focalize,
    CycleFocalized(bool),
    Invert,
    ListManagedWindows,
    QueryInfo,
    Minimize,
    Quit,
    Amplify,
}

impl TMCommand {
    pub fn can_change_layout(&self) -> bool {
        match self {
            TMCommand::WindowEvent(window_event) => match window_event {
                WindowEvent::Opened(_) => true,
                WindowEvent::Closed(_) => true,
                WindowEvent::Minimized(_) => true,
                WindowEvent::Restored(_) => true,
                WindowEvent::Maximized(_) => true,
                WindowEvent::Unmaximized(_) => true,
                WindowEvent::Focused(_) => false,
                WindowEvent::StartMoveSize(_) => false,
                WindowEvent::EndMoveSize(..) => true,
            },
            TMCommand::SystemEvent(_) => true,
            TMCommand::Focus(_) => false,
            TMCommand::Insert(_) => true,
            TMCommand::Move(_, _) => true,
            TMCommand::Resize(_, _) => true,
            TMCommand::Release(_) => true,
            TMCommand::Peek(_, _) => true,
            TMCommand::Update(_) => true,
            TMCommand::Focalize => true,
            TMCommand::Invert => true,
            TMCommand::CycleFocalized(_) => true,
            TMCommand::ListManagedWindows => false,
            TMCommand::QueryInfo => false,
            TMCommand::Minimize => true,
            TMCommand::Quit => false,
            TMCommand::Amplify => true,
        }
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
            MondrianMessage::Move(direction) => Ok(TMCommand::Move(*direction, false)),
            MondrianMessage::Insert(direction) => Ok(TMCommand::Insert(*direction)),
            MondrianMessage::MoveInsert(direction) => Ok(TMCommand::Move(*direction, true)),
            MondrianMessage::Release(b) => Ok(TMCommand::Release(*b)),
            MondrianMessage::Resize(d, s) => Ok(TMCommand::Resize(*d, *s)),
            MondrianMessage::Peek(d, r) => Ok(TMCommand::Peek(*d, *r)),
            MondrianMessage::Focalize => Ok(TMCommand::Focalize),
            MondrianMessage::CycleFocalized { next } => Ok(TMCommand::CycleFocalized(*next)),
            MondrianMessage::Invert => Ok(TMCommand::Invert),
            MondrianMessage::Amplify => Ok(TMCommand::Amplify),
            MondrianMessage::ListManagedWindows => Ok(TMCommand::ListManagedWindows),
            MondrianMessage::QueryInfo => Ok(TMCommand::QueryInfo),
            MondrianMessage::WindowEvent(event) => Ok(TMCommand::WindowEvent(*event)),
            MondrianMessage::SystemEvent(event) => Ok(TMCommand::SystemEvent(*event)),
            _ => Err(()),
        }
    }
}
