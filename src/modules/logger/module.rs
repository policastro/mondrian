use crate::app::configs::AppConfigs;
use crate::app::mondrian_message::MondrianMessage;
use crate::modules::module_impl::ModuleImpl;
use crate::win32::window::window_obj::WindowObjInfo;
use info_response_builder::build_info_response;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;

pub struct LoggerModule;

impl LoggerModule {
    const APP_STATE_FILENAME: &str = "./logs/app_state.txt";
    const COL_WIDTH: usize = 80;

    pub fn new() -> LoggerModule {
        LoggerModule {}
    }

    pub fn get_app_state_file() -> File {
        OpenOptions::new()
            .create(true)
            .append(true)
            .open(Self::APP_STATE_FILENAME)
            .expect("cannot open file")
    }
}

impl ModuleImpl for LoggerModule {
    fn start(&mut self) {}
    fn stop(&mut self) {}
    fn restart(&mut self) {}
    fn pause(&mut self, _is_paused: bool) {}
    fn enable(&mut self, _enabled: bool) {}
    fn enabled(&self) -> bool {
        true
    }

    fn handle(&mut self, event: &MondrianMessage, _app_configs: &AppConfigs) {
        match event {
            MondrianMessage::WindowEvent(e) => {
                let wref = e.get_window_ref();

                log::info!(
                    "[Window:{:?} | {}{}]: {{ class: {}, style: {}, [{}visible, {}iconic, {}cloaked], view: {} }}",
                    e,
                    wref.get_exe_name().unwrap_or("".to_string()),
                    if cfg!(debug_assertions) {
                        format!(" ({:?})", wref.get_title().unwrap_or("".to_string()))
                    } else {
                        "".to_string() // INFO: no title in prod
                    },
                    wref.get_class_name().unwrap_or("".to_string()),
                    format!("{:x}", wref.get_window_style()),
                    if wref.is_visible() { "" } else { "!" },
                    if wref.is_iconic() { "" } else { "!" },
                    if wref.is_cloaked() { "" } else { "!" },
                    wref.get_visible_area()
                        .map(|a| format!("({}, {}, {}, {})", a.x, a.y, a.width, a.height))
                        .unwrap_or("()".to_string()),
                );
            }
            MondrianMessage::QueryInfo => {
                let mut data_file = Self::get_app_state_file();

                let col_width = Self::COL_WIDTH;
                let divider = "=".repeat(col_width);
                let current_date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
                writeln!(data_file, "{divider}").ok();
                writeln!(data_file, "{:^col_width$}", "📝 App State Dump").ok();
                writeln!(data_file, "  {current_date:^col_width$}").ok();
                writeln!(data_file, "{divider}\n").ok();
            }
            MondrianMessage::QueryInfoResponse { name, icon, infos } => {
                let mut data_file = Self::get_app_state_file();

                let col_width = Self::COL_WIDTH;
                let divider = "-".repeat(Self::COL_WIDTH);
                let icon_str: Option<String> = icon.clone().into();
                let icon_str = icon_str.map(|s| format!("{} ", s)).unwrap_or_default();
                writeln!(data_file, "{divider}").ok();
                writeln!(data_file, "{:^col_width$}", format!("[ {icon_str}{name} ]")).ok();
                writeln!(data_file, "{divider}").ok();
                writeln!(data_file, "{}", build_info_response(infos, 3, "▸")).ok();
            }
            _ => log::trace!("{:?}", event),
        }
    }

    fn name(&self) -> String {
        "logger".to_string()
    }
}

mod info_response_builder {
    use crate::app::structs::info_entry::InfoEntry;

    pub fn build_info_response<P: Into<String> + Clone + Copy>(
        infos: &Vec<InfoEntry>,
        indent_step: usize,
        prefix: P,
    ) -> String {
        let result_string = &mut String::new();
        build_info_response_rec(result_string, infos, 0, indent_step, prefix);
        result_string.to_string()
    }

    fn build_info_response_rec<P: Into<String> + Clone + Copy>(
        result_string: &mut String,
        infos: &Vec<InfoEntry>,
        indent: usize,
        indent_step: usize,
        prefix: P,
    ) {
        let spacing = " ".repeat(indent_step).repeat(indent);
        let prefix_str = match indent > 0 {
            true => &prefix.into().to_string(),
            false => "",
        };

        for info in infos {
            if indent == 0 {
                result_string.push('\n');
            }
            let icon: String = info.icon.clone().into();
            let text = match &info.value {
                None => format!("{spacing}{prefix_str}{icon} {}\n", info.title),
                Some(v) => format!("{spacing}{prefix_str}{icon} {}: {}\n", info.title, v),
            };
            result_string.push_str(&text);
            build_info_response_rec(result_string, &info.subentries, indent + 1, indent_step, prefix);
        }
    }
}
