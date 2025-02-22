use crate::app::configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::api::misc::get_current_thread_id;
use crate::win32::api::misc::post_empty_thread_message;
use inputbot::BlockInput;
use inputbot::KeybdKey;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::thread;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

use inputbot::KeybdKey::*;

use super::configs::KeybindingsModuleConfigs;
pub struct KeybindingsModule {
    bus: Sender<MondrianMessage>,
    input_thread: Option<thread::JoinHandle<()>>,
    enabled: bool,
    running: Arc<AtomicBool>,
    configs: KeybindingsModuleConfigs,
    binded_keys: Vec<inputbot::KeybdKey>,
    main_thread_id: Arc<AtomicU32>,
    paused: bool,
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
            paused: false,
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
        let mut grouped_bindings = self.configs.group_by_key();

        // INFO: when paused, keeps only the bindings to unpause
        if self.paused {
            grouped_bindings.values_mut().for_each(|v| {
                v.retain(|b| {
                    matches!(
                        b.action(),
                        MondrianMessage::Pause(_) | MondrianMessage::PauseModule(_, _)
                    )
                })
            });
            grouped_bindings.retain(|_, v| !v.is_empty());
        }

        self.binded_keys = grouped_bindings.clone().into_keys().collect();
        if self.binded_keys.is_empty() {
            self.running.store(false, Ordering::SeqCst);
            return;
        }

        const ALL_MODIFIERS: &[KeybdKey] = &[
            LAltKey,
            LControlKey,
            LShiftKey,
            LSuper,
            RAltKey,
            RControlKey,
            RShiftKey,
            RSuper,
        ];
        for (key, binding) in grouped_bindings {
            let bus = bus.clone();
            key.blockable_bind(move || {
                for b in binding.iter() {
                    let (m, a) = (b.modifiers(), b.action());
                    let skip = ALL_MODIFIERS.iter().filter(|k| !m.contains(k)).any(|m| m.is_pressed()); // INFO: only the binding modifiers must be pressed

                    if !skip && b.are_modifiers_pressed() {
                        bus.send(a.clone()).inspect_err(|e| log::error!("{}", e)).unwrap();
                        return BlockInput::Block;
                    }
                }
                BlockInput::DontBlock
            });
        }

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

        // WARNING: not sure why this is required, but without it the module crashes
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
        Module::stop(self);
        self.paused = is_paused;
        Module::start(self);
    }

    fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfigs) {
        match event {
            MondrianMessage::Pause(pause) => Module::pause(self, pause.unwrap_or(!self.paused)),
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

    fn name(&self) -> String {
        "keybindings".to_string()
    }
}

impl ConfigurableModule for KeybindingsModule {
    type Config = KeybindingsModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}
