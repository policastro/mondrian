use crate::app::mondrian_message::WindowEvent;
use crate::app::structs::win_matcher;

pub fn skip_window(event: &WindowEvent, win_match: &win_matcher::WinMatcher) -> bool {
    match event {
        WindowEvent::Closed(_) => false,
        command => {
            let info = command.get_window_ref().snapshot();
            if win_match.matches(info.clone()) {
                log::trace!("[excluded][{:?}]: {}", event, info);
                return true;
            }

            false
        }
    }
}
