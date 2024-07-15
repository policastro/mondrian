use crate::{
    app::mondrian_command::MondrianCommand,
    modules::module::{ConfigurableModule, Module},
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
};

use super::configs::KeybindingsConfig;
pub struct KeybindingsModule {
    bus: Sender<MondrianCommand>,
    input_thread: Option<thread::JoinHandle<()>>,
    enabled: bool,
    running: Arc<AtomicBool>,
    configs: KeybindingsConfig,
    binded_keys: Vec<inputbot::KeybdKey>,
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

impl ConfigurableModule<KeybindingsConfig> for KeybindingsModule {
    fn configure(&mut self, config: KeybindingsConfig) {
        self.configs = config;
    }
}

impl KeybindingsModule {
    pub fn new(bus: Sender<MondrianCommand>) -> KeybindingsModule {
        KeybindingsModule {
            input_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
            configs: KeybindingsConfig::default(),
            binded_keys: vec![],
            bus,
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

        let bus = self.bus.clone();

        self.configs.bindings.iter().for_each(move |b| {
            let modifiers = b.0.clone();
            let command = b.2.clone();
            let bus = bus.clone();
            b.1.bind(move || {
                if modifiers.iter().all(|m| m.is_pressed()) {
                    bus.send(command).unwrap();
                }
            });
        });

        self.binded_keys = self.configs.bindings.iter().map(|b| b.1).collect();
        let input_thread = thread::spawn(|| inputbot::handle_input_events());
        self.input_thread = Some(input_thread);
    }

    fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(false, Ordering::SeqCst);

        self.binded_keys.iter().for_each(|k| k.unbind());
        self.binded_keys.clear();
        if self.input_thread.is_some() {
            self.input_thread.take().unwrap().join().unwrap();
        }
    }

    fn restart(&mut self) {
        self.stop();
        self.start();
    }
}
