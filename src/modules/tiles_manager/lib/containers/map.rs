use super::keys::ContainerKeyTrait;
use super::Container;
use std::collections::HashMap;

pub trait ContainersMap<K> {
    fn has_vd(&self, vd: u128) -> bool;
}

impl<K: ContainerKeyTrait> ContainersMap<K> for HashMap<K, Container> {
    fn has_vd(&self, vd: u128) -> bool {
        self.iter().any(|(k, _)| k.is_vd(vd))
    }
}
