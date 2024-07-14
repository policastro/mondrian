use serde::{Deserialize, Deserializer};

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MondrianCommand {
    RefreshConfig,
    OpenConfig,
    Retile,
    Pause(bool),
    Quit,
}

impl<'de> serde::Deserialize<'de> for MondrianCommand {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let s: String = Deserialize::deserialize(d)?;
        let parts: Vec<&str> = s.splitn(2, ' ').collect();
    
        match parts[0] {
            "refresh-config" => Ok(MondrianCommand::RefreshConfig),
            "open-config" => Ok(MondrianCommand::OpenConfig),
            "retile" => Ok(MondrianCommand::Retile),
            "quit" => Ok(MondrianCommand::Quit),
            _ => {
                let actions = ["refresh-config", "open-config", "retile", "pause", "quit"];
                let msg = format!("Invalid action: {}, valid actions are: {:?}", s, actions.join(", "));
                return Err(serde::de::Error::custom(msg));
            }
        }
    }
}