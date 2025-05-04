use super::container::ContainerLayer;
use super::keys::ActiveContainerKey;
use super::keys::ContainerKeyTrait;
use super::Container;
use crate::modules::tiles_manager::lib::tm::result::TilesManagerError;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;

pub trait ContainersMap<K> {
    fn has_vd(&self, vd: u128) -> bool;
    fn has(&self, vd: u128, monitor: &str, workspace: &str) -> bool;
    fn get_key_with_workspace(&mut self, vd: u128, monitor: &str, workspace: &str) -> Option<K>;
    fn get_key_with_window(&mut self, vd: u128, window: &WindowRef) -> Option<K>;
}

impl<K: ContainerKeyTrait + Clone> ContainersMap<K> for HashMap<K, Container> {
    fn has_vd(&self, vd: u128) -> bool {
        self.iter().any(|(k, _)| k.is_vd(vd))
    }

    fn has(&self, vd: u128, monitor: &str, workspace: &str) -> bool {
        self.iter()
            .any(|(k, _)| k.is_vd(vd) && k.is_monitor(monitor) && k.is_workspace(workspace))
    }

    fn get_key_with_workspace(&mut self, vd: u128, monitor: &str, workspace: &str) -> Option<K> {
        self.iter()
            .find(|(k, _)| k.is_vd(vd) && k.is_monitor(monitor) && k.is_workspace(workspace))
            .map(|(k, _)| k)
            .cloned()
    }

    fn get_key_with_window(&mut self, vd: u128, window: &WindowRef) -> Option<K> {
        self.iter()
            .find(|(k, c)| k.is_vd(vd) && c.get_tree(ContainerLayer::Normal).has(*window))
            .map(|(k, _)| k)
            .cloned()
    }
}

pub trait ActiveContainersMap {
    fn replace(&mut self, key: ActiveContainerKey, container: Container) -> Option<Container>;
    fn get_key_by_monitor(&self, monitor: &str) -> Result<ActiveContainerKey, TilesManagerError>;
}

impl ActiveContainersMap for HashMap<ActiveContainerKey, Container> {
    fn replace(&mut self, key: ActiveContainerKey, container: Container) -> Option<Container> {
        let prev = self.remove(&key);
        self.insert(key, container);
        prev
    }

    fn get_key_by_monitor(&self, monitor: &str) -> Result<ActiveContainerKey, TilesManagerError> {
        self.keys()
            .find(|k| k.is_monitor(monitor))
            .cloned()
            .ok_or(TilesManagerError::container_not_found())
    }
}
