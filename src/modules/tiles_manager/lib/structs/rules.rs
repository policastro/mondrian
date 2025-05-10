use crate::{
    app::configs::{
        floating::FloatingWinsConfig,
        rules::{WindowBehavior, WindowRule},
    },
    win32::window::window_ref::WindowRef,
};

#[derive(Default, Debug, Clone)]
pub struct WorkspaceOptions {
    pub workspace: String,
    pub silent: bool,
}

#[derive(Default, Debug)]
pub struct AddOptions {
    pub monitor: Option<String>,
    pub workspace_options: Option<WorkspaceOptions>,
    pub floating_config: Option<FloatingWinsConfig>,
}

impl AddOptions {
    pub fn merge_with_rule(&mut self, other: &WindowRule) {
        match &other.behavior {
            WindowBehavior::Float { config } => self.floating_config = Some(config.clone()),
            WindowBehavior::Insert {
                monitor,
                workspace,
                silent,
            } => {
                self.monitor = monitor.clone();
                self.workspace_options = workspace.as_ref().map(|w| WorkspaceOptions {
                    workspace: w.clone(),
                    silent: *silent,
                });
            }
        }
    }
}

pub trait Rules {
    fn get_add_options(&self, window: WindowRef) -> Option<AddOptions>;
}

impl Rules for Vec<WindowRule> {
    fn get_add_options(&self, window: WindowRef) -> Option<AddOptions> {
        let rules = find_matches(self, window);
        let mut empty = true;
        let mut options = AddOptions::default();
        rules.for_each(|rule| {
            empty = false;
            options.merge_with_rule(rule);
        });
        match empty {
            true => return None,
            false => Some(options),
        }
    }
}

fn find_matches(rules: &[WindowRule], window: WindowRef) -> impl Iterator<Item = &WindowRule> {
    rules.iter().filter(move |r| r.filter.matches(window))
}
