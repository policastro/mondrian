use winvd::Desktop;

use crate::modules::tiles_manager::lib::tm::result::TilesManagerError;

#[derive(Debug, Clone, Copy)]
pub struct VirtualDesktop {
    id: u128,
    desktop: Desktop,
}

impl TryFrom<Desktop> for VirtualDesktop {
    type Error = TilesManagerError;
    fn try_from(value: Desktop) -> Result<Self, Self::Error> {
        let id = value.get_id().map_err(TilesManagerError::VDError)?.to_u128();
        Ok(VirtualDesktop { id, desktop: value })
    }
}

impl From<VirtualDesktop> for u128 {
    fn from(v: VirtualDesktop) -> Self {
        v.id
    }
}

impl From<VirtualDesktop> for Desktop {
    fn from(v: VirtualDesktop) -> Self {
        v.desktop
    }
}

impl VirtualDesktop {
    pub fn get_id(&self) -> u128 {
        self.id
    }

    pub fn is_desktop(&self, vd: &Desktop) -> bool {
        self.desktop.get_id() == vd.get_id()
    }

    pub fn get_desktop(&self) -> Desktop {
        self.desktop
    }
}
