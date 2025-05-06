use crate::{app::structs::area::Area, win32::window::window_ref::WindowRef};

#[derive(Debug, Clone, PartialEq)]
pub enum TilesManagerError {
    Generic,
    VDError(winvd::Error),
    WindowAlreadyAdded(WindowRef),
    NoWindowsInfo,
    ContainerNotFound { refresh: bool },
    WinNotManaged(WindowRef),
    NoWindow,
    MonitorNotFound(String),
    NoMonitorFound,
    NoMonitorAtPoint((i32, i32)),
    NoContainerAtPoint((i32, i32)),
    WorkspaceAlreadyCreated,
    VDContainersAlreadyCreated,
    VDContainersAlreadyActivated,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TilesManagerSuccess {
    LayoutChanged,
    UpdateAndFocus {
        window: Option<WindowRef>,
    },
    Queue {
        window: WindowRef,
        area: Area,
        topmost: Option<bool>,
    },
    Dequeue {
        window: WindowRef,
    },
    NoChange,
}

impl TilesManagerSuccess {
    pub fn queue(window: WindowRef, area: Area, topmost: Option<bool>) -> Self {
        Self::Queue { window, area, topmost }
    }

    pub fn dequeue(window: WindowRef) -> Self {
        Self::Dequeue { window }
    }
}

impl<T> From<TilesManagerError> for Result<T, TilesManagerError> {
    fn from(value: TilesManagerError) -> Self {
        Err(value)
    }
}

impl TilesManagerError {
    pub fn get_log_level(&self) -> log::Level {
        match self {
            Self::NoContainerAtPoint(_) => log::Level::Error,
            Self::NoWindowsInfo => log::Level::Warn,
            _ => log::Level::Debug,
        }
    }

    pub fn get_info(&self) -> String {
        format!("{:?}", self)
    }

    pub fn require_refresh(&self) -> bool {
        matches!(self, Self::ContainerNotFound { refresh: true })
    }

    pub fn container_not_found() -> Self {
        Self::ContainerNotFound { refresh: false }
    }

    pub fn container_not_found_refresh() -> Self {
        Self::ContainerNotFound { refresh: true }
    }
}
