// =============================================================================
// crates/gargantua-core/src/clock.rs
// =============================================================================
//
// PURPOSE:
//   High-resolution frame clock. Tracks wall-clock time, frame delta (dt),
//   frame index, and elapsed time since app start. Provides the time
//   values consumed by WGSL shaders (via scene uniforms) for:
//     - TAA jitter sequence (frame_idx % 8 determines Halton sample)
//     - Film grain animation (time-varying noise offset in film_grain.wgsl)
//     - Accretion disk rotation (time-based angular velocity in accretion_disk.wgsl)
//     - Camera animation playback (elapsed_s drives spline interpolation)
//
//   Platform-specific:
//     - Native: std::time::Instant (monotonic, ~1ns resolution)
//     - WASM:   web_sys::window().performance().now() (f64 milliseconds)
//
// SIZE: ~120 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::errors::CoreError
//   External (native):
//     - std::time::{Instant, Duration}
//   External (WASM):
//     - web_sys::window (performance.now())
//     - wasm_bindgen::prelude::*
//
// CALLED BY:
//   - crate::app::App::render_frame()   — calls clock.tick() each frame
//   - crates/gargantua-render/src/pipelines/ray_march.rs
//       — reads clock.elapsed_s() for disk rotation uniform
//   - crates/gargantua-render/src/postfx/taa.rs
//       — reads clock.frame_idx() for jitter sequence index
//   - crates/gargantua-render/src/postfx/film_grain.rs
//       — reads clock.elapsed_s() for noise animation offset
//   - crates/gargantua-camera/src/spline.rs
//       — reads clock.elapsed_s() for camera path playback
//
// PUBLIC TYPES:
//
//   pub struct Clock {
//     #[cfg(not(target_arch = "wasm32"))]
//     start:      std::time::Instant,    // app start time
//     #[cfg(not(target_arch = "wasm32"))]
//     last_tick:  std::time::Instant,    // time of previous tick()
//     #[cfg(target_arch = "wasm32")]
//     start_ms:   f64,                   // performance.now() at start
//     #[cfg(target_arch = "wasm32")]
//     last_ms:    f64,                   // performance.now() at last tick
//     frame_idx:  u64,                   // monotonically increasing frame counter
//     delta_t:    f32,                   // seconds since last tick (capped at 0.1s)
//     elapsed_s:  f32,                   // total seconds since app start
//     fps:        f32,                   // smoothed frames per second
//     fps_history: [f32; 16],            // circular buffer of last 16 frame times
//     fps_cursor:  usize,
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new() -> Self
//     — records the start time.
//     — initializes frame_idx = 0, delta_t = 0.0, elapsed_s = 0.0.
//     — fps_history filled with 1.0/60.0 (assumes 60fps before first tick).
//
//   pub fn tick(&mut self)
//     — records current time, computes delta from last_tick.
//     — delta_t = (now - last_tick).as_secs_f32()
//     — caps delta_t at 0.1 seconds (prevents spiral of death on tab switch,
//       breakpoint resume, or system sleep/wake).
//     — elapsed_s += delta_t
//     — frame_idx += 1
//     — updates fps_history[fps_cursor % 16] = delta_t
//     — fps_cursor += 1
//     — fps = 1.0 / (fps_history.iter().sum::<f32>() / 16.0)  (smoothed FPS)
//     — updates last_tick = now
//     — WASM: uses performance.now() converted from ms to seconds.
//
//   pub fn delta_t(&self)   -> f32  { self.delta_t   }
//     — seconds since last frame. Use for physics integration and animation.
//     — capped at 0.1s. Typical value: 0.0167s (60fps) or 0.0083s (120fps).
//
//   pub fn elapsed_s(&self) -> f32  { self.elapsed_s }
//     — total seconds since app start (monotonically increasing).
//     — used for accretion disk rotation: angle = elapsed_s * angular_velocity.
//
//   pub fn frame_idx(&self) -> u64  { self.frame_idx }
//     — monotonically increasing frame counter starting at 1 after first tick.
//     — used by TAA: jitter_idx = (frame_idx % 8) as u32
//       (8-sample Halton sequence in taa.wgsl).
//     — also used by film_grain.wgsl for per-frame noise variation.
//
//   pub fn fps(&self) -> f32  { self.fps }
//     — smoothed FPS (16-frame rolling average).
//     — displayed in the stats bar overlay.
//
//   pub fn frame_time_ms(&self) -> f32
//     — returns self.delta_t * 1000.0 (milliseconds).
//     — displayed alongside FPS in the stats bar.
//
// NOTES FOR AI:
//   - delta_t cap at 0.1s is critical. Without it, returning from a
//     breakpoint could produce delta_t of 10+ seconds, causing physics
//     and animation systems to produce NaN or extreme values.
//   - On WASM, std::time::Instant is not available. Use performance.now():
//       let now_ms: f64 = web_sys::window()
//           .unwrap().performance().unwrap().now();
//       let delta_ms = now_ms - self.last_ms;
//       self.delta_t = (delta_ms / 1000.0) as f32;
//   - frame_idx starts at 0 and is incremented in tick(). After the first
//     tick it is 1. Shaders receive frame_idx as a u32 uniform — cast safely
//     since u64 → u32 wrap is acceptable for jitter/grain purposes.
//   - fps_history uses a fixed-size array (not VecDeque) to avoid allocation
//     on the hot path. 16 samples ≈ 267ms smoothing window at 60fps.
// =============================================================================

