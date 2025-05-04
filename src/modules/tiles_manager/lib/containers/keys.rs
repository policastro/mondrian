use std::hash::Hash;
use std::hash::Hasher;

pub trait ContainerKeyTrait {
    fn is_vd(&self, vd: u128) -> bool;
}

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct ContainerKey {
    pub vd: u128,
    pub monitor: String,
}

#[derive(Eq, Clone, Debug)]
pub struct ActiveContainerKey {
    pub vd: u128,
    pub monitor: String,
}

impl ContainerKey {
    pub fn new(vd: u128, monitor: String) -> Self {
        ContainerKey { vd, monitor }
    }

    pub fn is_vd(&self, vd: u128) -> bool {
        self.vd == vd
    }
}

impl PartialEq for ActiveContainerKey {
    fn eq(&self, other: &Self) -> bool {
        self.monitor == other.monitor
    }
}

impl Hash for ActiveContainerKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.monitor.hash(state);
    }
}

impl From<ContainerKey> for ActiveContainerKey {
    fn from(value: ContainerKey) -> Self {
        ActiveContainerKey {
            vd: value.vd,
            monitor: value.monitor,
        }
    }
}

impl From<ActiveContainerKey> for ContainerKey {
    fn from(value: ActiveContainerKey) -> Self {
        ContainerKey::new(value.vd, value.monitor)
    }
}

impl ContainerKeyTrait for ContainerKey {
    fn is_vd(&self, vd: u128) -> bool {
        self.vd == vd
    }
}

impl ContainerKeyTrait for ActiveContainerKey {
    fn is_vd(&self, vd: u128) -> bool {
        self.vd == vd
    }
}
