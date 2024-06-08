#[derive(Debug, PartialEq, Clone, Copy)]
pub enum MondrianAppEvent {
    RefreshConfig,
    OpenConfig,
    Retile,
    Pause(bool),
    Quit,
}

impl Into<&str> for MondrianAppEvent {
    fn into(self) -> &'static str {
        match self {
            MondrianAppEvent::RefreshConfig => "REFRESH_CONFIG",
            MondrianAppEvent::OpenConfig => "OPEN_CONFIG",
            MondrianAppEvent::Retile => "RETILE",
            MondrianAppEvent::Pause(true) => "PAUSE",
            MondrianAppEvent::Pause(false) => "RESUME",
            MondrianAppEvent::Quit => "QUIT",
        }
    }
}
