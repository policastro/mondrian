use super::{external, floating::FloatingWinsConfig};
use crate::app::structs::win_matcher::WinMatcher;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq)]
pub struct WindowRule {
    pub filter: WinMatcher,
    pub behavior: WindowBehavior,
}

#[derive(Clone, Debug, PartialEq)]
pub enum WindowBehavior {
    Float {
        config: FloatingWinsConfig,
    },
    Insert {
        monitor: Option<String>,
        workspace: Option<String>,
        silent: bool,
    },
}

pub(crate) fn extract_rules(
    ignore_filters: &[WinMatcher],
    rules: &[external::core::WindowRule],
    floating_wins_ext: &external::general::FloatingWinsConfig,
) -> (WinMatcher, Vec<(WinMatcher, u32)>, Vec<WindowRule>) {
    let (mut ignore_rules, other_rules): (Vec<_>, Vec<external::core::WindowRule>) = rules
        .iter()
        .cloned()
        .partition(|r| matches!(r.behavior, external::core::WindowBehavior::Ignore));

    let (delayed_filter, other_rules): (Vec<_>, Vec<external::core::WindowRule>) = other_rules
        .iter()
        .cloned()
        .partition(|r| matches!(r.behavior, external::core::WindowBehavior::DelayInsert { .. }));

    let delayed_filter = delayed_filter
        .iter()
        .map(|r| match &r.behavior {
            external::core::WindowBehavior::DelayInsert { delay: wait } => (r.filter.clone(), *wait),
            _ => unreachable!(),
        })
        .collect();

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
                external::core::WindowBehavior::Ignore | external::core::WindowBehavior::DelayInsert { .. } => {
                    unreachable!()
                }
                external::core::WindowBehavior::Float {
                    topmost,
                    size,
                    centered,
                    size_fixed,
                    size_ratio,
                } => {
                    let config_ext = external::general::FloatingWinsConfig {
                        size: size.unwrap_or(floating_wins_ext.size),
                        centered: centered.unwrap_or(floating_wins_ext.centered),
                        topmost: topmost.unwrap_or(floating_wins_ext.topmost),
                        size_ratio: size_ratio.unwrap_or(floating_wins_ext.size_ratio),
                        size_fixed: size_fixed.unwrap_or(floating_wins_ext.size_fixed),
                    };
                    WindowBehavior::Float {
                        config: config_ext.into(),
                    }
                }
                external::core::WindowBehavior::Insert {
                    monitor,
                    workspace,
                    silent,
                } => WindowBehavior::Insert {
                    monitor: monitor.clone(),
                    workspace: workspace.clone(),
                    silent: *silent,
                },
            },
        })
        .collect();

    (ignore_filter, delayed_filter, other_rules)
}
