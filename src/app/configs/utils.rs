use super::{external, MonitorConfig};
use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use std::collections::HashMap;

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
) -> HashMap<String, MonitorConfig> {
    ext_monitors_config
        .iter()
        .map(|(k, c)| {
            let l = c.layout.clone();
            let layout_strategy = get_layout_strategy(
                &l.tiling_strategy.unwrap_or(ext_layout.tiling_strategy.clone()),
                &ext_layout.strategy,
            );

            let config = MonitorConfig {
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
            };

            (k.clone(), config)
        })
        .collect()
}
