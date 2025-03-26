use crate::app::{
    mondrian_message::{MondrianMessage, SystemEvent, WindowEvent},
    structs::direction::Direction,
};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum TMCommand {
    WindowEvent(WindowEvent),
    SystemEvent(SystemEvent),
    Focus(Direction),
    SwitchFocus,
    Insert(Direction),
    Move(Direction, bool, u16),
    Resize(Direction, u16, u16),
    Release(Option<bool>),
    Peek(Direction, f32),
    Update(bool),
    Focalize,
    HalfFocalize,
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
            TMCommand::Focus(_) | TMCommand::SwitchFocus => false,
            TMCommand::Insert(_) => true,
            TMCommand::Move(_, _, _) => true,
            TMCommand::Resize(_, _, _) => true,
            TMCommand::Release(_) => true,
            TMCommand::Peek(_, _) => true,
            TMCommand::Update(_) => true,
            TMCommand::Focalize | TMCommand::HalfFocalize => true,
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
            MondrianMessage::SwitchFocus => Ok(TMCommand::SwitchFocus),
            MondrianMessage::Move(direction, floating_inc) => Ok(TMCommand::Move(*direction, false, *floating_inc)),
            MondrianMessage::Insert(direction) => Ok(TMCommand::Insert(*direction)),
            MondrianMessage::MoveInsert(direction, floating_inc) => {
                Ok(TMCommand::Move(*direction, true, *floating_inc))
            }
            MondrianMessage::Release(b) => Ok(TMCommand::Release(*b)),
            MondrianMessage::Resize(d, s, fs) => Ok(TMCommand::Resize(*d, *s, *fs)),
            MondrianMessage::Peek(d, r) => Ok(TMCommand::Peek(*d, *r)),
            MondrianMessage::Focalize => Ok(TMCommand::Focalize),
            MondrianMessage::HalfFocalize => Ok(TMCommand::HalfFocalize),
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
