pub(crate) mod deserializers;
mod external;
pub(crate) mod floating;
mod modules;
pub(crate) mod rules;
mod utils;

use super::area_tree::layout_strategy::LayoutStrategyEnum;
use super::structs::paddings::Paddings;
use super::structs::win_matcher::WinMatcher;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimation;
use external::AppConfigExternal;
use floating::FloatingWinsConfig;
use modules::Modules;
use rules::extract_rules;
use rules::WindowRule;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AnimationsConfig {
    pub duration: u32,
    pub framerate: u8,
    pub animation_type: Option<WindowAnimation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayoutConfig {
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub borders_padding: Paddings,
    pub focalized_padding: Paddings,
    pub half_focalized_borders_pad: Paddings,
    pub half_focalized_tiles_pad: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorConfig {
    pub default_workspace: String,
    pub layout: LayoutConfig,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkspaceConfig {
    pub bind_to_monitor: Option<String>,
    pub layout: LayoutConfig,
    pub monitors: HashMap<String, LayoutConfig>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(try_from = "AppConfigExternal")]
pub struct AppConfig {
    pub history_based_navigation: bool,
    pub move_cursor_on_focus: bool,
    pub auto_reload_configs: bool,
    pub detect_maximized_windows: bool,
    pub insert_in_monitor: bool,
    pub free_move_in_monitor: bool,
    pub allow_focus_on_empty_monitor: bool,
    pub animations: AnimationsConfig,
    pub floating_wins_config: FloatingWinsConfig,
    pub default_workspace: String,
    pub ignore_filter: WinMatcher,
    pub rules: Vec<WindowRule>,
    pub tiles_pad: u8,
    pub borders_pads: Paddings,
    pub half_focalized_tiles_pad: u8,
    pub half_focalized_borders_pads: Paddings,
    pub focalized_pads: Paddings,
    pub layout_strategy: LayoutStrategyEnum,
    pub monitors_config: HashMap<String, MonitorConfig>,
    pub workspaces_config: HashMap<String, WorkspaceConfig>,
    pub modules: Modules,
}

impl Default for AppConfig {
    fn default() -> Self {
        let v: Result<AppConfig, _> = AppConfigExternal::default().try_into();
        v.expect("Invalid default config")
    }
}

impl TryFrom<AppConfigExternal> for AppConfig {
    type Error = String;
    fn try_from(v: AppConfigExternal) -> Result<Self, Self::Error> {
        if v.workspaces
            .get(&v.general.default_workspace)
            .is_some_and(|ws| ws.bind_to_monitor.is_some())
        {
            return Err("Default workspace cannot be bound to a monitor".to_string());
        }

        let floating_wins_config = v.general.floating_wins.into();
        let (ignore_filter, other_rules) = extract_rules(&v.core.ignore_rules, &v.core.rules, &v.general.floating_wins);
        let layout_strategy = utils::get_layout_strategy(&v.layout.tiling_strategy, &v.layout.strategy);
        let monitors_config = utils::get_monitors_config(&v.monitors, &v.layout, &v.general.default_workspace);
        let workspaces_config = utils::get_workspaces_config(&v.workspaces, &v.monitors, &v.layout);

        Ok(AppConfig {
            history_based_navigation: v.general.history_based_navigation,
            move_cursor_on_focus: v.general.move_cursor_on_focus,
            auto_reload_configs: v.general.auto_reload_configs,
            detect_maximized_windows: v.general.detect_maximized_windows,
            insert_in_monitor: v.general.insert_in_monitor,
            free_move_in_monitor: v.general.free_move_in_monitor,
            animations: AnimationsConfig {
                duration: v.general.animations.duration,
                framerate: v.general.animations.framerate,
                animation_type: match v.general.animations.enabled {
                    true => Some(v.general.animations.animation_type),
                    false => None,
                },
            },
            floating_wins_config,
            default_workspace: v.general.default_workspace,
            ignore_filter,
            rules: other_rules,
            tiles_pad: v.layout.paddings.tiles,
            borders_pads: v.layout.paddings.borders,
            half_focalized_tiles_pad: v.layout.half_focalized_paddings.tiles,
            half_focalized_borders_pads: v.layout.half_focalized_paddings.borders,
            focalized_pads: v.layout.focalized_padding,
            layout_strategy,
            monitors_config,
            workspaces_config,
            modules: v.modules,
            allow_focus_on_empty_monitor: v.general.allow_focus_on_empty_monitor,
        })
    }
}
