use crate::app::structs::win_matcher::WinMatcher;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Core {
    pub ignore_rules: Vec<RuleConfig>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RuleConfig {
    pub classname: Option<String>,
    pub exename: Option<String>,
    pub title: Option<String>,
    pub style: Option<String>,
}

impl From<&Vec<RuleConfig>> for WinMatcher {
    fn from(rules: &Vec<RuleConfig>) -> Self {
        let matchers: HashSet<WinMatcher> = rules
            .iter()
            .map(|r| {
                let mut matchers: HashSet<WinMatcher> = HashSet::new();

                if let Some(exename) = &r.exename {
                    matchers.insert(WinMatcher::Exename(exename.clone()));
                }

                if let Some(classname) = &r.classname {
                    matchers.insert(WinMatcher::Classname(classname.clone()));
                }

                if let Some(title) = &r.title {
                    matchers.insert(WinMatcher::Title(title.clone()));
                }

                if let Some(style) = &r.style {
                    matchers.insert(WinMatcher::Style(style.clone()));
                }

                if matchers.is_empty() {
                    panic!("The filter must specify at least one field between 'exename', 'classname', 'title' and 'style'.")
                }

                WinMatcher::All(matchers)
            })
            .collect();

        WinMatcher::Any(matchers)
    }
}
