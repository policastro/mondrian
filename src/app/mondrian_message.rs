use crate::modules::tiles_manager::lib::tm::command::TMCommand;
use crate::win32::window::window_ref::WindowRef;

use super::structs::area::Area;
use super::structs::direction::Direction;
use super::structs::info_entry::InfoEntry;
use super::structs::info_entry::InfoEntryIcon;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::str::FromStr;
use windows::Win32::Foundation::HWND;
use winvd::Desktop;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IntermonitorMoveOp {
    Swap,
    Insert,
    InsertFreeMove,
    Invert,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum IntramonitorMoveOp {
    Swap,
    InsertFreeMove,
    Invert,
}

impl IntramonitorMoveOp {
    pub fn calc(invert_mod: bool, free_mode_mod: bool) -> Self {
        // NOTE: precedence: invert_mod > free_mode_mod
        if invert_mod {
            return IntramonitorMoveOp::Invert;
        }

        match free_mode_mod {
            true => IntramonitorMoveOp::InsertFreeMove,
            false => IntramonitorMoveOp::Swap,
        }
    }
}

impl IntermonitorMoveOp {
    pub fn calc(
        default_insert_in_monitor: bool,
        default_free_move_in_monitor: bool,
        invert_mod: bool,
        insert_mod: bool,
        free_mode_mod: bool,
    ) -> Self {
        // NOTE: precedence: invert_mod > free_mode_mod > insert_mod
        if invert_mod {
            return IntermonitorMoveOp::Invert;
        }

        if default_insert_in_monitor {
            match (default_free_move_in_monitor, free_mode_mod, insert_mod) {
                (_, false, true) => IntermonitorMoveOp::Swap,
                (true, false, false) | (false, true, _) => IntermonitorMoveOp::InsertFreeMove,
                (false, false, false) | (true, true, _) => IntermonitorMoveOp::Insert,
            }
        } else {
            match (free_mode_mod, insert_mod) {
                (true, _) => IntermonitorMoveOp::InsertFreeMove,
                (false, true) => IntermonitorMoveOp::Insert,
                (false, false) => IntermonitorMoveOp::Swap,
            }
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum SystemEvent {
    Standby,
    Resume { logged_in: bool },
    SessionLocked,
    SessionUnlocked,
    SessionLogon,
    SessionLogoff,
    WorkareaChanged,
    MonitorsLayoutChanged,
    VirtualDesktopChanged { old: Desktop, new: Desktop },
    VirtualDesktopCreated { desktop: Desktop },
    VirtualDesktopRemoved { destroyed: Desktop, fallback: Desktop },
}

impl From<SystemEvent> for MondrianMessage {
    fn from(event: SystemEvent) -> Self {
        MondrianMessage::SystemEvent(event)
    }
}

impl SystemEvent {
    pub fn session_is_active(&self) -> bool {
        matches!(
            self,
            SystemEvent::Resume { logged_in: true } | SystemEvent::SessionLogon | SystemEvent::SessionUnlocked
        )
    }

    pub fn session_is_inactive(&self) -> bool {
        matches!(
            self,
            SystemEvent::Standby | SystemEvent::SessionLogoff | SystemEvent::SessionLocked
        )
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum WindowEvent {
    Opened(HWND),
    Closed(HWND),
    Minimized(HWND),
    Restored(HWND),
    Maximized(HWND),
    Unmaximized(HWND),
    Focused(HWND),
    StartMoveSize(HWND),
    EndMoveSize(HWND, MoveSizeResult),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MoveSizeResult {
    Resized(Area, Area),
    Moved((i32, i32), IntramonitorMoveOp, IntermonitorMoveOp),
    None,
}

impl WindowEvent {
    pub fn get_hwnd(&self) -> HWND {
        match self {
            WindowEvent::Opened(hwnd)
            | WindowEvent::Closed(hwnd)
            | WindowEvent::Minimized(hwnd)
            | WindowEvent::Restored(hwnd)
            | WindowEvent::Maximized(hwnd)
            | WindowEvent::Unmaximized(hwnd)
            | WindowEvent::Focused(hwnd)
            | WindowEvent::StartMoveSize(hwnd)
            | WindowEvent::EndMoveSize(hwnd, _) => *hwnd,
        }
    }
}

impl From<WindowEvent> for MondrianMessage {
    fn from(event: WindowEvent) -> Self {
        MondrianMessage::WindowEvent(event)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum WindowTileState {
    Maximized,
    Normal,
    Floating,
    Focalized,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MondrianMessage {
    RefreshConfig,
    OpenConfig,
    Retile,
    Configure,
    Focus(Direction),
    Move(Direction),
    MoveInsert(Direction),
    Insert(Direction),
    Release(Option<bool>),
    Peek(Direction, f32),
    Resize(Direction, u8),
    Invert,
    Pause(Option<bool>),
    PauseModule(String, Option<bool>),
    UpdatedWindows(HashMap<isize, WindowTileState>, TMCommand),
    CoreUpdateStart(HashSet<WindowRef>),
    CoreUpdateError,
    CoreUpdateComplete,
    Focalize,
    Amplify,
    Minimize,
    QueryInfo,
    QueryInfoResponse {
        name: String,
        icon: InfoEntryIcon,
        infos: Vec<InfoEntry>,
    },
    ListManagedWindows,
    OpenLogFolder,
    About,
    Quit,
    WindowEvent(WindowEvent),
    SystemEvent(SystemEvent),
}

impl<'de> serde::Deserialize<'de> for MondrianMessage {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let actions = [
            "refresh-config",
            "open-config",
            "retile",
            "minimize",
            "focus <left|right|up|down>",
            "insert <left|right|up|down>",
            "move <left|right|up|down>",
            "moveinsert <left|right|up|down>",
            "resize <left|right|up|down> <40-250>",
            "peek <left|right|up|down> <10-90>",
            "invert",
            "release",
            "focalize",
            "amplify",
            "dumpstateinfo",
            "pause [keybindings|overlays]",
            "quit",
        ];

        let s: String = Deserialize::deserialize(d)?;
        let s = s.to_lowercase();
        let parts: Vec<&str> = s.split(' ').map(|s| s.trim()).collect();

        let valid_len = match parts[0] {
            "refresh-config" => parts.len() == 1,
            "open-config" => parts.len() == 1,
            "retile" => parts.len() == 1,
            "minimize" => parts.len() == 1,
            "focus" => parts.len() == 2,
            "move" => parts.len() == 2,
            "insert" => parts.len() == 2,
            "moveinsert" => parts.len() == 2,
            "resize" => parts.len() == 3,
            "invert" => parts.len() == 1,
            "release" => parts.len() == 1,
            "peek" => parts.len() == 3,
            "focalize" => parts.len() == 1,
            "amplify" => parts.len() == 1,
            "dumpstateinfo" => parts.len() == 1,
            "pause" => parts.len() <= 2,
            "quit" => parts.len() == 1,
            _ => false,
        };

        let err = format!("Invalid action: {}, valid actions are: {:?}", s, actions.join(", "));
        if !valid_len {
            return Err(serde::de::Error::custom(err.clone()));
        }

        match parts[0] {
            "refresh-config" => Ok(MondrianMessage::RefreshConfig),
            "open-config" => Ok(MondrianMessage::OpenConfig),
            "retile" => Ok(MondrianMessage::Retile),
            "minimize" => Ok(MondrianMessage::Minimize),
            "focus" => {
                let dir = Direction::from_str(parts[1]).map_err(|_| serde::de::Error::custom(err))?;
                Ok(MondrianMessage::Focus(dir))
            }
            "insert" => {
                let dir = Direction::from_str(parts[1]).map_err(|_| serde::de::Error::custom(err))?;
                Ok(MondrianMessage::Insert(dir))
            }
            "move" => {
                let dir = Direction::from_str(parts[1]).map_err(|_| serde::de::Error::custom(err))?;
                Ok(MondrianMessage::Move(dir))
            }
            "moveinsert" => {
                let dir = Direction::from_str(parts[1]).map_err(|_| serde::de::Error::custom(err))?;
                Ok(MondrianMessage::MoveInsert(dir))
            }
            "resize" => {
                let dir = Direction::from_str(parts[1]).map_err(|_| serde::de::Error::custom(err.clone()))?;
                let size: u8 = parts[2].parse().map_err(|_| serde::de::Error::custom(err.clone()))?;
                match (40..=250).contains(&size) {
                    true => Ok(MondrianMessage::Resize(dir, size)),
                    false => Err(serde::de::Error::custom(err)),
                }
            }
            "peek" => {
                let dir = Direction::from_str(parts[1]).map_err(|_| serde::de::Error::custom(err.clone()))?;
                let ratio: u8 = parts[2].parse().map_err(|_| serde::de::Error::custom(err.clone()))?;
                let ratio = ratio as f32 / 100.0;
                match (0.1..=0.9).contains(&ratio) {
                    true => Ok(MondrianMessage::Peek(dir, ratio)),
                    false => Err(serde::de::Error::custom(err)),
                }
            }
            "invert" => Ok(MondrianMessage::Invert),
            "focalize" => Ok(MondrianMessage::Focalize),
            "amplify" => Ok(MondrianMessage::Amplify),
            "dumpstateinfo" => Ok(MondrianMessage::QueryInfo),
            "release" => Ok(MondrianMessage::Release(None)),
            "pause" => {
                let command = match parts.get(1).to_owned() {
                    Some(v) if *v == "keybindings" => MondrianMessage::PauseModule(v.to_lowercase(), None),
                    Some(v) if *v == "overlays" => MondrianMessage::PauseModule(v.to_lowercase(), None),
                    _ => MondrianMessage::Pause(None),
                };
                Ok(command)
            }
            "quit" => Ok(MondrianMessage::Quit),
            _ => Err(serde::de::Error::custom(err)),
        }
    }
}

impl Serialize for MondrianMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // TODO: does not include all possible actions
        match self {
            MondrianMessage::RefreshConfig => serializer.serialize_str("refresh-config"),
            MondrianMessage::OpenConfig => serializer.serialize_str("open-config"),
            MondrianMessage::Retile => serializer.serialize_str("retile"),
            MondrianMessage::Minimize => serializer.serialize_str("minimize"),
            MondrianMessage::Focus(dir) => serializer.serialize_str(&format!("focus {}", dir)),
            MondrianMessage::Insert(dir) => serializer.serialize_str(&format!("insert {}", dir)),
            MondrianMessage::Move(dir) => serializer.serialize_str(&format!("move {}", dir)),
            MondrianMessage::MoveInsert(dir) => serializer.serialize_str(&format!("moveinsert {}", dir)),
            MondrianMessage::Resize(dir, size) => serializer.serialize_str(&format!("resize {} {}", dir, size)),
            MondrianMessage::Invert => serializer.serialize_str("invert"),
            MondrianMessage::Release(_) => serializer.serialize_str("release"),
            MondrianMessage::Focalize => serializer.serialize_str("focalize"),
            MondrianMessage::Pause(_) => serializer.serialize_str("pause"),
            MondrianMessage::PauseModule(v, _) => serializer.serialize_str(&format!("pause {}", v)),
            MondrianMessage::Quit => serializer.serialize_str("quit"),
            _ => Err(serde::ser::Error::custom("Unsupported action")),
        }
    }
}
