use crate::app::structs::area::Area;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::f32::consts::PI;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use windows::Win32::UI::WindowsAndMessaging::SET_WINDOW_POS_FLAGS;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOACTIVATE;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOSENDCHANGING;
use windows::Win32::UI::WindowsAndMessaging::SWP_NOZORDER;
use windows::Win32::UI::WindowsAndMessaging::SWP_SHOWWINDOW;

pub struct WindowAnimationPlayer {
    windows: HashMap<WindowRef, WindowAnimationQueueInfo>,
    running: Arc<AtomicBool>,
    animation_thread: Option<std::thread::JoinHandle<()>>,
    animation_duration: Duration,
    framerate: u8,
    previous_foreground: Option<WindowRef>,
    on_start: Arc<dyn Fn(HashSet<WindowRef>) + Send + Sync + 'static>,
    on_error: Arc<dyn Fn() + Send + Sync + 'static>,
    on_complete: Arc<dyn Fn() + Send + Sync + 'static>,
}

#[derive(Clone)]
pub struct WindowAnimationQueueInfo {
    target_area: Area,
    topmost: bool,
}

impl WindowAnimationQueueInfo {
    pub fn new(new_area: Area, topmost: bool) -> Self {
        WindowAnimationQueueInfo {
            target_area: new_area,
            topmost,
        }
    }
}

impl WindowAnimationPlayer {
    pub fn new<S, E, C>(animation_duration: Duration, framerate: u8, on_start: S, on_error: E, on_complete: C) -> Self
    where
        S: Fn(HashSet<WindowRef>) + Sync + Send + 'static,
        E: Fn() + Sync + Send + 'static,
        C: Fn() + Sync + Send + 'static,
    {
        assert!(animation_duration.as_millis() > 0);
        assert!(framerate > 0);
        WindowAnimationPlayer {
            windows: HashMap::new(),
            running: Arc::new(AtomicBool::new(false)),
            animation_thread: None,
            animation_duration,
            framerate,
            previous_foreground: None,
            on_start: Arc::new(on_start),
            on_error: Arc::new(on_error),
            on_complete: Arc::new(on_complete),
        }
    }

    pub fn queue(&mut self, window: WindowRef, new_area: Area, topmost: bool) {
        if self.running.load(Ordering::Acquire) {
            self.clear();
        }

        self.windows
            .insert(window, WindowAnimationQueueInfo::new(new_area, topmost));
    }

    pub(crate) fn dequeue(&mut self, window: WindowRef) {
        self.windows.remove(&window);
    }

    pub fn clear(&mut self) {
        self.running.store(false, Ordering::Release);
        self.windows.clear();
        if let Some(t) = self.animation_thread.take() {
            t.join().unwrap();
        }

        if let Some(hwnd) = self.previous_foreground.take() {
            hwnd.focus();
        }
    }

    pub fn play(&mut self, animation: Option<WindowAnimation>) {
        self.cancel();
        self.previous_foreground = get_foreground_window()
            .map(|fw| fw.into())
            .filter(|fw| self.windows.iter().any(|(w, _)| w == fw));

        let wins = self.windows.clone();
        let wins_info: Vec<(WindowRef, Area, Area)> = wins
            .clone()
            .into_iter()
            .filter_map(|(w, i)| {
                let src_area: Area = w.get_area()?;
                if src_area == i.target_area {
                    return None;
                };
                Some((w, src_area, i.target_area))
            })
            .collect();

        let wins_to_animate: HashSet<WindowRef> = wins_info.iter().map(|(w, _, _)| *w).collect();
        if animation.is_none() {
            (self.on_start)(wins_to_animate);
            Self::move_windows(&self.windows);
            (self.on_complete)();
            self.clear();
            return;
        }

        let animation = animation.unwrap().clone();
        let running = self.running.clone();
        let duration = self.animation_duration.as_millis();
        let framerate = self.framerate;
        let frame_duration = Duration::from_millis(1000 / u64::from(framerate));

        let on_start = self.on_start.clone();
        let on_error = self.on_error.clone();
        let on_complete = self.on_complete.clone();

        self.running.store(true, Ordering::Release);

        (on_start)(wins_to_animate);
        self.animation_thread = Some(std::thread::spawn(move || {
            let mut is_running = running.load(Ordering::Acquire);

            let set_pos_flags = SWP_NOSENDCHANGING | SWP_NOACTIVATE | SWP_NOZORDER;
            let start_time = SystemTime::now();
            while is_running {
                let passed = start_time.elapsed().unwrap().as_millis();
                is_running = running.load(Ordering::Acquire) && passed <= duration;
                if !is_running {
                    break;
                }
                let complete_frac = (passed as f32 / duration as f32).clamp(0.0, 1.0);
                let res = Self::animate_frame(&wins_info, &animation, complete_frac, frame_duration, &set_pos_flags);
                if let Err(win) = res {
                    log::error!("Failed to animate window {:?}", win);
                    (on_error)();
                    return;
                }
            }

            Self::move_windows(&wins);
            running.store(false, Ordering::Release);
            (on_complete)();
        }));

        self.windows.clear();
    }

