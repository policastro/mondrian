use crate::app::config::win_matcher;
use crate::app::mondrian_message::WindowEvent;
use crate::win32::window::window_ref::WindowRef;

pub fn skip_window(event: &WindowEvent, win_match: &win_matcher::WinMatcher) -> bool {
    match event {
        WindowEvent::Closed(hwnd) => {
            let snap = WindowRef::new(*hwnd).snapshot();
            log::info!("[{:?}]: {}", event, snap);
            false
        }
        command => {
            let hwnd = command.get_hwnd();
            let info = WindowRef::new(hwnd).snapshot();

            let skip = win_match.matches(&info);
            match skip {
                true => log::trace!("[excluded][{:?}]: {}", event, info),
                false => log::info!("[{:?}]: {}", event, info),
            }

            skip
        }
    }
}
