#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum ContainerLayer {
    Normal,
    Focalized,
    HalfFocalized,
}

impl ContainerLayer {
    pub fn is_focalized(&self) -> bool {
        matches!(self, ContainerLayer::Focalized | ContainerLayer::HalfFocalized)
    }
}
