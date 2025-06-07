use crate::app::configs::AppConfig;
use crate::app::mondrian_message::MondrianMessage;
use crate::modules::module_impl::ModuleImpl;
use crate::modules::utils;
use crate::modules::Module;
use crossbeam_channel::Sender;
use notify::RecommendedWatcher;
use notify_debouncer_mini::DebounceEventResult;
use notify_debouncer_mini::DebouncedEventKind;
use notify_debouncer_mini::Debouncer;
use std::path::Path;
use std::path::PathBuf;
use std::time::Duration;

pub struct FileWatcher {
    config_watcher_debouncer: Option<Debouncer<RecommendedWatcher>>,
    config_path: PathBuf,
    running: bool,
    enabled: bool,
    bus_tx: Sender<MondrianMessage>,
}

impl FileWatcher {
    pub fn new<P: AsRef<Path>>(bus_tx: Sender<MondrianMessage>, config_path: P) -> Self {
        FileWatcher {
            config_watcher_debouncer: None,
            running: false,
            enabled: true,
            config_path: config_path.as_ref().to_path_buf(),
            bus_tx,
        }
    }

    fn start_watching_config(&mut self) {
        let bus_tx = self.bus_tx.clone();

        let watcher_debouncer =
            notify_debouncer_mini::new_debouncer(Duration::from_millis(200), move |res: DebounceEventResult| {
                if let Ok(events) = res {
                    events
                        .iter()
                        .filter(|e| e.kind == DebouncedEventKind::Any)
                        .take(1)
                        .for_each(|_| bus_tx.send(MondrianMessage::RefreshConfig).unwrap())
                }
            })
            .inspect_err(|_| log::warn!("Error creating config watcher debouncer"));

        if let Ok(mut wd) = watcher_debouncer {
            let res = wd
                .watcher()
                .watch(&self.config_path, notify::RecursiveMode::NonRecursive)
                .inspect_err(|_| log::error!("Error watching config file"));

            if res.is_ok() {
                self.config_watcher_debouncer = Some(wd);
                self.running = true;
            }
        }
    }
}

impl ModuleImpl for FileWatcher {
    fn start(&mut self) {
        if self.running {
            return;
        }

        self.start_watching_config();

        log::trace!("FileWatcherModule started");
    }

    fn stop(&mut self) {
        if !self.running {
            return;
        }

        if let Some(mut wd) = self.config_watcher_debouncer.take() {
            let _ = wd.watcher().unwatch(&self.config_path);
        }

        self.running = false;
        log::trace!("FileWatcherModule stopped");
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

    fn enabled(&self) -> bool {
        self.enabled
    }

    fn enable(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    fn handle(&mut self, event: &MondrianMessage, app_configs: &AppConfig, _tx: &Sender<MondrianMessage>) {
        match event {
            MondrianMessage::Pause(pause) => Module::pause(self, pause.unwrap_or(self.running)),
            MondrianMessage::Configure => {
                Module::enable(self, app_configs.auto_reload_configs);
            }
            MondrianMessage::RefreshConfig => {
                Module::enable(self, app_configs.auto_reload_configs);
                Module::restart(self);
            }
            MondrianMessage::SystemEvent(evt) => match evt {
                evt if evt.session_is_active() => Module::start(self),
                evt if evt.session_is_inactive() => Module::stop(self),
                _ => {}
            },
            MondrianMessage::HealthCheckPing => utils::send_pong(&Module::name(self), &self.bus_tx),
            MondrianMessage::Quit => Module::stop(self),
            _ => {}
        }
    }

    fn name(&self) -> String {
        "file-watcher".to_string()
    }
}