    pub fn cancel(&mut self) {
        if let Some(t) = self.animation_thread.take() {
            self.running.store(false, Ordering::Release);
            t.join().unwrap();
        }
    }

    fn animate_frame(
        wins: &[(WindowRef, Area, Area)],
        animation: &WindowAnimation,
        complete_frac: f32,
        frame_duration: Duration,
        set_pos_flags: &SET_WINDOW_POS_FLAGS,
    ) -> Result<(), WindowRef> {
        let frame_start = Instant::now();
        for (win, src_area, trg_area) in wins.iter() {
            if src_area == trg_area {
                continue;
            }

            let ((x1, y1), (w1, h1)) = (src_area.get_origin(), src_area.get_size());
            let ((x2, y2), (w2, h2)) = (trg_area.get_origin(), trg_area.get_size());
            let new_area = Area::new(
                animation.get_next_frame(x1 as f32, x2 as f32, complete_frac) as i32,
                animation.get_next_frame(y1 as f32, y2 as f32, complete_frac) as i32,
                animation.get_next_frame(w1 as f32, w2 as f32, complete_frac) as u16,
                animation.get_next_frame(h1 as f32, h2 as f32, complete_frac) as u16,
            );

            let (pos, size) = (new_area.get_origin(), new_area.get_size());
            if win.resize_and_move(pos, size, false, *set_pos_flags).is_err() {
                return Err(*win);
            }
        }

        if frame_start.elapsed() < frame_duration {
            let sleep_time = frame_duration.saturating_sub(frame_start.elapsed());
            std::thread::sleep(sleep_time);
        }

        Ok(())
    }

