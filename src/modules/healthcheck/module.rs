use std::collections::HashMap;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use crossbeam_channel::RecvTimeoutError;
use crossbeam_channel::Sender;

use crate::app::configs::AppConfig;
use crate::app::mondrian_message::MondrianMessage;
use crate::modules::module_impl::ModuleImpl;

pub struct HealthCheck {
    modules_names: Vec<String>,
    tx: Sender<MondrianMessage>,
    main_thread_stop_tx: Option<crossbeam_channel::Sender<()>>,
    main_thread: Option<std::thread::JoinHandle<()>>,
    modules_health: Arc<Mutex<HashMap<String, Instant>>>,
}

impl HealthCheck {
    const CHECK_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);
    const CHECK_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(500);
    pub fn new(modules_names: Vec<String>, tx: Sender<MondrianMessage>) -> Self {
        Self {
            modules_names,
            tx,
            main_thread_stop_tx: None,
            main_thread: None,
            modules_health: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl ModuleImpl for HealthCheck {
    fn start(&mut self) {
        if self.main_thread.is_some() {
            return;
        }

        self.modules_health.lock().unwrap().clear();

        let tx = self.tx.clone();
        let modules_health = self.modules_health.clone();
        let modules_names = self.modules_names.clone();
        let (main_thread_tx, main_thread_rx) = crossbeam_channel::bounded::<()>(1);
        self.main_thread_stop_tx = Some(main_thread_tx);
        let main_thread = std::thread::spawn(move || {
            let mut last_ping = std::time::Instant::now();
            let mut lock = modules_health.lock().unwrap();
            for module_name in &modules_names {
                lock.insert(module_name.clone(), last_ping);
            }
            drop(lock);
            while let Err(RecvTimeoutError::Timeout) = main_thread_rx.recv_timeout(HealthCheck::CHECK_INTERVAL) {
                last_ping = std::time::Instant::now();
                tx.send(MondrianMessage::HealthCheckPing).ok();

                match main_thread_rx.recv_timeout(HealthCheck::CHECK_TIMEOUT) {
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => (),
                    _ => break,
                }

                for (module_name, last_pong) in modules_health.lock().unwrap().iter().filter(|(_, l)| **l < last_ping) {
                    log::error!(
                        "[Health check] Module `{}` did not respond for {}ms",
                        module_name,
                        (last_ping - *last_pong).as_millis()
                    );
                }
            }
        });

        self.main_thread = Some(main_thread);
    }

    fn stop(&mut self) {
        if let Some(main_thread) = self.main_thread.take() {
            self.main_thread_stop_tx.take().unwrap().send(()).unwrap();
            main_thread.join().unwrap();
        }
    }

    fn restart(&mut self) {}
    fn pause(&mut self, _is_paused: bool) {}
    fn enable(&mut self, _enabled: bool) {}
    fn enabled(&self) -> bool {
        true
    }

    fn handle(&mut self, event: &MondrianMessage, _app_configs: &AppConfig, _tx: &Sender<MondrianMessage>) {
        if let MondrianMessage::HealthCheckPong { module_name } = event {
            if self.modules_names.contains(module_name) {
                self.modules_health
                    .lock()
                    .unwrap()
                    .insert(module_name.clone(), Instant::now());
            }
        };
        if matches!(event, MondrianMessage::Quit) {
            self.stop();
        }
    }

    fn name(&self) -> String {
        "healthcheck".to_string()
    }
}
