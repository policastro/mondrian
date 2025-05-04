use super::layout_optional::LayoutOptional;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Deserialize, Serialize, Clone, Debug, Default, PartialEq)]
#[serde(deny_unknown_fields, default)]
pub struct WorkspaceConfig {
    pub bind_to_monitor: Option<String>,
    pub layout: LayoutOptional,
    pub monitors: HashMap<String, LayoutOptional>,
}
