use super::keys::ContainerKey;
use super::keys::CrossLayerContainerKey;
use crate::app::area_tree::tree::WinTree;
use crate::app::structs::area::Area;
use crate::modules::tiles_manager::lib::tm::error::TilesManagerError;
use std::collections::HashMap;

type Result<T> = std::result::Result<T, TilesManagerError>;

pub trait InactiveContainers {
    fn get_normal(&mut self, base_key: &ContainerKey) -> Result<&WinTree>;
    fn get_normal_mut(&mut self, base_key: &ContainerKey) -> Result<&mut WinTree>;
    fn set_layers_area(&mut self, key: &CrossLayerContainerKey, new_area: Area);
    fn has_vd(&self, vd: u128) -> bool;
}

impl InactiveContainers for HashMap<ContainerKey, (WinTree, u128)> {
    fn get_normal(&mut self, base_key: &ContainerKey) -> Result<&WinTree> {
        self.get(&base_key.to_normal())
            .ok_or(TilesManagerError::ContainerNotFound { refresh: false })
            .map(|(t, _)| t)
    }

    fn get_normal_mut(&mut self, base_key: &ContainerKey) -> Result<&mut WinTree> {
        self.get_mut(&base_key.to_normal())
            .ok_or(TilesManagerError::ContainerNotFound { refresh: false })
            .map(|(t, _)| t)
    }

    fn set_layers_area(&mut self, key: &CrossLayerContainerKey, new_area: Area) {
        self.iter_mut()
            .filter(|(k, _)| k.is_vd(key.vd) && k.is_monitor(&key.monitor))
            .for_each(|(_, (t, _))| t.set_area(new_area));
    }

    fn has_vd(&self, vd: u128) -> bool {
        self.iter().any(|(k, _)| k.is_vd(vd))
    }
}
