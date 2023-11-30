use crate::win32utils::window::window_obj::WindowObj;

use super::{window_filter::WindowFilter, window_filter_type::WindowFilterType};

#[derive(Debug, Clone)]
pub struct WinMatchAllFilters {
    filters: Vec<WindowFilterType>,
}

impl WinMatchAllFilters {
    pub fn new(filters: Vec<WindowFilterType>) -> WinMatchAllFilters {
        WinMatchAllFilters { filters }
    }
}

impl WindowFilter for WinMatchAllFilters {
    fn filter(&self, window: &impl WindowObj) -> bool {
        self.filters.iter().all(|filter| filter.filter(window))
    }
}

#[derive(Debug, Clone)]
pub struct WinMatchAnyFilters {
    filters: Vec<WinMatchAllFilters>,
}

impl WinMatchAnyFilters {
    pub fn new(filters: Vec<WinMatchAllFilters>) -> WinMatchAnyFilters {
        WinMatchAnyFilters { filters }
    }
}

impl WindowFilter for WinMatchAnyFilters {
    fn filter(&self, window: &impl WindowObj) -> bool {
        self.filters.iter().any(|filter| filter.filter(window))
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
    fn test_win_match_all() {
        let filter = WinMatchAllFilters::new(vec![
            WindowFilterType::Title("title".to_string()),
            WindowFilterType::Exename("exe_name".to_string()),
            WindowFilterType::Classname("class_name".to_string()),
        ]);

        let bad_filter = WinMatchAllFilters::new(vec![
            WindowFilterType::Title("title1".to_string()),
            WindowFilterType::Exename("exe_name".to_string()),
            WindowFilterType::Classname("class_name".to_string()),
        ]);

        let window = create_win_info("exe_name", "title", "class_name");
        assert!(filter.filter(&window));
        assert!(!bad_filter.filter(&window));
    }

    #[test]
    fn test_win_match_any() {
        let filter = WinMatchAnyFilters::new(vec![
            WinMatchAllFilters::new(vec![
                WindowFilterType::Title("title".to_string()),
                WindowFilterType::Exename("exe_name".to_string()),
                WindowFilterType::Classname("class_name".to_string()),
            ]),
            WinMatchAllFilters::new(vec![
                WindowFilterType::Title("title1".to_string()),
                WindowFilterType::Exename("exe_name1".to_string()),
                WindowFilterType::Classname("class_name1".to_string()),
            ]),
        ]);

        let bad_filter = WinMatchAnyFilters::new(vec![
            WinMatchAllFilters::new(vec![
                WindowFilterType::Title("title1".to_string()),
                WindowFilterType::Exename("exe_name".to_string()),
                WindowFilterType::Classname("class_name".to_string()),
            ]),
            WinMatchAllFilters::new(vec![
                WindowFilterType::Title("title1".to_string()),
                WindowFilterType::Exename("exe_name1".to_string()),
                WindowFilterType::Classname("class_name1".to_string()),
            ]),
        ]);

        let window = create_win_info("exe_name", "title", "class_name");
        assert!(filter.filter(&window));
        assert!(!bad_filter.filter(&window));
    }
}
