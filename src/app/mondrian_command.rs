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
    Pause(bool),
    UpdatedWindows(HashSet<isize>),
    Minimize,
    Quit,
}

impl<'de> serde::Deserialize<'de> for MondrianMessage {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let actions = [
            "refresh-config",
            "open-config",
            "retile",
            "focus <left|right|up|down>",
            "move <left|right|up|down>",
            "minimize",
            "pause",
            "quit",
        ];

        let s: String = Deserialize::deserialize(d)?;
        let parts: Vec<&str> = s.split(' ').map(|s| s.trim()).collect();

        let valid_len = match parts[0] {
            "refresh-config" => parts.len() == 1,
            "open-config" => parts.len() == 1,
            "retile" => parts.len() == 1,
            "focus" => parts.len() == 2,
            "move" => parts.len() == 2,
            "pause" => parts.len() == 2,
            "minimize" => parts.len() == 1,
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
            "quit" => Ok(MondrianMessage::Quit),
            _ => invalid_action_err,
        }
    }
}
