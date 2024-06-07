use std::{fs, path::PathBuf};

use serde::{Deserialize, Serialize};

use super::filters::{
    window_filter_type::WindowFilterType,
    window_match_filter::{WinMatchAllFilters, WinMatchAnyFilters},
};

#[derive(Serialize, Deserialize, Debug)]
struct FilterExternalConfig {
    classname: Option<String>,
    exename: Option<String>,
    title: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct ExternalConfig {
    filters: Option<Vec<FilterExternalConfig>>,
}

impl From<&FilterExternalConfig> for WinMatchAllFilters {
    fn from(filters: &FilterExternalConfig) -> Self {
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

impl From<Vec<FilterExternalConfig>> for WinMatchAnyFilters {
    fn from(filters: Vec<FilterExternalConfig>) -> Self {
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

    pub fn from_file(path: &PathBuf) -> AppConfigs {
        let file_content = std::fs::read_to_string(path).expect("Something went wrong reading the file");

        let mut configs: ExternalConfig = toml::from_str(&file_content).unwrap();

        // Needed to prevent the tray icon app from being filtered
        let base_filter = FilterExternalConfig {
            exename: Some("mondrian.exe".to_owned()),
            classname: Some("tray_icon_app".to_owned()),
            title: Some("".to_owned()),
        };

        configs.filters.as_mut().unwrap().push(base_filter);

        let filters: Option<WinMatchAnyFilters> = match configs.filters {
            Some(filters) => Some(WinMatchAnyFilters::from(filters)),
            None => None,
        };

        AppConfigs::new(filters)
    }
}
