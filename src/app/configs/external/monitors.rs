use serde::Deserialize;
use serde::Serialize;

use super::layout_optional::LayoutOptional;

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct MonitorConfig {
    pub default_workspace: Option<String>,
    pub layout: LayoutOptional,
}
