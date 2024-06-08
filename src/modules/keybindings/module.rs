use crate::modules::module::Module;
use inputbot::KeybdKey::*;
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};
pub struct KeybindingsModule {
    input_thread: Option<thread::JoinHandle<()>>,
    enabled: bool,
    running: Arc<AtomicBool>,
}

impl Module for KeybindingsModule {
    fn start(&mut self) {
        if self.enabled {
            self.start();
        }
    }

    fn stop(&mut self) {
        if self.enabled {
            self.stop();
        }
    }

    fn restart(&mut self) {
        if self.enabled {
            self.restart();
        }
    }

    fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.stop();
        }
    }
}

impl KeybindingsModule {
    pub fn new() -> KeybindingsModule {
        KeybindingsModule {
            input_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
        }
    }

    pub fn pause(&mut self, is_paused: bool) {
        if is_paused {
            self.stop()
        } else {
            self.start()
        }
    }

    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(true, Ordering::SeqCst);

        // define an inline function

        LeftKey.bind(|| {
            let modifier = LAltKey.is_pressed() && LControlKey.is_pressed() && LSuper.is_pressed();
            if modifier {
                println!("LeftKey");
            }
        });

        RightKey.bind(|| {
            let modifier = LAltKey.is_pressed() && LControlKey.is_pressed() && LSuper.is_pressed();
            if modifier {
                println!("RightKey");
            }
        });

        UpKey.bind(|| {
            let modifier = LAltKey.is_pressed() && LControlKey.is_pressed() && LSuper.is_pressed();
            if modifier {
                println!("UpKey");
            }
        });

        DownKey.bind(|| {
            let modifier = LAltKey.is_pressed() && LControlKey.is_pressed() && LSuper.is_pressed();
            if modifier {
                println!("DownKey");
            }
        });

        let input_thread = thread::spawn(move || {
            inputbot::handle_input_events();
        });

        self.input_thread = Some(input_thread);
    }

    fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(false, Ordering::SeqCst);
        // TODO find a way to kill the thread (but honestly, I think that I should change the dependency)
    }

    fn restart(&mut self) {
        self.stop();
        self.start();
    }
}
