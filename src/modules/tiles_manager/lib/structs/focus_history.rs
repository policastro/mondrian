use crate::app::area_tree::leaf::AreaLeaf;
use crate::win32::window::window_ref::WindowRef;
use std::collections::HashMap;
use std::collections::VecDeque;

#[derive(Default, Clone, Debug)]
pub struct FocusHistory {
    map: HashMap<WindowRef, u64>,
    order: VecDeque<WindowRef>,
    order_map: HashMap<WindowRef, usize>,
    current_max: u64,
}

impl FocusHistory {
    const MAX_ENTRIES: usize = 100; // TODO: maybe should be configurable?

    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            order_map: HashMap::new(),
            current_max: 0,
        }
    }

    pub fn value(&self, window: &WindowRef) -> Option<u64> {
        self.map.get(window).copied()
    }

    pub fn update(&mut self, window: WindowRef) {
        // INFO: this should pratically never happen
        if self.current_max == u64::MAX {
            let min_v = *self.map.values().min().unwrap_or(&0);
            self.map.values_mut().for_each(|v| *v -= min_v);
            self.current_max -= min_v;
        }

        if let Some(p) = self.order_map.get(&window) {
            self.order.remove(*p);
        }

        self.current_max += 1;
        self.order.push_back(window);
        self.order_map.insert(window, self.order.len() - 1);
        self.map.insert(window, self.current_max);

        // INFO: prevent map from growing indefinitely
        if self.order.len() > Self::MAX_ENTRIES {
            let w = self.order.pop_front().unwrap();
            self.map.remove(&w);
            self.order_map.remove(&w);
        }
    }

    pub fn most_recent_win<'a>(&self, windows: impl IntoIterator<Item = &'a WindowRef>) -> Option<&'a WindowRef> {
        self.most_recent_by(windows, |w| w)
    }

    pub fn most_recent_leaf<'a>(
        &self,
        windows: impl IntoIterator<Item = &'a AreaLeaf<WindowRef>>,
    ) -> Option<&'a AreaLeaf<WindowRef>> {
        self.most_recent_by(windows, |l| &l.id)
    }

    pub fn most_recent_by<'a, T>(
        &self,
        windows: impl IntoIterator<Item = &'a T>,
        key: impl Fn(&'a T) -> &'a WindowRef,
    ) -> Option<&'a T> {
        windows
            .into_iter()
            .filter_map(|w| self.value(key(w)).map(|v| (w, v)))
            .max_by_key(|(_, v)| *v)
            .map(|(w, _)| w)
    }
}