pub struct Clock {
    #[cfg(not(target_arch = "wasm32"))]
    start:      std::time::Instant,
    #[cfg(not(target_arch = "wasm32"))]
    last_tick:  std::time::Instant,
    #[cfg(target_arch = "wasm32")]
    start_ms:   f64,
    #[cfg(target_arch = "wasm32")]
    last_ms:    f64,
    frame_idx:  u64,
    delta_t:    f32,
    elapsed_s:  f32,
    fps:        f32,
    fps_history: [f32; 16],
    fps_cursor:  usize,
}

impl Clock {
    pub fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let now = std::time::Instant::now();
        #[cfg(target_arch = "wasm32")]
        let now_ms = web_sys::window()
            .expect("no window")
            .performance()
            .expect("no performance")
            .now();

        Self {
            #[cfg(not(target_arch = "wasm32"))]
            start:      now,
            #[cfg(not(target_arch = "wasm32"))]
            last_tick:  now,
            #[cfg(target_arch = "wasm32")]
            start_ms:   now_ms,
            #[cfg(target_arch = "wasm32")]
            last_ms:    now_ms,
            frame_idx:  0,
            delta_t:    0.0,
            elapsed_s:  0.0,
            fps:        60.0,
            fps_history: [1.0 / 60.0; 16],
            fps_cursor:  0,
        }
    }

    pub fn tick(&mut self) {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let now   = std::time::Instant::now();
            let raw   = (now - self.last_tick).as_secs_f32();
            self.delta_t  = raw.min(0.1);
            self.last_tick = now;
        }
        #[cfg(target_arch = "wasm32")]
        {
            let now_ms = web_sys::window()
                .unwrap().performance().unwrap().now();
            let raw = ((now_ms - self.last_ms) / 1000.0) as f32;
            self.delta_t = raw.min(0.1);
            self.last_ms = now_ms;
        }

        self.elapsed_s += self.delta_t;
        self.frame_idx += 1;

        self.fps_history[self.fps_cursor % 16] = self.delta_t;
        self.fps_cursor += 1;
        let avg = self.fps_history.iter().sum::<f32>() / 16.0;
        self.fps = if avg > 0.0 { 1.0 / avg } else { 0.0 };
    }

    pub fn delta_t(&self)      -> f32 { self.delta_t  }
    pub fn elapsed_s(&self)    -> f32 { self.elapsed_s }
    pub fn frame_idx(&self)    -> u64 { self.frame_idx }
    pub fn fps(&self)          -> f32 { self.fps       }
    pub fn frame_time_ms(&self)-> f32 { self.delta_t * 1000.0 }
}

impl Default for Clock {
    fn default() -> Self { Self::new() }
}