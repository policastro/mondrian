use windows::Win32::UI::WindowsAndMessaging::{
    EVENT_OBJECT_CLOAKED, EVENT_OBJECT_DESTROY, EVENT_OBJECT_HIDE, EVENT_OBJECT_UNCLOAKED, EVENT_SYSTEM_FOREGROUND,
    EVENT_SYSTEM_MINIMIZEEND, EVENT_SYSTEM_MINIMIZESTART, EVENT_SYSTEM_MOVESIZEEND, EVENT_SYSTEM_MOVESIZESTART,
};

use crate::win32::win_events_manager::{WinEvent, WinEventHandler};

use super::window_overlay::WindowOverlay;

pub struct FocusEventHandler {
    overlay: WindowOverlay,
    waiting: bool,
}

impl FocusEventHandler {
    pub fn new(overlay: WindowOverlay) -> FocusEventHandler {
        FocusEventHandler {
            overlay,
            waiting: false,
        }
    }
}

impl WinEventHandler for FocusEventHandler {
    fn init(&mut self) {
        self.overlay.move_to_foreground();
    }

    fn handle(&mut self, event: &WinEvent) {
        if event.event == EVENT_SYSTEM_MOVESIZESTART {
            self.waiting = true;
            self.overlay.hide();
        }

        if event.event == EVENT_SYSTEM_MOVESIZEEND {
            self.waiting = false;
        }

        if !self.waiting {
            self.overlay.move_to_foreground();
        }
    }

    fn get_managed_events(&self) -> Option<Vec<u32>> {
        return Some(vec![
            EVENT_OBJECT_UNCLOAKED,
            EVENT_OBJECT_CLOAKED,
            EVENT_OBJECT_DESTROY,
            EVENT_OBJECT_HIDE,
            EVENT_SYSTEM_MOVESIZEEND,
            EVENT_SYSTEM_FOREGROUND,
            EVENT_SYSTEM_MINIMIZESTART,
            EVENT_SYSTEM_MINIMIZEEND,
            EVENT_SYSTEM_MOVESIZESTART,
        ]);
    }
}
