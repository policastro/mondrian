#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TilesManagerError {
    Generic,
    WindowAlreadyAdded,
    NoWindowsInfo,
    ContainerNotFound { refresh: bool },
    NoWindow,
}

impl TilesManagerError {
    pub fn is_warn(&self) -> bool {
        matches!(
            self,
            TilesManagerError::WindowAlreadyAdded | TilesManagerError::NoWindowsInfo
        )
    }

    pub fn require_refresh(&self) -> bool {
        matches!(self, TilesManagerError::ContainerNotFound { refresh: true })
    }
}
