use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::app::configs::floating::FloatingWinsConfig;
use crate::app::configs::rules::WindowRule;
use crate::app::configs::AnimationsConfig;
use crate::app::configs::AppConfig;
use crate::app::configs::LayoutConfig;
use crate::app::configs::MonitorConfig;
use crate::app::configs::WorkspaceConfig;
use crate::app::structs::paddings::Paddings;
use crate::app::structs::win_matcher::WinMatcher;
use std::collections::HashMap;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct TilesManagerConfig {
    tiles_padding: i16,
    borders_padding: Paddings,
    focalized_padding: Paddings,
    half_focalized_borders_pad: Paddings,
    half_focalized_tiles_pad: i16,
    layout_strategy: LayoutStrategyEnum,
    monitors_configs: HashMap<String, MonitorConfig>,
    workspaces_configs: HashMap<String, WorkspaceConfig>,
    default_workspace: String,
    pub focus_on_empty_monitor: bool,
    pub focus_follows_cursor: bool,
    pub ignore_filter: WinMatcher,
    pub rules: Vec<WindowRule>,
    pub animation: AnimationsConfig,
    pub history_based_navigation: bool,
    pub floating_wins: FloatingWinsConfig,
}

impl TilesManagerConfig {
    pub fn get_layout_strategy(&self, monitor_name: &str, workspace: &str) -> LayoutStrategyEnum {
        self.extract_config(
            monitor_name,
            workspace,
            |c| c.layout_strategy.clone(),
            &self.layout_strategy,
        )
    }

    pub fn get_focalized_padding(&self, monitor_name: &str, workspace: &str) -> Paddings {
        self.extract_config(
            monitor_name,
            workspace,
            |c| c.focalized_padding,
            &self.focalized_padding,
        )
    }

    pub fn get_borders_padding(&self, monitor_name: &str, workspace: &str) -> Paddings {
        self.extract_config(monitor_name, workspace, |c| c.borders_padding, &self.borders_padding)
    }

    pub fn get_tiles_padding(&self, monitor_name: &str, workspace: &str) -> i16 {
        self.extract_config(monitor_name, workspace, |c| c.tiles_padding as i16, &self.tiles_padding)
    }

    pub fn get_half_focalized_borders_pad(&self, monitor_name: &str, workspace: &str) -> Paddings {
        self.extract_config(
            monitor_name,
            workspace,
            |c| c.half_focalized_borders_pad,
            &self.half_focalized_borders_pad,
        )
    }

    pub fn get_half_focalized_tiles_pad(&self, monitor_name: &str, workspace: &str) -> i16 {
        self.extract_config(
            monitor_name,
            workspace,
            |c| c.half_focalized_tiles_pad as i16,
            &self.half_focalized_tiles_pad,
        )
    }

    pub fn get_half_focalized_tiles_pad_xy(&self, monitor_name: &str, workspace: &str) -> (i16, i16) {
        let pad = self.get_half_focalized_tiles_pad(monitor_name, workspace);
        (pad, pad)
    }

    pub fn get_tiles_padding_xy(&self, monitor_name: &str, workspace: &str) -> (i16, i16) {
        let tiles_padding = self.get_tiles_padding(monitor_name, workspace);
        (tiles_padding, tiles_padding)
    }

    pub fn get_bounded_monitor(&self, workspace: &str) -> Option<String> {
        self.workspaces_configs
            .get(workspace)
            .and_then(|c| c.bind_to_monitor.clone())
    }

    pub fn get_default_workspace(&self, monitor_name: &str) -> String {
        self.monitors_configs
            .get(monitor_name)
            .map(|c| c.default_workspace.clone())
            .unwrap_or(self.default_workspace.clone())
    }

    fn extract_config<V: Clone>(
        &self,
        monitor_name: &str,
        workspace: &str,
        config_extractor: impl Fn(&LayoutConfig) -> V,
        default: &V,
    ) -> V {
        self.workspaces_configs
            .get(workspace)
            .and_then(|w| w.monitors.get(monitor_name).or(Some(&w.layout)).map(&config_extractor))
            .unwrap_or(
                self.monitors_configs
                    .get(monitor_name)
                    .map(|c| config_extractor(&c.layout))
                    .unwrap_or(default.clone()),
            )
    }
}

impl From<&AppConfig> for TilesManagerConfig {
    fn from(config: &AppConfig) -> Self {
        TilesManagerConfig {
            tiles_padding: config.tiles_pad as i16,
            borders_padding: config.borders_pads,
            focalized_padding: config.focalized_pads,
            half_focalized_borders_pad: config.half_focalized_borders_pads,
            half_focalized_tiles_pad: config.half_focalized_tiles_pad as i16,
            layout_strategy: config.layout_strategy.clone(),
            monitors_configs: config.monitors_config.clone(),
            workspaces_configs: config.workspaces_config.clone(),
            default_workspace: config.default_workspace.clone(),
            focus_on_empty_monitor: config.allow_focus_on_empty_monitor,
            focus_follows_cursor: config.move_cursor_on_focus,
            ignore_filter: config.ignore_filter.clone(),
            rules: config.rules.clone(),
            animation: config.animations.clone(),
            history_based_navigation: config.history_based_navigation,
            floating_wins: config.floating_wins_config,
        }
    }
}
