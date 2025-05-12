use crate::app::{configs::AppConfig, mondrian_message::MondrianMessage};
use inputbot::KeybdKey::{self, *};
use serde::{de::Error, Deserialize, Deserializer};
use std::collections::HashMap;
use std::fmt::Debug;

#[derive(Clone, Debug, Deserialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct KeybindingsModuleConfigs {
    pub enabled: bool,

    #[serde(deserialize_with = "deserialize_bindings")]
    pub bindings: Vec<Binding>,
}

#[derive(Clone, Debug)]
pub struct Binding {
    modifiers: Vec<KeybdKey>,
    key: KeybdKey,
    action: MondrianMessage,
}

#[derive(Deserialize, Clone, Debug)]
enum ModifierKey {
    Shift,
    Control,
    Alt,
    Win,
    LeftShift,
    RightShift,
    LeftControl,
    RightControl,
    LeftAlt,
    RightAlt,
    LeftWin,
    RightWin,
}

#[derive(Deserialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
struct ExternalBinding {
    #[serde(
        default,
        alias = "modifier",
        alias = "mod",
        deserialize_with = "deserialize_modifiers"
    )]
    pub modifiers: Vec<ModifierKey>,

    #[serde(deserialize_with = "deserialize_key")]
    pub key: KeybdKey,
    pub action: MondrianMessage,
}

impl KeybindingsModuleConfigs {
    pub fn group_by_key(&self) -> HashMap<inputbot::KeybdKey, Vec<Binding>> {
        let mut bindings = self.bindings.clone();
        bindings.sort_by(|b1, b2| b2.modifiers.len().cmp(&b1.modifiers.len()));
        bindings.iter().fold(HashMap::new(), |mut acc, b| {
            acc.entry(b.key).or_default().push(b.clone());
            acc
        })
    }
}

impl Binding {
    fn new(modifiers: Vec<KeybdKey>, key: KeybdKey, action: MondrianMessage) -> Result<Binding, String> {
        if !matches!(
            key,
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
        ) && modifiers.is_empty()
        {
            return Err("Modifiers can be empty only for function keys".to_string());
        }

        Ok(Binding { modifiers, key, action })
    }

    pub fn modifiers(&self) -> &Vec<KeybdKey> {
        &self.modifiers
    }

    pub fn key(&self) -> &KeybdKey {
        &self.key
    }

    pub fn action(&self) -> &MondrianMessage {
        &self.action
    }

    pub fn are_modifiers_pressed(&self) -> bool {
        self.modifiers.iter().all(|m| m.is_pressed())
    }
}

impl From<&AppConfig> for KeybindingsModuleConfigs {
    fn from(app_configs: &AppConfig) -> Self {
        app_configs.modules.keybindings.clone()
    }
}

fn cartesian_product<T: Clone>(lists: &[Vec<T>]) -> Vec<Vec<T>> {
    let mut results: Vec<Vec<T>> = vec![vec![]];

    for list in lists {
        let mut new_results = Vec::new();
        for result in &results {
            for item in list {
                let mut new_combination = result.clone();
                new_combination.push(item.clone());
                new_results.push(new_combination);
            }
        }
        results = new_results;
    }
    results
}

fn deserialize_bindings<'de, D>(deserializer: D) -> Result<Vec<Binding>, D::Error>
where
    D: Deserializer<'de>,
{
    let bindings: Vec<ExternalBinding> = Vec::deserialize(deserializer)?;
    let bindings: Vec<Binding> = bindings
        .iter()
        .flat_map(|b| {
            let modifiers: Vec<Vec<KeybdKey>> = b
                .modifiers
                .iter()
                .map(|m| match m {
                    ModifierKey::Alt => vec![LAltKey, RAltKey],
                    ModifierKey::Control => vec![LControlKey, RControlKey],
                    ModifierKey::Shift => vec![LShiftKey, RShiftKey],
                    ModifierKey::Win => vec![LSuper, RSuper],
                    ModifierKey::LeftAlt => vec![LAltKey],
                    ModifierKey::LeftControl => vec![LControlKey],
                    ModifierKey::LeftShift => vec![LShiftKey],
                    ModifierKey::LeftWin => vec![LSuper],
                    ModifierKey::RightAlt => vec![RAltKey],
                    ModifierKey::RightControl => vec![RControlKey],
                    ModifierKey::RightShift => vec![RShiftKey],
                    ModifierKey::RightWin => vec![RSuper],
                })
                .collect();

            let bindings: Vec<Binding> = cartesian_product(&modifiers)
                .iter()
                .filter_map(|p| Binding::new(p.clone(), b.key, b.action.clone()).ok())
                .collect();

            bindings
        })
        .collect();

    Ok(bindings)
}

fn deserialize_modifiers<'de, D>(deserializer: D) -> Result<Vec<ModifierKey>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = match Option::deserialize(deserializer)? {
        Some(s) => s,
        None => return Ok(vec![]),
    };

    let mut keys: Vec<String> = s
        .trim()
        .split('+')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    keys.sort();
    keys.dedup();
    let parsed_keys: Vec<ModifierKey> = keys.iter().filter_map(|s| parse_modifier(s.clone())).collect();
    if keys.len() != parsed_keys.len() {
        return Err(D::Error::custom(format!("Invalid modifiers: {}", s)));
    }
    Ok(parsed_keys)
}

fn deserialize_key<'de, D>(deserializer: D) -> Result<inputbot::KeybdKey, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = String::deserialize(deserializer)?;
    match parse_key(s.clone()) {
        Some(k) => Ok(k),
        None => Err(D::Error::custom(format!("Invalid key: {}", s))),
    }
}

fn parse_modifier(modifier: String) -> Option<ModifierKey> {
    match modifier.trim().to_uppercase().as_str() {
        "ALT" => Some(ModifierKey::Alt),
        "CTRL" => Some(ModifierKey::Control),
        "SHIFT" => Some(ModifierKey::Shift),
        "WIN" => Some(ModifierKey::Win),
        "LALT" => Some(ModifierKey::LeftAlt),
        "LCTRL" => Some(ModifierKey::LeftControl),
        "LSHIFT" => Some(ModifierKey::LeftShift),
        "LWIN" => Some(ModifierKey::LeftWin),
        "RALT" => Some(ModifierKey::RightAlt),
        "RCTRL" => Some(ModifierKey::RightControl),
        "RSHIFT" => Some(ModifierKey::RightShift),
        "RWIN" => Some(ModifierKey::RightWin),
        _ => None,
    }
}

fn parse_key(key: String) -> Option<inputbot::KeybdKey> {
    if key.len() == 1 {
        let key = key.chars().next().unwrap();
        inputbot::get_keybd_key(key)
    } else {
        match key.trim().to_uppercase().as_str() {
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
            "NUM0" => Some(Numpad0Key),
            "NUM1" => Some(Numpad1Key),
            "NUM2" => Some(Numpad2Key),
            "NUM3" => Some(Numpad3Key),
            "NUM4" => Some(Numpad4Key),
            "NUM5" => Some(Numpad5Key),
            "NUM6" => Some(Numpad6Key),
            "NUM7" => Some(Numpad7Key),
            "NUM8" => Some(Numpad8Key),
            "NUM9" => Some(Numpad9Key),
            _ => None,
        }
    }
}
