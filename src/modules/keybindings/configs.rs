use crate::app::{config::app_configs::AppConfigs, mondrian_message::MondrianMessage};
use inputbot::KeybdKey::*;
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

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
                let key = parse_key(b.key.clone().trim().to_uppercase());
                let modifiers = match key.is_some_and(|k| {
                    matches!(
                        k,
                        F1Key
                            | F2Key
                            | F3Key
                            | F4Key
                            | F5Key
                            | F6Key
                            | F7Key
                            | F8Key
                            | F9Key
                            | F10Key
                            | F11Key
                            | F12Key
                            | F13Key
                            | F14Key
                            | F15Key
                            | F16Key
                            | F17Key
                            | F18Key
                            | F19Key
                            | F20Key
                            | F21Key
                            | F22Key
                            | F23Key
                            | F24Key
                    )
                }) {
                    true => parse_modifiers(b.modifier.clone().unwrap_or_default()),
                    false => parse_modifiers(b.modifier.clone().unwrap_or(val.default_modifier.clone())),
                };

                match (modifiers, key) {
                    (Some(m), Some(k)) => Some((m, k, b.action)),
                    _ => None,
                }
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
    let is_fn_key = regex::Regex::new(r"^\b(?i)F[0-9][0-9]?\b$").unwrap();
    let s: String = String::deserialize(deserializer)?;
    let s = s.trim().to_uppercase();
    let is_valid_fn = is_fn_key.is_match(&s) && s[1..].parse::<u8>().is_ok_and(|v| v > 0 && v <= 24);
    let is_valid = is_char.is_match(&s.to_uppercase()) || is_dir.is_match(&s.to_uppercase()) || is_valid_fn;
    match is_valid {
        true => Ok(s.to_uppercase()),
        false => Err(D::Error::custom(format!("Invalid key: {}", s))),
    }
}
fn parse_modifiers(modifiers: Vec<String>) -> Option<Vec<inputbot::KeybdKey>> {
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

    Some(modifiers_input)
}

fn parse_key(key: String) -> Option<inputbot::KeybdKey> {
    let key_input: Option<inputbot::KeybdKey> = if key.len() == 1 {
        let key = key.chars().next().unwrap();
        inputbot::get_keybd_key(key)
    } else {
        match key.to_uppercase().as_str() {
            "LEFT" => Some(LeftKey),
            "RIGHT" => Some(RightKey),
            "UP" => Some(UpKey),
            "DOWN" => Some(DownKey),
            "F1" => Some(F1Key),
            "F2" => Some(F2Key),
            "F3" => Some(F3Key),
            "F4" => Some(F4Key),
            "F5" => Some(F5Key),
            "F6" => Some(F6Key),
            "F7" => Some(F7Key),
            "F8" => Some(F8Key),
            "F9" => Some(F9Key),
            "F10" => Some(F10Key),
            "F11" => Some(F11Key),
            "F12" => Some(F12Key),
            "F13" => Some(F13Key),
            "F14" => Some(F14Key),
            "F15" => Some(F15Key),
            "F16" => Some(F16Key),
            "F17" => Some(F17Key),
            "F18" => Some(F18Key),
            "F19" => Some(F19Key),
            "F20" => Some(F20Key),
            "F21" => Some(F21Key),
            "F22" => Some(F22Key),
            "F23" => Some(F23Key),
            "F24" => Some(F24Key),
            _ => None,
        }
    };

    key_input
}
