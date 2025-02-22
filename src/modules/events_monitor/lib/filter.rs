use crate::app::mondrian_message::WindowEvent;
use crate::app::structs::win_matcher;
use crate::win32::window::window_ref::WindowRef;

pub fn skip_window(event: &WindowEvent, win_match: &win_matcher::WinMatcher) -> bool {
    match event {
        WindowEvent::Closed(_) => false,
        command => {
            let info = WindowRef::new(command.get_hwnd()).snapshot();
            if win_match.matches(info.clone()) {
                log::trace!("[excluded][{:?}]: {}", event, info);
                return true;
            }

            false
        }
    }
}
