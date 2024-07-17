use crate::win32::window::window_obj::WindowObjInfo;

use super::window_filter::WindowFilter;

#[derive(Debug, Clone)]
pub enum WindowFilterType {
    Exename(String),
    Title(String),
    Classname(String),
}

impl WindowFilterType {
    fn match_value(&self, query: &str, value: Option<String>) -> bool {
        match query.starts_with('/') && query.ends_with('/') {
            true => {
                let re = regex::Regex::new(&query[1..query.len() - 1]).unwrap();
                value.map_or(false, |v| re.is_match(&v))
            }
            false => value.map_or(false, |v| v.contains(query)),
        }
    }
}

impl WindowFilter for WindowFilterType {
    fn filter(&self, window: &impl WindowObjInfo) -> bool {
        match self {
            WindowFilterType::Exename(query) => self.match_value(query, window.get_exe_name()),
            WindowFilterType::Title(query) => self.match_value(query, window.get_title()),
            WindowFilterType::Classname(query) => self.match_value(query, window.get_class_name()),
        }
    }
}
