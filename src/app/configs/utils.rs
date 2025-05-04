use super::{
    external::{
        self,
        layout_optional::{LayoutOptional, PaddingsOptionalConfigs},
    },
    LayoutConfig, MonitorConfig, WorkspaceConfig,
};
use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use std::collections::{HashMap, HashSet};

pub fn get_layout_strategy(strategy_str: &str, strategies: &external::layout::StrategyConfigs) -> LayoutStrategyEnum {
    match strategy_str {
        "horizontal" => strategies.horizontal.into(),
        "vertical" => strategies.vertical.into(),
        "twostep" => strategies.twostep.into(),
        "squared" => strategies.squared.clone().into(),
        _ => strategies.golden_ratio.into(),
    }
}

pub fn get_monitors_config(
    ext_monitors_config: &HashMap<String, external::monitors::MonitorConfig>,
    ext_layout: &external::layout::Layout,
    default_workspace: &str,
) -> HashMap<String, MonitorConfig> {
    ext_monitors_config
        .iter()
        .map(|(k, c)| {
            let layout = get_layout_config(&c.layout, ext_layout);
            (
                k.clone(),
                MonitorConfig {
                    layout,
                    default_workspace: c.default_workspace.clone().unwrap_or(default_workspace.to_string()),
                },
            )
        })
        .collect()
}

pub fn get_workspaces_config(
    ext_workspaces_config: &HashMap<String, external::workspaces::WorkspaceConfig>,
    ext_monitors_config: &HashMap<String, external::monitors::MonitorConfig>,
    ext_layout: &external::layout::Layout,
) -> HashMap<String, WorkspaceConfig> {
    let mut workspaces: HashMap<_, _> = HashMap::new();
    for (ws_id, ws_config) in ext_workspaces_config.iter() {
        let mut monitors: HashMap<_, _> = HashMap::new();

        let monitors_names: HashSet<String> = ws_config
            .monitors
            .keys()
            .chain(ext_monitors_config.keys())
            .cloned()
            .collect();
        for monitor_name in monitors_names.iter() {
            let workspace_layout_config = &ws_config.layout;
            let workspace_monitor_layout_config = ws_config.monitors.get(monitor_name);
            let monitor_layout_config = ext_monitors_config.get(monitor_name).map(|c| &c.layout);
            let general_layout_config = ext_layout;

            let opt_layout = workspace_monitor_layout_config
                .map(|c| merge_layout_optional_config(workspace_layout_config, c))
                .unwrap_or(workspace_layout_config.clone());

            let opt_layout = monitor_layout_config
                .map(|c| merge_layout_optional_config(c, &opt_layout))
                .unwrap_or(opt_layout);

            let layout = get_layout_config(&opt_layout, general_layout_config);

            monitors.insert(monitor_name.clone(), layout);
        }

        workspaces.insert(
            ws_id.clone(),
            WorkspaceConfig {
                layout: get_layout_config(&ws_config.layout, ext_layout),
                bind_to_monitor: ws_config.bind_to_monitor.clone(),
                monitors,
            },
        );
    }

    workspaces
}

fn merge_layout_optional_config(
    base_layout_optional: &external::layout_optional::LayoutOptional,
    layout_optional: &external::layout_optional::LayoutOptional,
) -> LayoutOptional {
    let l = layout_optional;
    LayoutOptional {
        tiling_strategy: l
            .tiling_strategy
            .clone()
            .or(base_layout_optional.tiling_strategy.clone()),
        paddings: PaddingsOptionalConfigs {
            tiles: l.paddings.tiles.or(base_layout_optional.paddings.tiles),
            borders: l.paddings.borders.or(base_layout_optional.paddings.borders),
        },
        focalized_padding: l.focalized_padding.or(base_layout_optional.focalized_padding),
        half_focalized_paddings: PaddingsOptionalConfigs {
            tiles: l
                .half_focalized_paddings
                .tiles
                .or(base_layout_optional.half_focalized_paddings.tiles),
            borders: l
                .half_focalized_paddings
                .borders
                .or(base_layout_optional.half_focalized_paddings.borders),
        },
    }
}

fn get_layout_config(
    ext_layout_optional: &external::layout_optional::LayoutOptional,
    ext_layout: &external::layout::Layout,
) -> LayoutConfig {
    let l = ext_layout_optional;
    let layout_strategy = get_layout_strategy(
        l.tiling_strategy
            .as_ref()
            .unwrap_or(&ext_layout.tiling_strategy.clone()),
        &ext_layout.strategy,
    );

    LayoutConfig {
        layout_strategy,
        tiles_padding: l.paddings.tiles.unwrap_or(ext_layout.paddings.tiles),
        borders_padding: l.paddings.borders.unwrap_or(ext_layout.paddings.borders),
        focalized_padding: l.focalized_padding.unwrap_or(ext_layout.focalized_padding),
        half_focalized_borders_pad: l
            .half_focalized_paddings
            .borders
            .unwrap_or(ext_layout.half_focalized_paddings.borders),
        half_focalized_tiles_pad: l
            .half_focalized_paddings
            .tiles
            .unwrap_or(ext_layout.half_focalized_paddings.tiles),
    }
}

impl From<external::layout::Layout> for LayoutOptional {
    fn from(value: external::layout::Layout) -> Self {
        external::layout_optional::LayoutOptional {
            tiling_strategy: Some(value.tiling_strategy),
            paddings: external::layout_optional::PaddingsOptionalConfigs {
                tiles: Some(value.paddings.tiles),
                borders: Some(value.paddings.borders),
            },
            focalized_padding: Some(value.focalized_padding),
            half_focalized_paddings: external::layout_optional::PaddingsOptionalConfigs {
                tiles: Some(value.half_focalized_paddings.tiles),
                borders: Some(value.half_focalized_paddings.borders),
            },
        }
    }
}
