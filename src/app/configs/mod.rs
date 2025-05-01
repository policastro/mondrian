pub(crate) mod deserializers;
pub(crate) mod external;
pub(crate) mod modules;

use super::area_tree::layout_strategy::LayoutStrategyEnum;
use super::structs::paddings::Paddings;
use super::structs::win_matcher::WinMatcher;
use crate::modules::tiles_manager::lib::window_animation_player::WindowAnimation;
use external::AppConfigExternal;
use modules::Modules;
use serde::Deserialize;
use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Default, Debug, Clone, PartialEq)]
pub struct AnimationsConfig {
    pub duration: u32,
    pub framerate: u8,
    pub animation_type: Option<WindowAnimation>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatingWinsSizeStrategy {
    Preserve,
    Fixed { w: u16, h: u16 },
    Relative { w: f32, h: f32 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FloatingWinsConfig {
    pub topmost: bool,
    pub strategy: FloatingWinsSizeStrategy,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MonitorConfig {
    pub layout_strategy: LayoutStrategyEnum,
    pub tiles_padding: u8,
    pub borders_padding: Paddings,
    pub focalized_padding: Paddings,
    pub half_focalized_borders_pad: Paddings,
    pub half_focalized_tiles_pad: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowRule {
    pub filter: WinMatcher,
    pub behavior: WindowBehavior,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WindowBehavior {
    Float {
        topmost: bool,
        strategy: FloatingWinsSizeStrategy,
    },
    Insert {
        monitor: String,
    },
}

#[derive(Deserialize, Debug, Clone)]
#[serde(from = "AppConfigExternal")]
pub struct AppConfig {
    pub history_based_navigation: bool,
    pub move_cursor_on_focus: bool,
    pub auto_reload_configs: bool,
    pub detect_maximized_windows: bool,
    pub insert_in_monitor: bool,
    pub free_move_in_monitor: bool,
    pub animations: AnimationsConfig,
    pub floating_wins_config: FloatingWinsConfig,
    pub ignore_filter: WinMatcher,
    pub rules: Vec<WindowRule>,
    pub tiles_pad: u8,
    pub borders_pads: Paddings,
    pub half_focalized_tiles_pad: u8,
    pub half_focalized_borders_pads: Paddings,
    pub focalized_pads: Paddings,
    pub layout_strategy: LayoutStrategyEnum,
    pub monitors_config: HashMap<String, MonitorConfig>,
    pub modules: Modules,
}

impl Default for FloatingWinsConfig {
    fn default() -> Self {
        FloatingWinsConfig {
            topmost: false,
            strategy: FloatingWinsSizeStrategy::Relative { w: 0.5, h: 0.5 },
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfigExternal::default().into()
    }
}

fn extract_rules(
    ignore_filters: &[WinMatcher],
    rules: &[external::core::WindowRule],
    floating_wins: &FloatingWinsConfig,
) -> (WinMatcher, Vec<WindowRule>) {
    let (mut ignore_rules, other_rules): (Vec<_>, Vec<external::core::WindowRule>) = rules
        .iter()
        .cloned()
        .partition(|r| matches!(r.behavior, external::core::WindowBehavior::Ignore));

    if !ignore_filters.is_empty() {
        let rules = ignore_filters
            .iter()
            .map(|r| external::core::WindowRule::new(r.clone(), external::core::WindowBehavior::Ignore))
            .collect::<Vec<external::core::WindowRule>>();
        ignore_rules.extend(rules);
    }

    let mondrian_filter = WinMatcher::Exename("mondrian.exe".to_owned());
    let ignore_filter = match ignore_rules.is_empty() {
        true => mondrian_filter.clone(),
        false => {
            let mut filters: HashSet<WinMatcher> = ignore_rules.iter().map(|r| r.filter.clone()).collect();
            filters.insert(mondrian_filter);
            WinMatcher::Any(filters)
        }
    };

    let other_rules = other_rules
        .iter()
        .map(|r| WindowRule {
            filter: r.filter.clone(),
            behavior: match &r.behavior {
                external::core::WindowBehavior::Ignore => unreachable!(),
                external::core::WindowBehavior::Float => WindowBehavior::Float {
                    topmost: floating_wins.topmost,
                    strategy: floating_wins.strategy.clone(),
                },
                external::core::WindowBehavior::Insert { monitor } => WindowBehavior::Insert {
                    monitor: monitor.clone(),
                },
            },
        })
        .collect();

    (ignore_filter, other_rules)
}

fn get_layout_strategy(strategy_str: &str, strategies: &external::layout::StrategyConfigs) -> LayoutStrategyEnum {
    match strategy_str {
        "horizontal" => strategies.horizontal.into(),
        "vertical" => strategies.vertical.into(),
        "twostep" => strategies.twostep.into(),
        "squared" => strategies.squared.clone().into(),
        _ => strategies.golden_ratio.into(),
    }
}

fn get_monitors_config(
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

impl From<AppConfigExternal> for AppConfig {
    fn from(v: AppConfigExternal) -> Self {
        let floating_wins_config = v.general.floating_wins.into();
        let (ignore_filter, other_rules) = extract_rules(&v.core.ignore_rules, &v.core.rules, &floating_wins_config);
        let layout_strategy = get_layout_strategy(&v.layout.tiling_strategy, &v.layout.strategy);

        AppConfig {
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
            ignore_filter,
            rules: other_rules,
            modules: v.modules,
            monitors_config: get_monitors_config(&v.monitors, &v.layout),
            focalized_pads: v.layout.focalized_padding,
            layout_strategy,
            tiles_pad: v.layout.paddings.tiles,
            borders_pads: v.layout.paddings.borders,
            half_focalized_tiles_pad: v.layout.half_focalized_paddings.tiles,
            half_focalized_borders_pads: v.layout.half_focalized_paddings.borders,
        }
    }
}
