use crate::app::structs::area::Area;
use crate::win32::api::monitor::enum_display_monitors;
use crate::win32::api::window::begin_defer_window_pos;
use crate::win32::api::window::end_defer_window_pos;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use serde::Deserialize;
use std::f32::consts::PI;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use windows::Win32::Foundation::HWND;

pub struct WindowAnimationPlayer {
    windows: Vec<(WindowRef, Area)>,
    running: Arc<AtomicBool>,
    animation_thread: Option<std::thread::JoinHandle<()>>,
    animation_duration: Duration,
    framerate: u8,
    previous_foreground: Option<HWND>,
    on_error: Arc<dyn Fn() + Send + Sync + 'static>,
}

impl WindowAnimationPlayer {
    pub fn new<E: Fn() + Send + Sync + 'static>(animation_duration: Duration, framerate: u8, on_error: E) -> Self {
        assert!(animation_duration.as_millis() > 0);
        assert!(framerate > 0);
        WindowAnimationPlayer {
            windows: vec![],
            running: Arc::new(AtomicBool::new(false)),
            animation_thread: None,
            animation_duration,
            framerate,
            previous_foreground: None,
            on_error: Arc::new(on_error),
        }
    }

    pub fn queue(&mut self, window: WindowRef, new_area: Area) {
        if self.running.load(Ordering::Relaxed) {
            self.clear();
        }
        self.windows.push((window, new_area));
    }

    pub fn clear(&mut self) {
        self.running.store(false, std::sync::atomic::Ordering::SeqCst);
        self.windows.clear();
        if let Some(t) = self.animation_thread.take() {
            t.join().unwrap();
        }

        if let Some(hwnd) = self.previous_foreground.take() {
            WindowRef::new(hwnd).focus();
        }
    }

    pub fn play(&mut self, animation: Option<WindowAnimation>) {
        self.previous_foreground = get_foreground_window();
        if animation.is_none() {
            Self::move_windows(&self.windows);
            self.clear();
            return;
        }

        self.cancel();
        let animation = animation.unwrap().clone();
        let wins = self.windows.clone();
        let running = self.running.clone();
        let duration = self.animation_duration.as_millis();
        let framerate = self.framerate;
        let frame_duration = Duration::from_millis(1000 / u64::from(framerate));
        let on_error = self.on_error.clone();
        self.animation_thread = Some(std::thread::spawn(move || {
            running.store(true, Ordering::SeqCst);
            let mut is_running = true;
            let monitors: Vec<Area> = enum_display_monitors().into_iter().map(Area::from).collect();

            let fwins: Vec<(WindowRef, Area, Area, bool)> = wins
                .clone()
                .into_iter()
                .filter_map(|(w, t)| {
                    let m = &monitors.iter().find(|m| m.contains(t.get_center()));
                    let src_area: Area = w.get_window_box()?;
                    let same_monitor = m.is_some_and(|m| m.contains(src_area.get_center()));
                    w.restore();
                    match src_area == t {
                        true => None,
                        false => Some((w, src_area, t, same_monitor)),
                    }
                })
                .collect();

            let start_time = SystemTime::now();
            while is_running {
                let passed = start_time.elapsed().unwrap().as_millis();
                is_running = running.load(Ordering::Relaxed) && passed <= duration;
                if !is_running {
                    break;
                }
                let complete_frac = (passed as f32 / duration as f32).clamp(0.0, 1.0);
                let res = Self::animate_frame(&fwins, &animation, complete_frac, frame_duration);
                if let Err(win) = res {
                    log::error!("Failed to animate window {:?}", win);
                    (on_error)();
                    return;
                }
            }

            Self::move_windows(&wins);
            running.store(false, std::sync::atomic::Ordering::SeqCst);
        }));

        self.windows.clear();
    }

    pub fn cancel(&mut self) {
        if let Some(t) = self.animation_thread.take() {
            self.running.store(false, std::sync::atomic::Ordering::SeqCst);
            t.join().unwrap();
        }
    }

    fn animate_frame(
        wins: &[(WindowRef, Area, Area, bool)],
        animation: &WindowAnimation,
        complete_frac: f32,
        frame_duration: Duration,
    ) -> Result<(), WindowRef> {
        let mut hdwp = begin_defer_window_pos(wins.len() as i32).unwrap();
        let frame_start = Instant::now();
        for (win, src_area, trg_area, same_monitor) in wins.iter() {
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

            let pos = new_area.get_origin();

            // NOTE: animating the windows sizes is very expensive, so it's done only when
            // one of the following is true:
            //      1. no move operation (i.e. the same origin)
            //      2. the destination monitor is the same as the origin and the window is getting bigger
            // The second condition avoids intermonitor flickering
            let size = match src_area.get_origin() == trg_area.get_origin() {
                true => new_area.get_size(),
                false => match same_monitor {
                    true if src_area.get_size() < trg_area.get_size() => new_area.get_size(),
                    _ => trg_area.get_size(),
                },
            };

            match win.defer_resize_and_move(hdwp, pos, size, false, false) {
                Ok(v) => {
                    hdwp = v;
                }
                Err(_) => return Err(*win),
            }
        }

        end_defer_window_pos(hdwp);
        if frame_start.elapsed() < frame_duration {
            let sleep_time = frame_duration.saturating_sub(frame_start.elapsed());
            std::thread::sleep(sleep_time);
        }

        Ok(())
    }

    fn move_windows(windows: &[(WindowRef, Area)]) {
        windows.iter().for_each(|(window, target_area)| {
            let _ = window.resize_and_move(target_area.get_origin(), target_area.get_size(), true, false, true);
            let _ = window.redraw();
        });
        if let Some(hwnd) = get_foreground_window() {
            WindowRef::new(hwnd).focus();
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
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
                    false => 2.0 - 2.0f32.powf(-20.0 * t + 10.0) / 2.0,
                },
            },
            Self::EaseInOutCirc => match t < 0.5 {
                true => (1.0 - (1.0 - (2.0 * t).powf(2.0)).sqrt()) / 2.0,
                false => ((1.0 + (-2.0 * t - 2.0).powf(2.0)).sqrt() + 1.0) / 2.0,
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
        C3 * t.powi(3) + C1 * t.powi(2)
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
