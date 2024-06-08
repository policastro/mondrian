use crate::{app::structs::area_tree::layout::layout_strategy::AreaTreeLayoutStrategyEnum, win32::api::monitor::Monitor};

#[derive(Debug)]
pub struct MonitorLayout {
    pub monitor: Monitor,
    pub layout: AreaTreeLayoutStrategyEnum,
}

impl MonitorLayout {
    pub fn new(monitor: Monitor, layout: AreaTreeLayoutStrategyEnum) -> Self {
        MonitorLayout { monitor, layout }
    }
}
