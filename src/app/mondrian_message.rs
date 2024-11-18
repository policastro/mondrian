use super::structs::area::Area;
use super::structs::direction::Direction;
use crate::modules::tiles_manager::lib::tm_command::TMCommand;
use serde::Deserialize;
use serde::Deserializer;
use serde::Serialize;
use std::collections::HashSet;
use std::str::FromStr;
use windows::Win32::Foundation::HWND;

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
pub enum WindowEvent {
    Opened(HWND),
    Closed(HWND),
    Minimized(HWND),
    Restored(HWND),
    Maximized(HWND),
    Unmaximized(HWND),
    StartMoveSize(HWND),
    Moved(HWND, (i32, i32), IntramonitorMoveOp, IntermonitorMoveOp),
    Resized(HWND, Area, Area),
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
            | WindowEvent::StartMoveSize(hwnd)
            | WindowEvent::Moved(hwnd, _, _, _)
            | WindowEvent::Resized(hwnd, _, _) => *hwnd,
        }
    }
}

impl From<WindowEvent> for MondrianMessage {
    fn from(event: WindowEvent) -> Self {
        MondrianMessage::WindowEvent(event)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum MondrianMessage {
    RefreshConfig,
    OpenConfig,
    MonitorsLayoutChanged,
    Retile,
    Configure,
    Focus(Direction),
    Move(Direction),
    Release(Option<bool>),
    Resize(Direction, u8),
    Invert,
    Pause(Option<bool>),
    PauseModule(String, Option<bool>),
    UpdatedWindows(HashSet<isize>, TMCommand),
    CoreUpdateStart,
    CoreUpdateError,
    CoreUpdateComplete,
    Focalize,
    Minimize,
    ListManagedWindows,
    About,
    Quit,
    WindowEvent(WindowEvent),
}

impl<'de> serde::Deserialize<'de> for MondrianMessage {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let actions = [
            "refresh-config",
            "open-config",
            "retile",
            "minimize",
            "focus <left|right|up|down>",
            "move <left|right|up|down>",
            "resize <left|right|up|down> <40-250>",
            "invert",
            "release",
            "focalize",
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
            "resize" => parts.len() == 3,
            "invert" => parts.len() == 1,
            "release" => parts.len() == 1,
            "focalize" => parts.len() == 1,
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
            "move" => {
                let dir = Direction::from_str(parts[1]).map_err(|_| serde::de::Error::custom(err))?;
                Ok(MondrianMessage::Move(dir))
            }
            "resize" => {
                let dir = Direction::from_str(parts[1]).map_err(|_| serde::de::Error::custom(err.clone()))?;
                let size: u8 = parts[2].parse().map_err(|_| serde::de::Error::custom(err.clone()))?;
                match (40..=250).contains(&size) {
                    true => Ok(MondrianMessage::Resize(dir, size)),
                    false => Err(serde::de::Error::custom(err)),
                }
            }
            "invert" => Ok(MondrianMessage::Invert),
            "focalize" => Ok(MondrianMessage::Focalize),
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
        match self {
            MondrianMessage::RefreshConfig => serializer.serialize_str("refresh-config"),
            MondrianMessage::OpenConfig => serializer.serialize_str("open-config"),
            MondrianMessage::Retile => serializer.serialize_str("retile"),
            MondrianMessage::Minimize => serializer.serialize_str("minimize"),
            MondrianMessage::Focus(dir) => serializer.serialize_str(&format!("focus {}", dir)),
            MondrianMessage::Move(dir) => serializer.serialize_str(&format!("move {}", dir)),
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
