use crate::{app::structs::area_tree::layout_strategy::LayoutStrategyEnum, win32::api::monitor::Monitor};

#[derive(Debug)]
pub struct MonitorLayout {
    pub monitor: Monitor,
    pub layout: LayoutStrategyEnum,
}

impl MonitorLayout {
    pub fn new(monitor: Monitor, layout: LayoutStrategyEnum) -> Self {
        MonitorLayout { monitor, layout }
    }
}
