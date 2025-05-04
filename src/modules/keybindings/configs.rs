use crate::app::{configs::AppConfig, mondrian_message::MondrianMessage};
use inputbot::KeybdKey::{self, *};
use serde::{de::Error, Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;

#[derive(Clone, Debug, Deserialize, Serialize, Default)]
#[serde(default, deny_unknown_fields)]
pub struct KeybindingsModuleConfigs {
    pub enabled: bool,
    pub bindings: Vec<Binding>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(try_from = "ExternalBinding", into = "ExternalBinding")]
pub struct Binding {
    modifiers: Vec<KeybdKey>,
    key: KeybdKey,
    action: MondrianMessage,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
struct ExternalBinding {
    #[serde(
        default,
        alias = "modifier",
        alias = "mod",
        deserialize_with = "deserialize_modifiers",
        serialize_with = "serialize_modifiers"
    )]
    pub modifiers: Vec<KeybdKey>,

    #[serde(deserialize_with = "deserialize_key", serialize_with = "serialize_key")]
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

impl TryFrom<ExternalBinding> for Binding {
    type Error = String;
    fn try_from(binding: ExternalBinding) -> Result<Binding, String> {
        Binding::new(binding.modifiers, binding.key, binding.action)
    }
}

impl From<Binding> for ExternalBinding {
    fn from(binding: Binding) -> Self {
        ExternalBinding {
            modifiers: binding.modifiers,
            key: binding.key,
            action: binding.action,
        }
    }
}

impl From<&AppConfig> for KeybindingsModuleConfigs {
    fn from(app_configs: &AppConfig) -> Self {
        app_configs.modules.keybindings.clone()
    }
}

fn deserialize_modifiers<'de, D>(deserializer: D) -> Result<Vec<KeybdKey>, D::Error>
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
    let parsed_keys: Vec<KeybdKey> = keys.iter().filter_map(|s| parse_modifier(s.clone())).collect();
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

// FIX: probably doesn't work as expected
fn serialize_modifiers<S>(modifiers: &[inputbot::KeybdKey], serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let keys: Vec<String> = modifiers.iter().map(|k| k.to_string()).collect();
    serializer.serialize_str(&keys.join("+"))
}

fn serialize_key<S>(key: &inputbot::KeybdKey, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&key.to_string())
}

fn parse_modifier(modifier: String) -> Option<inputbot::KeybdKey> {
    match modifier.trim().to_uppercase().as_str() {
        "ALT" => Some(LAltKey),
        "CTRL" => Some(LControlKey),
        "SHIFT" => Some(LShiftKey),
        "WIN" => Some(LSuper),
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
