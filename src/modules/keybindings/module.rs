use inputbot::BlockInput;
use windows::Win32::{
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{PostThreadMessageW, WM_QUIT},
};

use crate::{
    app::{config::app_configs::AppConfigs, mondrian_command::MondrianCommand},
    modules::module::{module_impl::ModuleImpl, ConfigurableModule, Module},
};
use std::{
    sync::{
        atomic::{AtomicBool, AtomicU32, Ordering},
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
    main_thread_id: Arc<AtomicU32>,
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
        self.configs.bindings.iter().for_each(move |b| {
            let modifiers = b.0.clone();
            let command = b.2;
            let bus = bus.clone();
            b.1.blockable_bind(move || {
                if modifiers.iter().all(|m| m.is_pressed()) {
                    bus.send(command).expect("Failed to send command");
                    return BlockInput::Block;
                }
                BlockInput::DontBlock
            });
        });

        self.binded_keys = self.configs.bindings.iter().map(|b| b.1).collect();
        let input_thread = thread::spawn(move || {
            main_thread_id.store(unsafe { GetCurrentThreadId() }, Ordering::SeqCst);
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
        let _ = unsafe { PostThreadMessageW(self.main_thread_id.load(Ordering::SeqCst), WM_QUIT, None, None) };
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

    fn handle(&mut self, event: &MondrianCommand, app_configs: &AppConfigs) {
        match event {
            MondrianCommand::Pause(pause) => Module::pause(self, *pause),
            MondrianCommand::Configure => {
                Module::enable(self, app_configs.keybinds_enabled);
                self.configure(app_configs.into());
            }
            MondrianCommand::RefreshConfig => {
                Module::enable(self, app_configs.keybinds_enabled);
                self.configure(app_configs.into());
                Module::restart(self);
            }
            MondrianCommand::Quit => Module::stop(self),
            _ => {}
        }
    }
}

impl ConfigurableModule for KeybindingsModule {
    type Config = KeybindingsConfig;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}
