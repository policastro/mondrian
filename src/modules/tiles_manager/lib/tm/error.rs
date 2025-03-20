use crate::win32::window::window_ref::WindowRef;

#[derive(Debug, Clone, PartialEq)]
pub enum TilesManagerError {
    Generic,
    VDError(winvd::Error),
    WindowAlreadyAdded(WindowRef),
    NoWindowsInfo,
    ContainerNotFound { refresh: bool },
    WinNotManaged(WindowRef),
    NoWindow,
    NoContainerAtPoint((i32, i32)),
    VDContainersAlreadyCreated,
    VDContainersAlreadyActivated,
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
}
