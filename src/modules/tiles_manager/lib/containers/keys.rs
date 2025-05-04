use std::hash::Hash;
use std::hash::Hasher;

pub trait ContainerKeyTrait {
    fn is_vd(&self, vd: u128) -> bool;
    fn is_monitor(&self, monitor: &str) -> bool;
    fn is_workspace(&self, workspace: &str) -> bool;
}

#[derive(Hash, PartialEq, Debug, Eq, Clone)]
pub struct ContainerKey {
    pub vd: u128,
    pub monitor: String,
    pub workspace: String,
}

#[derive(Eq, Clone, Debug)]
pub struct ActiveContainerKey {
    pub vd: u128,
    pub monitor: String,
    pub workspace: String,
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
            workspace: value.workspace,
        }
    }
}

impl ContainerKey {
    pub fn new(vd: u128, monitor: &str, workspace: &str) -> Self {
        ContainerKey {
            vd,
            monitor: monitor.to_string(),
            workspace: workspace.to_string(),
        }
    }
}

impl From<ActiveContainerKey> for ContainerKey {
    fn from(value: ActiveContainerKey) -> Self {
        ContainerKey::new(value.vd, &value.monitor, &value.workspace)
    }
}

impl ContainerKeyTrait for ContainerKey {
    fn is_vd(&self, vd: u128) -> bool {
        self.vd == vd
    }

    fn is_monitor(&self, monitor: &str) -> bool {
        self.monitor == monitor
    }

    fn is_workspace(&self, workspace: &str) -> bool {
        self.workspace == workspace
    }
}

impl ContainerKeyTrait for ActiveContainerKey {
    fn is_vd(&self, vd: u128) -> bool {
        self.vd == vd
    }

    fn is_monitor(&self, monitor: &str) -> bool {
        self.monitor == monitor
    }

    fn is_workspace(&self, workspace: &str) -> bool {
        self.workspace == workspace
    }
}
