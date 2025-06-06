use crate::app::configs::AppConfig;
use crate::app::mondrian_message::MondrianMessage;
use crate::app::structs::info_entry::InfoEntry;
use crate::app::structs::info_entry::InfoEntryIcon;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::ConfigurableModule;
use crate::modules::Module;
use crate::win32::api::misc::get_current_thread_id;
use crate::win32::api::misc::post_empty_thread_message;
use crossbeam_channel::Sender;
use inputbot::BlockInput;
use inputbot::KeybdKey;
use inputbot::KeybdKey::*;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::thread;
use windows::Win32::UI::WindowsAndMessaging::WM_QUIT;

use super::configs::KeybindingsModuleConfigs;
pub struct Keybindings {
    bus: Sender<MondrianMessage>,
    input_thread: Option<thread::JoinHandle<()>>,
    enabled: bool,
    running: Arc<AtomicBool>,
    configs: KeybindingsModuleConfigs,
    binded_keys: Vec<inputbot::KeybdKey>,
    main_thread_id: Arc<AtomicU32>,
    paused: bool,
}

impl Keybindings {
    pub fn new(bus: Sender<MondrianMessage>) -> Keybindings {
        Keybindings {
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

impl ModuleImpl for Keybindings {
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

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfig) {
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
            MondrianMessage::SystemEvent(evt) => match evt {
                evt if evt.session_is_active() => Module::start(self),
                evt if evt.session_is_inactive() => Module::stop(self),
                _ => {}
            },
            MondrianMessage::QueryInfo => {
                self.bus
                    .send(MondrianMessage::QueryInfoResponse {
                        name: "Keybindings".to_string(),
                        icon: InfoEntryIcon::Keybindings,
                        infos: get_info_entries(&self.configs),
                    })
                    .ok();
            }
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }

    fn name(&self) -> String {
        "keybindings".to_string()
    }
}

impl ConfigurableModule for Keybindings {
    type Config = KeybindingsModuleConfigs;
    fn configure(&mut self, config: Self::Config) {
        self.configs = config;
    }
}

fn get_info_entries(configs: &KeybindingsModuleConfigs) -> Vec<InfoEntry> {
    let (answer, icon) = match configs.enabled {
        true => ("Yes", InfoEntryIcon::Enabled),
        false => ("No", InfoEntryIcon::Disabled),
    };
    let enabled_info = InfoEntry::simple("Enabled", answer).with_icon(icon);

    let mappings: Vec<(String, String)> = configs
        .bindings
        .iter()
        .map(|b| {
            let mods: Vec<String> = b
                .modifiers()
                .iter()
                .map(|k| {
                    k.to_string()
                        .replace("Left ", "")
                        .replace("Windows", "WIN")
                        .replace("Control", "CTRL")
                })
                .collect();

            let binding = match mods.is_empty() {
                true => b.key().to_string(),
                false => format!("{}+{}", mods.join("+"), b.key()),
            };

            (binding.to_uppercase(), format!("{:?}", b.action()))
        })
        .collect();

    let mappings_entries = mappings.iter().map(|b| InfoEntry::simple(b.0.clone(), b.1.clone()));
    let mappings_infos = InfoEntry::list("Mappings", mappings_entries).with_icon(InfoEntryIcon::Action);

    vec![enabled_info, mappings_infos]
}
