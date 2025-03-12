#[derive(Debug, PartialEq, Clone)]
pub enum InfoEntryIcon {
    Monitor,
    Window,
    Configs,
    Keybindings,
    TilesManager,
    General,
    Action,
    Enabled,
    Disabled,
    None,
}

impl From<InfoEntryIcon> for String {
    fn from(icon: InfoEntryIcon) -> Self {
        match icon {
            InfoEntryIcon::Monitor => "🖥️",
            InfoEntryIcon::Window => "🗔",
            InfoEntryIcon::TilesManager => "🔲",
            InfoEntryIcon::Configs => "⚙️",
            InfoEntryIcon::Keybindings => "⌨️",
            InfoEntryIcon::General => "🗃️",
            InfoEntryIcon::Action => "⚡️",
            InfoEntryIcon::Enabled => "🟢",
            InfoEntryIcon::Disabled => "🔴",
            InfoEntryIcon::None => "",
        }
        .to_string()
    }
}

impl From<InfoEntryIcon> for Option<String> {
    fn from(icon: InfoEntryIcon) -> Self {
        match icon {
            InfoEntryIcon::None => None,
            _ => Some(icon.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct InfoEntry {
    pub title: String,
    pub value: Option<String>,
    pub subentries: Vec<InfoEntry>,
    pub icon: InfoEntryIcon,
}

impl InfoEntry {
    pub fn new<T: Into<String>, V: Into<Option<String>>, E: IntoIterator<Item = InfoEntry>>(
        title: T,
        value: V,
        subentries: E,
        icon: InfoEntryIcon,
    ) -> InfoEntry {
        InfoEntry {
            title: title.into(),
            value: value.into(),
            subentries: subentries.into_iter().collect(),
            icon,
        }
    }

    pub fn single<T: Into<String>>(title: T) -> InfoEntry {
        InfoEntry::new(title, None, vec![], InfoEntryIcon::None)
    }

    pub fn simple<T: Into<String>, V: Into<String>>(title: T, value: V) -> InfoEntry {
        InfoEntry::new(title, Some(value.into()), vec![], InfoEntryIcon::None)
    }

    pub fn list<T: Into<String>, E: IntoIterator<Item = InfoEntry>>(title: T, subentries: E) -> InfoEntry {
        InfoEntry::new(title, None, subentries, InfoEntryIcon::None)
    }

    pub fn with_icon(mut self, icon: InfoEntryIcon) -> InfoEntry {
        self.icon = icon;
        self
    }
}

impl<T> From<T> for InfoEntry
where
    T: Into<String>,
{
    fn from(title: T) -> Self {
        InfoEntry::single(title)
    }
}
