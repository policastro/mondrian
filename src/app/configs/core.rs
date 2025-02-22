use crate::app::structs::win_matcher::WinMatcher;
use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Core {
    pub rules: Vec<RuleConfig>,
    #[serde(default)]
    pub move_cursor_on_focus: bool,
    #[serde(default)]
    pub auto_reload_configs: bool,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
pub struct RuleConfig {
    pub classname: Option<String>,
    pub exename: Option<String>,
    pub title: Option<String>,
}

impl From<&Vec<RuleConfig>> for WinMatcher {
    fn from(rules: &Vec<RuleConfig>) -> Self {
        let matchers: Vec<WinMatcher> = rules
            .iter()
            .map(|r| {
                let mut matchers: Vec<WinMatcher> = Vec::new();

                if let Some(exename) = &r.exename {
                    matchers.push(WinMatcher::Exename(exename.clone()));
                }

                if let Some(classname) = &r.classname {
                    matchers.push(WinMatcher::Classname(classname.clone()));
                }

                if let Some(title) = &r.title {
                    matchers.push(WinMatcher::Title(title.clone()));
                }

                if matchers.is_empty() {
                    panic!("The filter must specify at least one field between 'exename', 'classname' and 'title'.")
                }

                WinMatcher::All(matchers)
            })
            .collect();

        WinMatcher::Any(matchers)
    }
}
