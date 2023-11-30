use std::sync::{mpsc::{channel, Receiver, Sender}, Arc, Mutex};

use lazy_static::lazy_static;
use windows::Win32::{Foundation::HWND, UI::Accessibility::HWINEVENTHOOK};

use crate::win32utils::win_events_manager::WinEvent;

lazy_static! {
    pub(crate) static ref WINEVENT_CHANNEL: Arc<Mutex<(Sender<WinEvent>, Receiver<WinEvent>)>> = Arc::new(Mutex::new(channel()));
}

pub(crate) unsafe extern "system" fn win_event_hook(
    hook_id: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    id_object: i32,
    id_child: i32,
    id_event_thread: u32,
    dwms_event_time: u32,
) {
    WINEVENT_CHANNEL
        .lock()
        .unwrap()
        .0
        .send(WinEvent {
            h_win_event_hook: hook_id,
            event,
            hwnd,
            id_object,
            id_child,
            id_event_thread,
            dwms_event_time,
        })
        .unwrap();
}