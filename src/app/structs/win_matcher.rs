use crate::win32::window::window_obj::WindowObjInfo;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashSet;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "WinMatcherExt")]
pub enum WinMatcher {
    Exename(String),
    Title(String),
    Classname(String),
    Style(String),
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
                value.as_ref().is_none_or(|v| re.is_match(v))
            }
            false => value.as_ref().is_none_or(|v| v.contains(query)),
        }
    }

    pub fn matches<T: WindowObjInfo>(&self, window: T) -> bool {
        self.matches_internal(&mut window.into())
    }

    pub fn is_empty(&self) -> bool {
        match self {
            WinMatcher::Any(filters) => filters.is_empty(),
            WinMatcher::All(filters) => filters.is_empty(),
            _ => false,
        }
    }

    pub fn any(filters: impl Iterator<Item = WinMatcher>) -> WinMatcher {
        WinMatcher::Any(filters.collect())
    }

    fn matches_internal<T: WindowObjInfo>(&self, window: &mut WinMatcherTarget<T>) -> bool {
        match self {
            WinMatcher::Exename(query) => self.match_value(query, window.exe_name()),
            WinMatcher::Title(query) => self.match_value(query, window.title()),
            WinMatcher::Classname(query) => self.match_value(query, window.class_name()),
            WinMatcher::Style(query) => self.match_value(query, &Some(window.style())),
            WinMatcher::Any(filters) => filters.iter().any(|f| f.matches_internal(window)),
            WinMatcher::All(filters) => !filters.is_empty() && filters.iter().all(|f| f.matches_internal(window)),
        }
    }
}

pub struct WinMatcherTarget<T: WindowObjInfo> {
    win_obj: T,
    title: Option<Option<String>>,
    class_name: Option<Option<String>>,
    exe_name: Option<Option<String>>,
    style: Option<String>,
}

impl<T: WindowObjInfo> From<T> for WinMatcherTarget<T> {
    fn from(value: T) -> Self {
        WinMatcherTarget {
            win_obj: value,
            title: None,
            class_name: None,
            exe_name: None,
            style: None,
        }
    }
}

impl<T: WindowObjInfo> WinMatcherTarget<T> {
    pub fn title(&mut self) -> &Option<String> {
        self.title.get_or_insert(self.win_obj.get_title())
    }

    pub fn class_name(&mut self) -> &Option<String> {
        self.class_name.get_or_insert(self.win_obj.get_class_name())
    }

    pub fn exe_name(&mut self) -> &Option<String> {
        self.exe_name.get_or_insert(self.win_obj.get_exe_name())
    }

    pub fn style(&mut self) -> String {
        self.style
            .get_or_insert(format!("{:x}", self.win_obj.get_window_style()))
            .clone()
    }
}

impl std::hash::Hash for WinMatcher {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(deny_unknown_fields)]
struct WinMatcherExt {
    pub classname: Option<String>,
    pub exename: Option<String>,
    pub title: Option<String>,
    pub style: Option<String>,
}

impl From<&WinMatcherExt> for WinMatcher {
    fn from(v: &WinMatcherExt) -> Self {
        let mut matchers: Vec<WinMatcher> = Vec::new();

        if let Some(exename) = &v.exename {
            matchers.push(WinMatcher::Exename(exename.clone()));
        }

        if let Some(classname) = &v.classname {
            matchers.push(WinMatcher::Classname(classname.clone()));
        }

        if let Some(title) = &v.title {
            matchers.push(WinMatcher::Title(title.clone()));
        }

        if let Some(style) = &v.style {
            matchers.push(WinMatcher::Style(style.clone()));
        }

        match matchers.len() == 1 {
            true => matchers[0].clone(),
            false => WinMatcher::All(matchers.into_iter().collect()),
        }
    }
}

impl From<WinMatcherExt> for WinMatcher {
    fn from(v: WinMatcherExt) -> Self {
        WinMatcher::from(&v)
    }
}
