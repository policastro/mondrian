use std::collections::HashMap;

use crate::app::{config::app_configs::AppConfigs, mondrian_command::MondrianMessage};
use inputbot::KeybdKey::*;
use serde::{de::Error, Deserialize, Deserializer, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default, try_from = "ExtKeybindingsConfig", into = "ExtKeybindingsConfig")]
pub struct KeybindingsModuleConfigs {
    pub enabled: bool,
    pub bindings: Vec<(Vec<inputbot::KeybdKey>, inputbot::KeybdKey, MondrianMessage)>,
}

impl KeybindingsModuleConfigs {
    pub fn get_grouped_bindings(
        &self,
    ) -> HashMap<inputbot::KeybdKey, Vec<(Vec<inputbot::KeybdKey>, MondrianMessage, bool)>> {
        let mut bindings = self.bindings.clone();
        bindings.sort_by(|(m0, _, _), (m1, _, _)| m1.len().cmp(&m0.len()));
        bindings.iter().fold(HashMap::new(), |mut acc, (m, k, a)| {
            let keep_alive = matches!(a, MondrianMessage::Pause(_) | MondrianMessage::PauseModule(_, _));
            acc.entry(*k).or_default().push((m.clone(), a.clone(), keep_alive));
            acc
        })
    }
}

impl From<&AppConfigs> for KeybindingsModuleConfigs {
    fn from(app_configs: &AppConfigs) -> Self {
        app_configs.modules.keybindings.clone()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(default)]
struct ExtKeybindingsConfig {
    enabled: bool,
    #[serde(deserialize_with = "deserialize_modifier")]
    default_modifier: Vec<String>,
    bindings: Vec<ExtBindingConfig>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct ExtBindingConfig {
    #[serde(default, deserialize_with = "deserialize_modifier_opt")]
    pub modifier: Option<Vec<String>>,
    #[serde(deserialize_with = "deserialize_key")]
    pub key: String,
    pub action: MondrianMessage,
}

impl Default for ExtKeybindingsConfig {
    fn default() -> Self {
        ExtKeybindingsConfig {
            enabled: false,
            default_modifier: vec!["ALT".to_string()],
            bindings: vec![],
        }
    }
}

impl From<ExtKeybindingsConfig> for KeybindingsModuleConfigs {
    fn from(val: ExtKeybindingsConfig) -> Self {
        let bindings = val
            .bindings
            .clone()
            .into_iter()
            .filter_map(|b| {
                parse_binding(
                    b.modifier.clone().unwrap_or(val.default_modifier.clone()),
                    b.key.clone(),
                    b.action,
                )
            })
            .collect();

        KeybindingsModuleConfigs {
            enabled: val.enabled,
            bindings,
        }
    }
}

impl From<KeybindingsModuleConfigs> for ExtKeybindingsConfig {
    fn from(val: KeybindingsModuleConfigs) -> Self {
        ExtKeybindingsConfig {
            enabled: val.enabled,
            default_modifier: ["ALT".to_string()].to_vec(),
            bindings: [].to_vec(),
        }
    }
}

fn deserialize_modifier<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let res = deserialize_modifier_opt(deserializer)?.ok_or(D::Error::missing_field("modifier"))?;
    Ok(res)
}

fn deserialize_modifier_opt<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let valid_modifiers = ["ALT", "CTRL", "SHIFT", "WIN"];

    let s: Option<String> = Option::deserialize(deserializer)?;
    let s = match s {
        Some(s) => s,
        None => return Ok(None),
    };
    let mut keys: Vec<String> = s.trim().split('+').map(|key| key.trim().to_uppercase()).collect();
    keys.sort();
    keys.dedup();
    let is_valid = keys.iter().all(|key| valid_modifiers.contains(&key.as_str()));
    match is_valid {
        true => Ok(Some(keys)),
        false => Err(D::Error::custom(format!("Invalid modifier: {}", s))),
    }
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let is_char = regex::Regex::new(r"^[A-Za-z\d]$").unwrap();
    let is_dir = regex::Regex::new(r"^\b(?i)left|right|up|down\b$").unwrap();
    let s: String = String::deserialize(deserializer)?;
    let is_valid = is_char.is_match(&s.to_uppercase()) || is_dir.is_match(&s.to_uppercase());
    match is_valid {
        true => Ok(s.to_uppercase()),
        false => Err(D::Error::custom(format!("Invalid key: {}", s))),
    }
}

fn parse_binding(
    modifiers: Vec<String>,
    key: String,
    command: MondrianMessage,
) -> Option<(Vec<inputbot::KeybdKey>, inputbot::KeybdKey, MondrianMessage)> {
    let modifiers_input: Vec<inputbot::KeybdKey> = modifiers
        .into_iter()
        .filter_map(|m| match m.to_uppercase().as_str() {
            "ALT" => Some(LAltKey),
            "CTRL" => Some(LControlKey),
            "SHIFT" => Some(LShiftKey),
            "WIN" => Some(LSuper),
            _ => None,
        })
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
