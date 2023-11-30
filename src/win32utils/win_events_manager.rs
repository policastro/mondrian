use log::trace;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::{
    Accessibility::{SetWinEventHook, UnhookWinEvent, HWINEVENTHOOK},
    WindowsAndMessaging::{EVENT_MAX, EVENT_MIN, WINEVENT_OUTOFCONTEXT},
};

use super::api::callbacks::win_event_hook::{win_event_hook, WINEVENT_CHANNEL};

#[derive(Debug)]
pub struct WinEvent {
    pub h_win_event_hook: HWINEVENTHOOK,
    pub event: u32,
    pub hwnd: HWND,
    pub id_object: i32,
    pub id_child: i32,
    pub id_event_thread: u32,
    pub dwms_event_time: u32,
}

pub trait WinEventHandler {
    fn handle(&mut self, event: &WinEvent);
}

pub struct WinEventManager {
    pub hook_id: HWINEVENTHOOK,
    pub handlers: Vec<Box<dyn WinEventHandler + Send>>,
}

impl WinEventManager {
    pub fn new() -> WinEventManager {
        let hook_id = unsafe {
            SetWinEventHook(
                EVENT_MIN,
                EVENT_MAX,
                None,
                Some(win_event_hook),
                0,
                0,
                WINEVENT_OUTOFCONTEXT,
            )
        };

        WinEventManager {
            hook_id,
            handlers: vec![],
        }
    }

    pub fn check_for_events(&mut self) {
        let event = WINEVENT_CHANNEL.lock().unwrap().1.try_recv();
        if let Ok(event) = event {
            trace!("EVENT: {:?}", event);
            self.execute_hooks(&event);
        }
    }

    // TODO 'static doesn't seem right
    pub fn add_handler(&mut self, handler: impl WinEventHandler + Send + 'static) {
        self.handlers.push(Box::new(handler));
    }

    fn execute_hooks(&mut self, event: &WinEvent) {
        for h in self.handlers.iter_mut() {
            h.as_mut().handle(event);
        }
    }
}

impl Drop for WinEventManager {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWinEvent(self.hook_id);
        }
    }
}
