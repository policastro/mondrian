use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize, Default, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct General {
    #[serde(default)]
    pub history_based_navigation: bool,
}
