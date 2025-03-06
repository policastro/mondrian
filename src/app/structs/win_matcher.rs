use std::collections::HashSet;

use crate::win32::window::window_obj::WindowObjInfo;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WinMatcher {
    Exename(String),
    Title(String),
    Classname(String),
    Any(HashSet<WinMatcher>),
    All(HashSet<WinMatcher>),
}

impl Default for WinMatcher {
    fn default() -> Self {
        WinMatcher::Any(HashSet::new())
    }
}

impl WinMatcher {
    fn match_value(&self, query: &str, value: &Option<String>) -> bool {
        match query.starts_with('/') && query.ends_with('/') {
            true => {
                let re = regex::Regex::new(&query[1..query.len() - 1]).unwrap();
                value.as_ref().map_or(false, |v| re.is_match(v))
            }
            false => value.as_ref().map_or(false, |v| v.contains(query)),
        }
    }

    pub fn matches<T: WindowObjInfo>(&self, window: T) -> bool {
        self.matches_internal(&mut window.into())
    }

    fn matches_internal<T: WindowObjInfo>(&self, window: &mut WinMatcherTarget<T>) -> bool {
        match self {
            WinMatcher::Exename(query) => self.match_value(query, window.exe_name()),
            WinMatcher::Title(query) => self.match_value(query, window.title()),
            WinMatcher::Classname(query) => self.match_value(query, window.class_name()),
            WinMatcher::Any(filters) => filters.iter().any(|f| f.matches_internal(window)),
            WinMatcher::All(filters) => filters.iter().all(|f| f.matches_internal(window)),
        }
    }
}

pub struct WinMatcherTarget<T: WindowObjInfo> {
    win_obj: T,
    title: Option<Option<String>>,
    class_name: Option<Option<String>>,
    exe_name: Option<Option<String>>,
}

impl<T: WindowObjInfo> From<T> for WinMatcherTarget<T> {
    fn from(value: T) -> Self {
        WinMatcherTarget {
            win_obj: value,
            title: None,
            class_name: None,
            exe_name: None,
        }
    }
}

impl<T: WindowObjInfo> WinMatcherTarget<T> {
    pub fn title(&mut self) -> &Option<String> {
        self.title.get_or_insert_with(|| self.win_obj.get_title())
    }

    pub fn class_name(&mut self) -> &Option<String> {
        self.class_name.get_or_insert_with(|| self.win_obj.get_class_name())
    }

    pub fn exe_name(&mut self) -> &Option<String> {
        self.exe_name.get_or_insert_with(|| self.win_obj.get_exe_name())
    }
}

impl std::hash::Hash for WinMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}
