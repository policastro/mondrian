use inputbot::BlockInput;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

use crate::{
    app::{config::app_configs::AppConfigs, mondrian_command::MondrianMessage},
    modules::module::{module_impl::ModuleImpl, ConfigurableModule, Module},
    win32::api::misc::{get_current_thread_id, post_empty_thread_message},
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
};

use super::configs::KeybindingsModuleConfigs;
pub struct KeybindingsModule {
    bus: Sender<MondrianMessage>,
    input_thread: Option<thread::JoinHandle<()>>,
    enabled: bool,
    running: Arc<AtomicBool>,
    configs: KeybindingsModuleConfigs,
    binded_keys: Vec<inputbot::KeybdKey>,
    main_thread_id: Arc<AtomicU32>,
}

impl KeybindingsModule {
    pub fn new(bus: Sender<MondrianMessage>) -> KeybindingsModule {
        KeybindingsModule {
            input_thread: None,
            running: Arc::new(AtomicBool::new(false)),
            enabled: true,
            configs: KeybindingsModuleConfigs::default(),
            binded_keys: vec![],
            bus,
            main_thread_id: Arc::new(AtomicU32::new(0)),
        }
    }
}

impl ModuleImpl for KeybindingsModule {
    fn start(&mut self) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(true, Ordering::SeqCst);

        let bus = self.bus.clone();
        let main_thread_id = self.main_thread_id.clone();
        let bindings = self.configs.get_grouped_bindings();
        for (key, list_mod_command) in bindings {
            let bus = bus.clone();
            key.blockable_bind(move || {
                for (m, c) in list_mod_command.iter() {
                    if m.iter().all(|m| m.is_pressed()) {
                        bus.send(c.clone()).expect("Failed to send command");
                        return BlockInput::Block;
                    }
                }
                BlockInput::DontBlock
            });
        }
        self.binded_keys = self.configs.bindings.iter().map(|b| b.1).collect();
        let input_thread = thread::spawn(move || {
            main_thread_id.store(get_current_thread_id(), Ordering::SeqCst);
            inputbot::handle_input_events(false);
        });
        self.input_thread = Some(input_thread);
    }

    fn stop(&mut self) {
        if !self.running.load(Ordering::SeqCst) {
            return;
        }
        self.running.store(false, Ordering::SeqCst);

        self.binded_keys.iter().for_each(|k| k.unbind());
        self.binded_keys.clear();
        // Not sure why this is required, but without it the module crashes
        post_empty_thread_message(self.main_thread_id.load(Ordering::SeqCst), WM_QUIT);
        inputbot::stop_handling_input_events();
        if self.input_thread.is_some() {
            self.input_thread.take().unwrap().join().unwrap();
        }
    }

    fn restart(&mut self) {
        Module::stop(self);
        Module::start(self);
    }

    fn pause(&mut self, is_paused: bool) {
        match is_paused {
            true => Module::stop(self),
            false => Module::start(self),
        }
    }

    fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfigs) {
        match event {
            MondrianMessage::Pause(pause) => Module::pause(self, *pause),
            MondrianMessage::Configure => {
                Module::enable(self, app_configs.modules.keybindings.enabled);
                self.configure(app_configs.into());
            }
            MondrianMessage::RefreshConfig => {
                Module::enable(self, app_configs.modules.keybindings.enabled);
                self.configure(app_configs.into());
                Module::restart(self);
            }
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }
}

impl ConfigurableModule for KeybindingsModule {
    type Config = KeybindingsModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}