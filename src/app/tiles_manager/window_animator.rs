use std::collections::HashMap;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::SystemTime;

use serde::Deserialize;
use windows::Win32::Foundation::HWND;

use crate::app::structs::area::Area;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;

pub struct WindowAnimator {
    windows: Vec<(WindowRef, Area)>,
    running: Arc<AtomicBool>,
    animation_thread: Option<std::thread::JoinHandle<()>>,
    animation_duration: Duration,
    framerate: u8,
    previous_foreground: Option<HWND>,
}

impl WindowAnimator {
    pub fn new(animation_duration: Duration, framerate: u8) -> Self {
        assert!(animation_duration.as_millis() > 0);
        assert!(framerate > 0);
        WindowAnimator {
            windows: vec![],
            running: Arc::new(AtomicBool::new(false)),
            animation_thread: None,
            animation_duration,
            framerate,
            previous_foreground: None,
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

    pub fn start(&mut self, animation: Option<(WindowAnimation, WindowAnimation)>) {
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
        self.animation_thread = Some(std::thread::spawn(move || {
            let mut is_running = true;
            let src_areas: HashMap<isize, Option<Area>> = wins
                .iter()
                .filter_map(|(w, _)| {
                    w.restore();
                    w.get_window_box().map(|a| (w.hwnd.0, Some(a)))
                })
                .collect();
            running.store(true, Ordering::SeqCst);
            let start_time = SystemTime::now();
            while is_running {
                let passed = start_time.elapsed().unwrap().as_millis();
                let frame_end = passed + frame_duration.as_millis();
                for (win, target_area) in wins
                    .iter()
                    .filter(|(w, t)| src_areas.get(&w.hwnd.0).is_some_and(|a| a.is_some_and(|a| a != *t)))
                {
                    is_running = running.load(Ordering::Relaxed) && passed <= duration;
                    if !is_running {
                        break;
                    }
                    if let Some(Some(src_area)) = src_areas.get(&win.hwnd.0) {
                        if src_area == target_area {
                            continue;
                        }
                        let ((x1, y1), (w1, h1)) = (src_area.get_origin(), src_area.get_size());
                        let ((x2, y2), (w2, h2)) = (target_area.get_origin(), target_area.get_size());
                        let percent = (passed as f32 / duration as f32).clamp(0.0, 100.0);
                        let new_area = Area::new(
                            animation.0.get_next_frame(x1 as f32, x2 as f32, percent) as i32,
                            animation.0.get_next_frame(y1 as f32, y2 as f32, percent) as i32,
                            animation.1.get_next_frame(w1 as f32, w2 as f32, percent) as u16,
                            animation.1.get_next_frame(h1 as f32, h2 as f32, percent) as u16,
                        );

                        let _ = win
                            .resize_and_move(new_area.get_origin(), new_area.get_size(), false, false)
                            .inspect_err(|_| log::error!("Failed to resize and move window {:?}", win.hwnd));
                    }
                }
                is_running = running.load(Ordering::Relaxed) && passed <= duration;
                let animation_time = start_time.elapsed().unwrap().as_millis();
                if animation_time < frame_end {
                    std::thread::sleep(Duration::from_millis(frame_end.saturating_sub(animation_time) as u64));
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

    fn move_windows(windows: &[(WindowRef, Area)]) {
        windows.iter().for_each(|(window, target_area)| {
            let _ = window.resize_and_move(target_area.get_origin(), target_area.get_size(), false, true);
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
    EaseInCubic,
    EaseInQuad,
    EaseInQuint,
    EaseOut,
    EaseOutCubic,
    EaseOutQuad,
    EaseOutQuint,
    EaseOutBack,
    EaseOutBounce,
    EaseOutElastic,
    EaseInOut,
    EaseInOutBack,
}

impl Default for WindowAnimation {
    fn default() -> Self {
        Self::Linear
    }
}

impl WindowAnimation {
    fn get_next_frame(&self, src: f32, trg: f32, percent: f32) -> f32 {
        let t = percent;
        match self {
            Self::Linear => Self::lerp(src, trg, t),
            Self::EaseIn => Self::lerp(src, trg, t * t),
            Self::EaseInCubic => Self::lerp(src, trg, t * t * t),
            Self::EaseInQuad => Self::lerp(src, trg, t * t * t * t),
            Self::EaseInQuint => Self::lerp(src, trg, t * t * t * t * t),
            Self::EaseOut => Self::lerp(src, trg, 1.0 - (1.0 - t).powi(2)),
            Self::EaseOutCubic => Self::lerp(src, trg, 1.0 - (1.0 - t).powi(3)),
            Self::EaseOutQuad => Self::lerp(src, trg, 1.0 - (1.0 - t).powi(2)),
            Self::EaseOutQuint => Self::lerp(src, trg, 1.0 - (1.0 - t).powi(5)),
            Self::EaseOutBack => Self::lerp(src, trg, Self::ease_out_back(t)),
            Self::EaseOutBounce => Self::lerp(src, trg, Self::ease_out_bounce(t)),
            Self::EaseOutElastic => Self::lerp(src, trg, Self::ease_out_elastic(t)),
            Self::EaseInOut => Self::lerp(src, trg, t * t * (3.0 - 2.0 * t)),
            Self::EaseInOutBack => Self::lerp(src, trg, Self::ease_in_out_back(t)),
        }
    }

    fn lerp(a: f32, b: f32, t: f32) -> f32 {
        a + (b - a) * t
    }

    fn ease_out_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C3: f32 = C1 + 1.0;
        1.0 + C3 * (t - 1.0).powi(3) + C1 * (t - 1.0).powi(2)
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

    fn ease_out_elastic(t: f32) -> f32 {
        const C4: f32 = (2.0 * std::f32::consts::PI) / 3.0;
        if t == 0.0 || t == 1.0 {
            return t;
        }
        2.0f32.powf(-10.0 * t) * (t * 10.0 - 0.75).sin() * C4 + 1.0
    }

    fn ease_in_out_back(t: f32) -> f32 {
        const C1: f32 = 1.70158;
        const C2: f32 = C1 * 1.525;

        if t < 0.5 {
            ((2.0 * t).powi(2) * ((C2 + 1.0) * 2.0 * t - C2)) / 2.0
        } else {
            ((2.0 * t - 2.0).powi(2) * ((C2 + 1.0) * (2.0 * t - 2.0) + C2) + 2.0) / 2.0
        }
    }
}
