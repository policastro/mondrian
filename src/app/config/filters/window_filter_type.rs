use crate::win32utils::window::window_obj::WindowObj;

use super::window_filter::WindowFilter;

#[derive(Debug, Clone)]
pub enum WindowFilterType {
    Exename(String),
    Title(String),
    Classname(String),
}

impl WindowFilterType {
    fn match_value(&self, query: &str, value: Option<String>) -> bool {
        match query.starts_with("/") && query.ends_with("/") {
            true => {
                let re = regex::Regex::new(&query[1..query.len() - 1]).unwrap();
                value.map_or(false, |v| re.is_match(&v))
            }
            false => value.map_or(false, |v| v.contains(query)),
        }
    }
}

impl WindowFilter for WindowFilterType {
    fn filter(&self, window: &impl WindowObj) -> bool {
        match self {
            WindowFilterType::Exename(query) => self.match_value(query, window.get_exe_name()),
            WindowFilterType::Title(query) => self.match_value(query, window.get_title()),
            WindowFilterType::Classname(query) => self.match_value(query, window.get_class_name()),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::app::structs::area::Area;

    use super::*;

    struct DummyWindow {
        title: String,
        exe_name: String,
        class_name: String,
    }

    fn create_win_info(exe_name: &str, title: &str, class_name: &str) -> DummyWindow {
        DummyWindow {
            title: title.to_string(),
            exe_name: exe_name.to_string(),
            class_name: class_name.to_string(),
        }
    }

    impl WindowObj for DummyWindow {
        fn get_title(&self) -> Option<String> {
            Some(self.title.clone())
        }

        fn get_exe_name(&self) -> Option<String> {
            Some(self.exe_name.clone())
        }

        fn get_class_name(&self) -> Option<String> {
            Some(self.class_name.clone())
        }

        fn get_window_box(&self) -> Option<Area> {
            todo!()
        }

        fn focus(&self) {
            todo!()
        }

        fn resize_and_move(&self, _coordinates: (u32, u32), _size: (u32, u32)) {
            todo!()
        }
    }

    #[test]
    fn test_win_exename_filter() {
        let f_exact = WindowFilterType::Exename("exename123.exe".to_string());
        let f_contains = WindowFilterType::Exename("exen".to_string());
        let f_wrong = WindowFilterType::Exename("notexen".to_string());
        let f_regex_exact = WindowFilterType::Exename("/^exename[1-3]{3}.exe$/".to_string());
        let f_regex_contains = WindowFilterType::Exename("/^exen/".to_string());
        let f_regex_wrong = WindowFilterType::Exename("/^notexen.*/".to_string());
        let dummy = &create_win_info("exename123.exe", "", "");

        assert!(f_exact.filter(dummy));
        assert!(f_contains.filter(dummy));
        assert!(!f_wrong.filter(dummy));
        assert!(f_regex_exact.filter(dummy));
        assert!(f_regex_contains.filter(dummy));
        assert!(!f_regex_wrong.filter(dummy));
    }

    #[test]
    fn test_win_title_filter() {
        let f_exact = WindowFilterType::Title("title123".to_string());
        let f_contains = WindowFilterType::Title("title".to_string());
        let f_wrong = WindowFilterType::Title("NotTitle".to_string());
        let f_regex_exact = WindowFilterType::Title("/^title[1-3]{3}$/".to_string());
        let f_regex_contains = WindowFilterType::Title("/^title/".to_string());
        let f_regex_wrong = WindowFilterType::Title("/^NotTitle.*/".to_string());
        let dummy = &create_win_info("", "title123", "");

        assert!(f_exact.filter(dummy));
        assert!(f_contains.filter(dummy));
        assert!(!f_wrong.filter(dummy));
        assert!(f_regex_exact.filter(dummy));
        assert!(f_regex_contains.filter(dummy));
        assert!(!f_regex_wrong.filter(dummy));
    }

    #[test]
    fn test_win_classname_filter() {
        let f_exact = WindowFilterType::Classname("class123".to_string());
        let f_contains = WindowFilterType::Classname("class".to_string());
        let f_wrong = WindowFilterType::Classname("NotClass".to_string());
        let f_regex_exact = WindowFilterType::Classname("/^class[1-3]{3}$/".to_string());
        let f_regex_contains = WindowFilterType::Classname("/^class/".to_string());
        let f_regex_wrong = WindowFilterType::Classname("/^NotClass.*/".to_string());
        let dummy = &create_win_info("", "", "class123");

        assert!(f_exact.filter(dummy));
        assert!(f_contains.filter(dummy));
        assert!(!f_wrong.filter(dummy));
        assert!(f_regex_exact.filter(dummy));
        assert!(f_regex_contains.filter(dummy));
        assert!(!f_regex_wrong.filter(dummy));
    }
}
