use windows::Win32::Foundation::HWND;

use crate::{
    app::structs::{area::Area, area_tree::leaf::AreaLeaf},
    win32::window::window_snapshot::WindowSnapshot,
};

#[derive(Debug, Clone, Copy)]
pub(super) struct ManagedWindow {
    pub id: HWND,
    pub viewarea: Area,
    pub leaf: Option<AreaLeaf<isize>>,
}

impl From<WindowSnapshot> for ManagedWindow {
    fn from(snapshot: WindowSnapshot) -> Self {
        ManagedWindow::new(snapshot.hwnd, snapshot.viewarea.expect("Area not found"), None)
    }
}

impl ManagedWindow {
    pub fn new(id: HWND, viewarea: Area, leaf: Option<AreaLeaf<isize>>) -> Self {
        ManagedWindow { id, viewarea, leaf }
    }
}
