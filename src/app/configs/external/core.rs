use super::general::FloatingWinsSizeStrategyLabel;
use crate::app::structs::win_matcher::WinMatcher;
use serde::de;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
pub struct Core {
    #[serde(default)]
    pub ignore_rules: Vec<WinMatcher>,
    #[serde(default, deserialize_with = "deserialize_rules")]
    pub rules: Vec<WindowRule>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WindowRule {
    pub filter: WinMatcher,
    pub behavior: WindowBehavior,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(untagged)]
pub enum WindowBehaviorRaw {
    Shortcut(String),
    Full(WindowBehavior),
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum WindowBehavior {
    Ignore,
    Float {
        topmost: Option<bool>,
        size: Option<FloatingWinsSizeStrategyLabel>,
        size_ratio: Option<(f32, f32)>,
        size_fixed: Option<(u16, u16)>,
    },
    Insert {
        monitor: String,
    },
}

impl TryFrom<WindowBehaviorRaw> for WindowBehavior {
    type Error = String;
    fn try_from(value: WindowBehaviorRaw) -> Result<Self, Self::Error> {
        match value {
            WindowBehaviorRaw::Shortcut(s) => match s.as_str() {
                "ignore" => Ok(WindowBehavior::Ignore),
                "float" => Ok(WindowBehavior::Float {
                    topmost: None,
                    size: None,
                    size_ratio: None,
                    size_fixed: None,
                }),
                _ => Err(format!("Unknown behavior: {}", s)),
            },
            WindowBehaviorRaw::Full(w) => match w {
                WindowBehavior::Float {
                    size_ratio, size_fixed, ..
                } => {
                    if size_ratio.is_some_and(|(w, h)| w < 0.1 || h < 0.1 || w > 1.0 || h > 1.0) {
                        Err("Width and height must be between 0.1 and 1.0".to_string())
                    } else if size_fixed.is_some_and(|(w, h)| w < 100 || h < 100 || w > 10000 || h > 10000) {
                        Err("Width and height must be between 100 and 10000".to_string())
                    } else {
                        Ok(w)
                    }
                }
                _ => Ok(w),
            },
        }
    }
}

impl WindowRule {
    pub fn new(filter: WinMatcher, behavior: WindowBehavior) -> Self {
        WindowRule { filter, behavior }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
struct WindowRuleExternal {
    pub filter: WinMatcher,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_behavior"
    )]
    pub behavior: Option<WindowBehavior>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "deserialize_behaviors"
    )]
    pub behaviors: Option<Vec<WindowBehavior>>,
}

fn deserialize_rules<'de, D>(deserializer: D) -> Result<Vec<WindowRule>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Vec<WindowRuleExternal> = serde::Deserialize::deserialize(deserializer)?;
    let mut rules_config: Vec<WindowRule> = Vec::new();
    for rule in &value {
        if rule.behavior.is_none() && rule.behaviors.is_none() {
            return Err(de::Error::custom("A rule must have at least one action"));
        }
        if rule.behavior.is_some() && rule.behaviors.is_some() {
            return Err(de::Error::custom("A rule cannot have both action and actions fields"));
        }

        if let Some(behaviors) = rule.behaviors.as_ref() {
            let new_rules = behaviors
                .iter()
                .map(|b| WindowRule::new(rule.filter.clone(), b.clone()));
            rules_config.extend(new_rules);
        } else if let Some(behavior) = rule.behavior.as_ref() {
            rules_config.push(WindowRule::new(rule.filter.clone(), behavior.clone()));
        }
    }

    Ok(rules_config)
}

fn deserialize_behavior<'de, D>(deserializer: D) -> Result<Option<WindowBehavior>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: WindowBehaviorRaw = serde::Deserialize::deserialize(deserializer)?;
    let value: WindowBehavior = value.try_into().map_err(de::Error::custom)?;
    Ok(Some(value))
}

fn deserialize_behaviors<'de, D>(deserializer: D) -> Result<Option<Vec<WindowBehavior>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value: Vec<WindowBehaviorRaw> = serde::Deserialize::deserialize(deserializer)?;
    let mut values = Vec::new();
    for v in value {
        let value: WindowBehavior = v.try_into().map_err(de::Error::custom)?;
        values.push(value);
    }
    Ok(Some(values))
}
