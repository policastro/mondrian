use crate::win32::window::window_ref::WindowRef;

#[derive(Debug, Clone, PartialEq)]
pub enum TilesManagerError {
    Generic,
    VirtualDesktopError(winvd::Error),
    WindowAlreadyAdded,
    NoWindowsInfo,
    ContainerNotFound { refresh: bool },
    WinNotManaged(WindowRef),
    NoWindow,
}

impl TilesManagerError {
    pub fn get_log_level(&self) -> log::Level {
        match self {
            TilesManagerError::Generic
            | TilesManagerError::WindowAlreadyAdded
            | TilesManagerError::NoWindow
            | TilesManagerError::WinNotManaged(_) => log::Level::Debug,
            TilesManagerError::NoWindowsInfo => log::Level::Warn,
            _ => log::Level::Error,
        }
    }

    pub fn get_info(&self) -> String {
        format!("{:?}", self)
    }

    pub fn require_refresh(&self) -> bool {
        matches!(self, TilesManagerError::ContainerNotFound { refresh: true })
    }
}
