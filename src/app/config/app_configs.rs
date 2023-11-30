use serde::{Deserialize, Serialize};

use super::filters::{
    window_filter_type::WindowFilterType,
    window_match_filter::{WinMatchAllFilters, WinMatchAnyFilters},
};

#[derive(Serialize, Deserialize, Debug)]
struct FilterJsonConfig {
    classname: Option<String>,
    exename: Option<String>,
    title: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct JsonConfig {
    filters: Option<Vec<FilterJsonConfig>>,
}

impl From<&FilterJsonConfig> for WinMatchAllFilters {
    fn from(filters: &FilterJsonConfig) -> Self {
        let mut window_filters: Vec<WindowFilterType> = Vec::new();

        if let Some(exename) = &filters.exename {
            window_filters.push(WindowFilterType::Exename(exename.clone()));
        }

        if let Some(classname) = &filters.classname {
            window_filters.push(WindowFilterType::Classname(classname.clone()));
        }

        if let Some(title) = &filters.title {
            window_filters.push(WindowFilterType::Title(title.clone()));
        }

        if window_filters.is_empty() {
            panic!("The filter must specify at least one field between 'exename', 'classname' and 'title'.")
        }

        WinMatchAllFilters::new(window_filters)
    }
}

impl From<Vec<FilterJsonConfig>> for WinMatchAnyFilters {
    fn from(filters: Vec<FilterJsonConfig>) -> Self {
        WinMatchAnyFilters::new(
            filters
                .iter()
                .map(|f| WinMatchAllFilters::from(f))
                .collect::<Vec<WinMatchAllFilters>>(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct AppConfigs {
    pub filter: Option<WinMatchAnyFilters>,
}

impl AppConfigs {
    pub fn new(filters: Option<WinMatchAnyFilters>) -> AppConfigs {
        AppConfigs { filter: filters }
    }

    pub fn from_file(path: &str) -> AppConfigs {
        let file = std::fs::File::open(path).unwrap();
        let configs: JsonConfig = serde_json::from_reader(file).unwrap();

        let filters: Option<WinMatchAnyFilters> = match configs.filters {
            Some(filters) => Some(WinMatchAnyFilters::from(filters)),
            None => None,
        };
        AppConfigs::new(filters)
    }
}
