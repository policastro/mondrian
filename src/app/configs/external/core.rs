use crate::app::structs::win_matcher::WinMatcher;
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
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum WindowBehavior {
    Ignore,
    Float,
    Insert { monitor: String },
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
    pub behavior: Option<WindowBehavior>,
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
            return Err(serde::de::Error::custom("A rule must have at least one action"));
        }
        if rule.behavior.is_some() && rule.behaviors.is_some() {
            return Err(serde::de::Error::custom(
                "A rule cannot have both action and actions fields",
            ));
        }

        if let Some(actions) = rule.behaviors.as_ref() {
            rules_config.extend(
                actions
                    .iter()
                    .map(|a| WindowRule {
                        filter: rule.filter.clone(),
                        behavior: a.clone(),
                    })
                    .collect::<Vec<WindowRule>>(),
            );
        } else if let Some(action) = rule.behavior.as_ref() {
            rules_config.push(WindowRule {
                filter: rule.filter.clone(),
                behavior: action.clone(),
            });
        }
    }

    Ok(rules_config)
}