    fn move_windows(windows: &HashMap<WindowRef, WindowAnimationQueueInfo>) {
        let flags = SWP_SHOWWINDOW | SWP_NOSENDCHANGING | SWP_NOACTIVATE;
        windows
            .iter()
            .filter(|(w, i)| w.get_area().is_some_and(|a| a != i.target_area))
            .for_each(|(window, info)| {
                let (pos, size) = (info.target_area.get_origin(), info.target_area.get_size());
                let _ = window.resize_and_move(pos, size, true, flags);
                let _ = window.set_topmost(info.topmost);
                let _ = window.redraw();
            });
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WindowAnimation {
    Linear,
    EaseIn,
    EaseInSine,
    EaseInQuad,
    EaseInCubic,
    EaseInQuart,
    EaseInQuint,
    EaseInExpo,
    EaseInCirc,
    EaseInBack,
    EaseInElastic,
    EaseInBounce,
    EaseOut,
    EaseOutSine,
    EaseOutQuad,
    EaseOutCubic,
    EaseOutQuart,
    EaseOutQuint,
    EaseOutExpo,
    EaseOutCirc,
    EaseOutBack,
    EaseOutElastic,
    EaseOutBounce,
    EaseInOut,
    EaseInOutSine,
    EaseInOutQuad,
    EaseInOutCubic,
    EaseInOutQuart,
    EaseInOutQuint,
    EaseInOutExpo,
    EaseInOutCirc,
    EaseInOutBack,
    EaseInOutElastic,
    EaseInOutBounce,
}

impl Default for WindowAnimation {
    fn default() -> Self {
        Self::Linear
    }
}

impl WindowAnimation {
    fn get_next_frame(&self, src: f32, trg: f32, percent: f32) -> f32 {
        let t = percent;
        let t = match self {
            Self::Linear => t,
            Self::EaseIn | Self::EaseInQuad => t * t,
            Self::EaseInSine => 1.0 - ((t * PI) / 2.0).cos(),
            Self::EaseInCubic => t * t * t,
            Self::EaseInQuart => t * t * t * t,
            Self::EaseInQuint => t * t * t * t * t,
            Self::EaseInExpo => match t == 0.0 {
                true => 0.0,
                false => 2.0f32.powf(10.0 * t - 10.0),
            },
            Self::EaseInCirc => 1.0 - (1.0 - (t * t)).sqrt(),
            Self::EaseInBack => Self::ease_in_back(t),
            Self::EaseInElastic => Self::ease_in_elastic(t),
            Self::EaseInBounce => 1.0 - Self::ease_out_bounce(1.0 - t),
            Self::EaseOut | Self::EaseOutQuad => 1.0 - (1.0 - t).powi(2),
            Self::EaseOutSine => ((t * PI) / 2.0).sin(),
            Self::EaseOutCubic => 1.0 - (1.0 - t).powi(3),
            Self::EaseOutQuart => 1.0 - (1.0 - t).powi(4),
            Self::EaseOutQuint => 1.0 - (1.0 - t).powi(5),
            Self::EaseOutExpo => match t == 1.0 {
                true => 1.0,
                false => 1.0 - 2.0f32.powf(-10.0 * t),
            },
            Self::EaseOutCirc => (1.0 - (t - 1.0).powf(2.0)).sqrt(),
            Self::EaseOutBack => Self::ease_out_back(t),
            Self::EaseOutElastic => Self::ease_out_elastic(t),
            Self::EaseOutBounce => Self::ease_out_bounce(t),
            Self::EaseInOut | Self::EaseInOutQuad => Self::ease_in_out_pow(t, 2.0),
            Self::EaseInOutSine => -((t * PI).cos() - 1.0) / 2.0,
            Self::EaseInOutCubic => Self::ease_in_out_pow(t, 3.0),
            Self::EaseInOutQuart => Self::ease_in_out_pow(t, 4.0),
            Self::EaseInOutQuint => Self::ease_in_out_pow(t, 5.0),
            Self::EaseInOutExpo => match t == 0.0 || t == 1.0 {
                true => t,
                false => match t < 0.5 {
                    true => 2.0f32.powf(20.0 * t - 10.0) / 2.0,
                    false => (2.0 - 2.0f32.powf(-20.0 * t + 10.0)) / 2.0,
                },
            },
            Self::EaseInOutCirc => match t < 0.5 {
                true => (1.0 - (1.0 - (2.0 * t).powf(2.0)).sqrt()) / 2.0,
                false => (1.0 + (1.0 - (-2.0 * t + 2.0).powf(2.0)).sqrt()) / 2.0,
            },
            Self::EaseInOutBack => Self::ease_in_out_back(t),
            Self::EaseInOutElastic => Self::ease_in_out_elastic(t),
            Self::EaseInOutBounce => match t < 0.5 {
                true => (1.0 - Self::ease_out_bounce(1.0 - 2.0 * t)) / 2.0,
                false => (1.0 + Self::ease_out_bounce(2.0 * t - 1.0)) / 2.0,
            },
        };
        Self::lerp(src, trg, t)
    }

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    fn ease_in_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C3: f32 = C1 + 1.0;
        C3 * t.powi(3) - C1 * t.powi(2)
    }

    fn ease_in_elastic(t: f32) -> f32 {
        const C4: f32 = (2.0 * PI) / 3.0;
        if t == 0.0 || t == 1.0 {
            return t;
        }
        -(2.0f32.powf(10.0 * t - 10.0)) * ((10.0 * t - 10.75) * C4).sin()
    }

    fn ease_out_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C3: f32 = C1 + 1.0;
        1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
    }

    fn ease_out_elastic(t: f32) -> f32 {
        const C4: f32 = (2.0 * PI) / 3.0;
        if t == 0.0 || t == 1.0 {
            return t;
        }
        2.0f32.powf(-10.0 * t) * ((10.0 * t - 0.75) * C4).sin() + 1.0
    }

    fn ease_out_bounce(t: f32) -> f32 {
        const N1: f32 = 7.5625;
        const D1: f32 = 2.75;

        if t < 1.0 / D1 {
            N1 * t * t
        } else if t < 2.0 / D1 {
            let x = t - 1.5 / D1;
            N1 * x * x + 0.75
        } else if t < 2.5 / D1 {
            let x = t - 2.25 / D1;
            N1 * x * x + 0.9375
        } else {
            let x = t - 2.625 / D1;
            N1 * x * x + 0.984375
        }
    }

    fn ease_in_out_pow(t: f32, p: f32) -> f32 {
        match t < 0.5 {
            true => 2.0f32.powf(p - 1.0) * t.powf(p),
            false => 1.0 - ((-2.0 * t + 2.0).powf(p) / 2.0),
        }
    }

    fn ease_in_out_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C2: f32 = C1 * 1.525;

        match t < 0.5 {
            true => ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0,
            false => ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (2.0 * t - 2.0) + C2) + 2.0) / 2.0,
        }
    }

    fn ease_in_out_elastic(t: f32) -> f32 {
        const C5: f32 = (2.0 * PI) / 4.5;
        if t == 0.0 || t == 1.0 {
            return t;
        }
        match t < 0.5 {
            true => -(2.0f32.powf(20.0 * t - 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0,
            false => ((2.0f32.powf(-20.0 * t + 10.0) * ((20.0 * t - 11.125) * C5).sin()) / 2.0) + 1.0,
        }
    }
}
