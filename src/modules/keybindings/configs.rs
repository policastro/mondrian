use crate::app::{config::app_configs::AppConfigs, mondrian_command::MondrianCommand};
use inputbot::KeybdKey::*;
pub struct KeybindingsConfig {
    pub bindings: Vec<(Vec<inputbot::KeybdKey>, inputbot::KeybdKey, MondrianCommand)>,
}

impl Default for KeybindingsConfig {
    fn default() -> Self {
        KeybindingsConfig { bindings: vec![] }
    }
}

impl From<&AppConfigs> for KeybindingsConfig {
    fn from(app_configs: &AppConfigs) -> Self {
        let bindings = app_configs
            .bindings
            .clone()
            .iter()
            .map(|b| parse_binding(b.modifier.clone().unwrap_or_default(), b.key.clone(), b.action.clone()))
            .flatten()
            .collect();

        KeybindingsConfig { bindings }
    }
}

fn parse_binding(
    modifiers: Vec<String>,
    key: String,
    command: MondrianCommand,
) -> Option<(Vec<inputbot::KeybdKey>, inputbot::KeybdKey, MondrianCommand)> {
    let modifiers_input: Vec<inputbot::KeybdKey> = modifiers
        .into_iter()
        .map(|m| match m.to_uppercase().as_str() {
            "ALT" => Some(LAltKey),
            "CTRL" => Some(LControlKey),
            "SHIFT" => Some(LShiftKey),
            "WIN" => Some(LSuper),
            _ => None,
        })
        .filter(|m| m.is_some())
        .map(|m| m.unwrap())
        .collect();

    let key_input: Option<inputbot::KeybdKey> = if key.len() == 1 {
        let key = key.chars().next().unwrap();
        inputbot::get_keybd_key(key)
    } else {
        match key.to_uppercase().as_str() {
            "LEFT" => Some(LeftKey),
            "RIGHT" => Some(RightKey),
            "UP" => Some(UpKey),
            "DOWN" => Some(DownKey),
            _ => None,
        }
    };

    match (key_input, modifiers_input.is_empty()) {
        (Some(key), false) => Some((modifiers_input, key, command)),
        _ => None,
    }
}
