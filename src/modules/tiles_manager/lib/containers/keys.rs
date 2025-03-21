use super::layer::ContainerLayer;
use std::hash::Hash;
use std::hash::Hasher;

#[derive(Hash, PartialEq, Eq, Clone)]
pub struct ContainerKey {
    pub vd: u128,
    pub monitor: String,
    pub layer: ContainerLayer, // TODO: support for multiple layers
}

#[derive(Eq, Clone)]
pub struct CrossLayerContainerKey {
    pub vd: u128,
    pub monitor: String,
    pub layer: ContainerLayer, // TODO: support for multiple layers
}

impl ContainerKey {
    pub fn new(vd: u128, monitor: String, layer: ContainerLayer) -> Self {
        ContainerKey { vd, monitor, layer }
    }

    pub fn is_vd(&self, vd: u128) -> bool {
        self.vd == vd
    }

    pub fn is_monitor(&self, monitor: &str) -> bool {
        self.monitor == monitor
    }

    pub fn is_layer(&self, layer: ContainerLayer) -> bool {
        self.layer == layer
    }

    pub fn normal(vd: u128, monitor: String) -> Self {
        ContainerKey {
            vd,
            monitor,
            layer: ContainerLayer::Normal,
        }
    }

    pub fn focalized(vd: u128, monitor: String) -> Self {
        ContainerKey {
            vd,
            monitor,
            layer: ContainerLayer::Focalized,
        }
    }

    pub fn to_normal(&self) -> Self {
        ContainerKey {
            vd: self.vd,
            monitor: self.monitor.clone(),
            layer: ContainerLayer::Normal,
        }
    }
}

impl PartialEq for CrossLayerContainerKey {
    fn eq(&self, other: &Self) -> bool {
        self.vd == other.vd && self.monitor == other.monitor
    }
}

impl Hash for CrossLayerContainerKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.vd.hash(state);
        self.monitor.hash(state);
    }
}

impl From<ContainerKey> for CrossLayerContainerKey {
    fn from(value: ContainerKey) -> Self {
        CrossLayerContainerKey {
            vd: value.vd,
            monitor: value.monitor,
            layer: value.layer,
        }
    }
}

impl From<CrossLayerContainerKey> for ContainerKey {
    fn from(value: CrossLayerContainerKey) -> Self {
        ContainerKey::new(value.vd, value.monitor, value.layer)
    }
}
