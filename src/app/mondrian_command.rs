use serde::{Deserialize, Deserializer};

use super::structs::direction::Direction;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MondrianCommand {
    RefreshConfig,
    OpenConfig,
    Retile,
    Configure,
    Focus(Direction),
    Pause(bool),
    Quit,
}

impl<'de> serde::Deserialize<'de> for MondrianCommand {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let actions = [
            "refresh-config",
            "open-config",
            "retile",
            "focus <left|right|up|down>",
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
            "pause" => parts.len() == 2,
            "quit" => parts.len() == 1,
            _ => false,
        };

        let msg = format!("Invalid action: {}, valid actions are: {:?}", s, actions.join(", "));
        let invalid_action_err: Result<Self, D::Error> = Err(serde::de::Error::custom(msg));

        if !valid_len {
            return invalid_action_err;
        }

        match parts[0] {
            "refresh-config" => Ok(MondrianCommand::RefreshConfig),
            "open-config" => Ok(MondrianCommand::OpenConfig),
            "retile" => Ok(MondrianCommand::Retile),
            "focus" => {
                let dir = match parts[1] {
                    "up" => Direction::Up,
                    "down" => Direction::Down,
                    "left" => Direction::Left,
                    "right" => Direction::Right,
                    _ => return invalid_action_err,
                };
                Ok(MondrianCommand::Focus(dir))
            }
            "quit" => Ok(MondrianCommand::Quit),
            _ => invalid_action_err,
        }
    }
}
