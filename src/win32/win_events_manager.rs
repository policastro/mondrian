use super::{
    callbacks::win_event_hook::{WindowsEvent, EVENT_MANAGER},
    win_event_loop::start_win_event_loop,
};

pub trait WinEventHandler {
    fn init(&mut self);
    fn handle(&mut self, event: &WindowsEvent);
    /// Returns the list of events managed by this handler. If None, all events are managed.
    fn get_managed_events(&self) -> Option<Vec<u32>>;
}

pub struct WindowsEventManager {
    handlers_key: Vec<u32>,
}

impl WindowsEventManager {
    pub fn new() -> WindowsEventManager {
        WindowsEventManager { handlers_key: vec![] }
    }

    pub fn hook(&mut self, handler: impl WinEventHandler + Send + 'static) {
        let handler_key = EVENT_MANAGER.lock().unwrap().add(handler);
        self.handlers_key.push(handler_key);
    }

    pub fn unhook_all(&mut self) {
        self.handlers_key
            .iter()
            .for_each(|i| EVENT_MANAGER.lock().unwrap().remove(*i));
        self.handlers_key.clear();
    }

    pub fn start_event_loop(&self) {
        start_win_event_loop();
    }
}

impl Drop for WindowsEventManager {
    fn drop(&mut self) {
        self.unhook_all();
    }
}
