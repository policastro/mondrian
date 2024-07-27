use std::collections::HashSet;

use serde::{Deserialize, Deserializer};

use super::structs::direction::Direction;

#[derive(Debug, PartialEq, Clone)]
pub enum MondrianMessage {
    RefreshConfig,
    OpenConfig,
    Retile,
    Configure,
    Focus(Direction),
    Move(Direction),
    Release(Option<bool>),
    Resize(Direction, u8),
    Invert,
    Pause(Option<bool>),
    PauseModule(String, Option<bool>),
    UpdatedWindows(HashSet<isize>),
    Minimize,
    ListManagedWindows,
    Quit,
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
            "resize <left|right|up|down> <40-250> ",
            "invert",
            "release [on|off|toggle]",
            "pause [on|off|toggle]",
            "module <keybindings|overlays> [on|off|toggle]",
            "quit",
        ];

        let s: String = Deserialize::deserialize(d)?;
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
            "release" => parts.len() <= 2,
            "pause" => parts.len() <= 2,
            "module" => parts.len() > 1 && parts.len() <= 3,
            "quit" => parts.len() == 1,
            _ => false,
        };

        let msg = format!("Invalid action: {}, valid actions are: {:?}", s, actions.join(", "));
        let invalid_action_err: Result<Self, D::Error> = Err(serde::de::Error::custom(msg));

        if !valid_len {
            return invalid_action_err;
        }

        match parts[0] {
            "refresh-config" => Ok(MondrianMessage::RefreshConfig),
            "open-config" => Ok(MondrianMessage::OpenConfig),
            "retile" => Ok(MondrianMessage::Retile),
            "minimize" => Ok(MondrianMessage::Minimize),
            "focus" => {
                let dir = match parts[1] {
                    "up" => Direction::Up,
                    "down" => Direction::Down,
                    "left" => Direction::Left,
                    "right" => Direction::Right,
                    _ => return invalid_action_err,
                };
                Ok(MondrianMessage::Focus(dir))
            }
            "move" => {
                let dir = match parts[1] {
                    "up" => Direction::Up,
                    "down" => Direction::Down,
                    "left" => Direction::Left,
                    "right" => Direction::Right,
                    _ => return invalid_action_err,
                };
                Ok(MondrianMessage::Move(dir))
            }
            "resize" => {
                let dir = match parts[1] {
                    "up" => Direction::Up,
                    "down" => Direction::Down,
                    "left" => Direction::Left,
                    "right" => Direction::Right,
                    _ => return invalid_action_err,
                };
                let size: u8 = match parts[2].parse() {
                    Ok(s) => s,
                    Err(_) => return invalid_action_err,
                };
                if !(40..=250).contains(&size) {
                    return invalid_action_err;
                }
                Ok(MondrianMessage::Resize(dir, size))
            }
            "invert" => Ok(MondrianMessage::Invert),
            "release" => {
                let param = match parts.get(1).to_owned() {
                    Some(&"on") => Some(true),
                    Some(&"off") => Some(false),
                    Some(&"toggle") => None,
                    None => None,
                    _ => Err(serde::de::Error::custom(
                        "Invalid parameter for 'release', expected 'on', 'off', 'toggle' or no parameter",
                    ))?,
                };
                Ok(MondrianMessage::Release(param))
            }
            "pause" => {
                let param = match parts.get(1).to_owned() {
                    Some(&"on") => Some(false),
                    Some(&"off") => Some(true),
                    Some(&"toggle") | None => None,
                    _ => Err(serde::de::Error::custom(
                        "Invalid parameter for 'pause', expected 'on', 'off', 'toggle' or no parameter",
                    ))?,
                };
                Ok(MondrianMessage::Pause(param))
            }
            "module" => {
                let name = match parts.get(1).to_owned() {
                    Some(&"keybindings") => "keybindings",
                    Some(&"overlays") => "overlays",
                    _ => Err(serde::de::Error::custom(
                        "Invalid parameter for 'module', expected 'keybindings' or 'overlays'",
                    ))?,
                };

                let param = match parts.get(2).to_owned() {
                    Some(&"on") => Some(false),
                    Some(&"off") => Some(true),
                    Some(&"toggle") | None => None,
                    _ => Err(serde::de::Error::custom(
                        "Invalid parameter for 'module', expected 'on', 'off', 'toggle' or no parameter",
                    ))?,
                };
                Ok(MondrianMessage::PauseModule(name.to_lowercase(), param))
            }
            "quit" => Ok(MondrianMessage::Quit),
            _ => invalid_action_err,
        }
    }
}
