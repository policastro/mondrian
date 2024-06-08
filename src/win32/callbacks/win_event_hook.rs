use std::sync::{
    mpsc::{channel, Receiver, Sender},
    Arc, Mutex,
};

use windows::Win32::{Foundation::HWND, UI::Accessibility::HWINEVENTHOOK};

use crate::win32::win_events_manager::WinEvent;

thread_local! {
    pub(crate) static WINEVENT_CHANNEL: Arc<Mutex<(Sender<WinEvent>, Receiver<WinEvent>)>> = {
        let (sender, receiver) = channel();
        Arc::new(Mutex::new((sender, receiver)))
    };
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
    WINEVENT_CHANNEL.with(|channel| {
        channel
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
    })
}
