use crate::app::structs::area::Area;
use crate::win32::api::window::begin_defer_window_pos;
use crate::win32::api::window::end_defer_window_pos;
use crate::win32::api::window::get_foreground_window;
use crate::win32::window::window_obj::WindowObjHandler;
use crate::win32::window::window_obj::WindowObjInfo;
use crate::win32::window::window_ref::WindowRef;
use serde::Deserialize;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::time::SystemTime;
use windows::Win32::Foundation::HWND;

pub struct WindowAnimator {
    windows: Vec<(WindowRef, Area)>,
    running: Arc<AtomicBool>,
    animation_thread: Option<std::thread::JoinHandle<()>>,
    animation_duration: Duration,
    framerate: u8,
    previous_foreground: Option<HWND>,
    on_error: Arc<dyn Fn() + Send + Sync + 'static>,
}

impl WindowAnimator {
    pub fn new<E: Fn() + Send + Sync + 'static>(animation_duration: Duration, framerate: u8, on_error: E) -> Self {
        assert!(animation_duration.as_millis() > 0);
        assert!(framerate > 0);
        WindowAnimator {
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

    pub fn start(&mut self, animation: Option<WindowAnimation>) {
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
            let fwins: Vec<(WindowRef, Area, Area)> = wins
                .clone()
                .into_iter()
                .filter_map(|(w, t)| {
                    let src_area: Area = w.get_window_box()?;
                    w.restore();
                    match src_area == t {
                        true => None,
                        false => Some((w, src_area, t)),
                    }
                })
                .collect();
            let start_time = SystemTime::now();
            while is_running {
                let passed = start_time.elapsed().unwrap().as_millis();
                let mut hdwp = begin_defer_window_pos(fwins.len() as i32).unwrap();
                is_running = running.load(Ordering::Relaxed) && passed <= duration;
                if !is_running {
                    break;
                }
                let frame_start = Instant::now();
                for (win, src_area, target_area) in fwins.iter() {
                    if src_area == target_area {
                        continue;
                    }
                    let ((x1, y1), (w1, h1)) = (src_area.get_origin(), src_area.get_size());
                    let ((x2, y2), (w2, h2)) = (target_area.get_origin(), target_area.get_size());
                    let percent = (passed as f32 / duration as f32).clamp(0.0, 100.0);
                    let new_area = Area::new(
                        animation.get_next_frame(x1 as f32, x2 as f32, percent) as i32,
                        animation.get_next_frame(y1 as f32, y2 as f32, percent) as i32,
                        animation.get_next_frame(w1 as f32, w2 as f32, percent) as u16,
                        animation.get_next_frame(h1 as f32, h2 as f32, percent) as u16,
                    );

                    let (pos, size) = match src_area.get_origin() == target_area.get_origin() {
                        true => (target_area.get_origin(), new_area.get_size()),
                        false => (new_area.get_origin(), target_area.get_size()),
                    };

                    if let Ok(v) = win.defer_resize_and_move(hdwp, pos, size, false, false) {
                        hdwp = v;
                    } else {
                        log::error!("Failed to move window");
                        (on_error)();
                        return;
                    }
                }

                end_defer_window_pos(hdwp);
                if frame_start.elapsed() < frame_duration {
                    let sleep_time = frame_duration.saturating_sub(frame_start.elapsed());
                    std::thread::sleep(sleep_time);
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
    EaseInQuad,
    EaseInCubic,
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
