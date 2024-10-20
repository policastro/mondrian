use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use windows::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{UnhookWinEvent, HWINEVENTHOOK},
        WindowsAndMessaging::{EVENT_MAX, EVENT_MIN},
    },
};

use crate::win32::{api::accessibility::set_global_win_event_hook, win_events_manager::WinEventHandler};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct WinEvent {
    pub h_win_event_hook: HWINEVENTHOOK,
    pub event: u32,
    pub hwnd: HWND,
    pub id_object: i32,
    pub id_child: i32,
    pub id_event_thread: u32,
    pub dwms_event_time: u32,
}

pub struct WinEventDispatcher {
    pub events_map: HashMap<u32, HashSet<u32>>,
    pub hooks: HashMap<u32, HWINEVENTHOOK>,
    pub counter: u32,
    pub handlers: HashMap<u32, Box<dyn WinEventHandler + Send>>,
}

impl WinEventDispatcher {
    pub fn new() -> WinEventDispatcher {
        WinEventDispatcher {
            events_map: HashMap::new(),
            hooks: HashMap::new(),
            counter: 0,
            handlers: HashMap::new(),
        }
    }

    pub fn add(&mut self, mut handler: impl WinEventHandler + Send + 'static) -> u32 {
        let events = handler.get_managed_events();
        let key = self.counter;
        if let Some(events) = events {
            events.iter().for_each(|e| {
                self.hooks.entry(*e).or_insert(set_global_win_event_hook(*e, *e));
                self.events_map.entry(*e).or_default().insert(key);
            })
        } else {
            let hook = set_global_win_event_hook(EVENT_MIN, EVENT_MAX);
            self.hooks.entry(0).or_insert(hook);
            self.events_map.entry(0).or_default().insert(key);
        };
        handler.init();
        self.handlers.insert(key, Box::new(handler));
        self.counter += 1;
        key
    }

    pub fn remove(&mut self, id: u32) {
        let h = match self.handlers.remove(&id) {
            Some(h) => h,
            None => return,
        };

        let events = h.get_managed_events();
        if let Some(events) = events {
            events.iter().for_each(|e| self.remove_event_handler(*e, id))
        } else {
            self.remove_event_handler(0, id)
        }
    }

    fn remove_event_handler(&mut self, event: u32, id: u32) {
        let evt_handler = match self.events_map.get_mut(&event) {
            Some(e) => e,
            None => return,
        };

        evt_handler.remove(&id);
        if evt_handler.is_empty() {
            self.events_map.remove(&event);
            if let Some(hook) = self.hooks.remove(&event) {
                unsafe {
                    let _ = UnhookWinEvent(hook);
                }
            }
        }
    }

    fn dispatch(&mut self, event: &WinEvent) {
        if let Some(handlers) = self.events_map.get(&event.event) {
            handlers.iter().for_each(|i| {
                self.handlers.get_mut(i).expect("Handler not found").handle(event);
            })
        }

        if let Some(handlers) = self.events_map.get(&0) {
            handlers.iter().for_each(|i| {
                self.handlers.get_mut(i).expect("Handler not found").handle(event);
            })
        }
    }
}

thread_local! {
    pub(crate) static EVENT_MANAGER: Arc<Mutex<WinEventDispatcher>> = Arc::new(Mutex::new(WinEventDispatcher::new()));
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
    let event = WinEvent {
        h_win_event_hook: hook_id,
        event,
        hwnd,
        id_object,
        id_child,
        id_event_thread,
        dwms_event_time,
    };
    EVENT_MANAGER.with(|event_manager| event_manager.lock().unwrap().dispatch(&event));
}
