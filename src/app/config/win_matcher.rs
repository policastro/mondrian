use crate::win32::window::window_obj::WindowObjInfo;

#[derive(Clone, Debug)]
pub enum WinMatcher {
    Exename(String),
    Title(String),
    Classname(String),
    Any(Vec<WinMatcher>),
    All(Vec<WinMatcher>),
}

impl WinMatcher {
    fn match_value(&self, query: &str, value: Option<String>) -> bool {
        match query.starts_with('/') && query.ends_with('/') {
            true => {
                let re = regex::Regex::new(&query[1..query.len() - 1]).unwrap();
                value.map_or(false, |v| re.is_match(&v))
            }
            false => value.map_or(false, |v| v.contains(query)),
        }
    }
    pub fn matches(&self, window: &impl WindowObjInfo) -> bool {
        match self {
            WinMatcher::Exename(query) => self.match_value(query, window.get_exe_name()),
            WinMatcher::Title(query) => self.match_value(query, window.get_title()),
            WinMatcher::Classname(query) => self.match_value(query, window.get_class_name()),
            WinMatcher::Any(filters) => filters.iter().any(|f| f.matches(window)),
            WinMatcher::All(filters) => filters.iter().all(|f| f.matches(window)),
        }
    }
}
