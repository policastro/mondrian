use crate::app::area_tree::layout_strategy::LayoutStrategyEnum;
use crate::win32::api::monitor::Monitor;

#[derive(Debug, Clone)]
pub struct MonitorLayout {
    pub monitor: Monitor,
    pub layout: LayoutStrategyEnum,
}

impl MonitorLayout {
    pub fn new(monitor: Monitor, layout: LayoutStrategyEnum) -> Self {
        MonitorLayout { monitor, layout }
    }
}
