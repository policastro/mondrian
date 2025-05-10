use crate::app::{
    mondrian_message::{MondrianMessage, SystemEvent, WindowEvent},
    structs::direction::Direction,
};

#[derive(Debug, PartialEq, Clone)]
pub enum TMCommand {
    WindowEvent(WindowEvent),
    SystemEvent(SystemEvent),
    Focus(Direction),
    FocusMonitor(Direction),
    Close,
    Topmost,
    SwitchFocus,
    FocusWorkspace { id: String },
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
    MoveToWorkspace { id: String, focus: bool },
}

impl TMCommand {
    pub fn can_change_layout(&self) -> bool {
        match self {
            TMCommand::WindowEvent(window_event) => match window_event {
                WindowEvent::Opened(..)
                | WindowEvent::Closed(..)
                | WindowEvent::Minimized(..)
                | WindowEvent::Restored(..)
                | WindowEvent::Maximized(..)
                | WindowEvent::Unmaximized(..)
                | WindowEvent::EndMoveSize(..) => true,
                WindowEvent::Focused(..) | WindowEvent::StartMoveSize(..) => false,
            },
            TMCommand::SystemEvent(..)
            | TMCommand::Close
            | TMCommand::Insert(..)
            | TMCommand::Move(..)
            | TMCommand::Peek(..)
            | TMCommand::Resize(..)
            | TMCommand::Release(..)
            | TMCommand::Focalize
            | TMCommand::HalfFocalize
            | TMCommand::Invert
            | TMCommand::Amplify
            | TMCommand::Minimize
            | TMCommand::CycleFocalized(..)
            | TMCommand::Update(..)
            | TMCommand::FocusWorkspace { .. }
            | TMCommand::MoveToWorkspace { .. } => true,
            TMCommand::ListManagedWindows
            | TMCommand::Topmost
            | TMCommand::Focus(..)
            | TMCommand::FocusMonitor(..)
            | TMCommand::SwitchFocus
            | TMCommand::QueryInfo
            | TMCommand::Quit => false,
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
            MondrianMessage::FocusMonitor(direction) => Ok(TMCommand::FocusMonitor(*direction)),
            MondrianMessage::FocusWorkspace { id } => Ok(TMCommand::FocusWorkspace { id: id.clone() }),
            MondrianMessage::MoveToWorkspace { id, focus } => Ok(TMCommand::MoveToWorkspace {
                id: id.clone(),
                focus: *focus,
            }),
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
            MondrianMessage::Close => Ok(TMCommand::Close),
            MondrianMessage::Topmost => Ok(TMCommand::Topmost),
            MondrianMessage::RefreshConfig
            | MondrianMessage::OpenConfig
            | MondrianMessage::Retile
            | MondrianMessage::Configure
            | MondrianMessage::Pause(_)
            | MondrianMessage::PauseModule(_, _)
            | MondrianMessage::UpdatedWindows(..)
            | MondrianMessage::CoreUpdateStart(..)
            | MondrianMessage::CoreUpdateError
            | MondrianMessage::CoreUpdateComplete
            | MondrianMessage::QueryInfoResponse { .. }
            | MondrianMessage::OpenLogFolder
            | MondrianMessage::About
            | MondrianMessage::Quit => Err(()),
        }
    }
}
