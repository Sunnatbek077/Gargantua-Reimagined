// =============================================================================
// crates/gargantua-camera/src/path/recorder.rs
// =============================================================================
//
// PURPOSE:
//   Records live camera poses as Keyframes during interactive use, building
//   a CameraSpline from the user's free-flight or orbit camera movement.
//   The user presses a key to start recording; the recorder samples the
//   camera pose at regular intervals; pressing the key again stops recording
//   and creates a spline for playback or export.
//
//   Also supports manual keyframe insertion: the user positions the camera
//   and presses "add keyframe" to record a single pose at the current time.
//
// SIZE: ~180 lines
//
// DEPENDENCIES:
//   Internal:
//     - crate::path::keyframe::Keyframe
//     - crate::path::spline::CameraSpline
//     - crate::world_camera::WorldCamera
//     - crate::errors::CameraError
//   External:
//     - glam::{Vec3, Quat}
//
// CALLED BY:
//   - crates/gargantua-ui/src/panel::path_editor.rs
//       — calls PathRecorder::start_recording() / stop_recording()
//   - crates/gargantua-core/src/app.rs
//       — calls PathRecorder::tick() each frame when recording
//
// PUBLIC TYPES:
//
//   #[derive(Debug, Clone, Copy, PartialEq, Eq)]
//   pub enum RecordingState {
//     Idle,        // not recording
//     Recording,   // actively sampling camera poses
//     Paused,      // recording paused (keeps timestamps continuous)
//     Finished,    // recording complete; spline available
//   }
//
//   pub struct PathRecorder {
//     state:         RecordingState,
//     recorded:      Vec<Keyframe>,      // captured keyframes
//     start_time:    f32,                // app time when recording began
//     current_time:  f32,                // current recording time
//     sample_rate:   f32,                // keyframes per second (default 10.0)
//     last_sample_t: f32,                // time of last sampled keyframe
//     min_move_dist: f32,                // minimum position change to record (default 0.1M)
//   }
//
// PUBLIC FUNCTIONS:
//
//   pub fn new(sample_rate: f32) -> Self
//     — creates in Idle state.
//     — sample_rate: how many keyframes per second to capture (default 10.0).
//     — min_move_dist: only records if camera moved > this distance since last sample.
//
//   pub fn start_recording(&mut self, app_time: f32)
//     — sets state = Recording.
//     — clears recorded keyframes.
//     — sets start_time = app_time.
//
//   pub fn stop_recording(&mut self) -> Result<CameraSpline, CameraError>
//     — sets state = Finished.
//     — calls CameraSpline::new(self.recorded.clone(), false).
//     — returns the built spline or CameraError::InsufficientKeyframes if < 2 captured.
//
//   pub fn pause(&mut self)   { self.state = RecordingState::Paused;    }
//   pub fn resume(&mut self)  { self.state = RecordingState::Recording;  }
//
//   pub fn tick(
//     &mut self,
//     app_time: f32,
//     camera:   &WorldCamera,
//   )
//     — if state != Recording: no-op.
//     — current_time = app_time - start_time.
//     — if current_time - last_sample_t >= 1.0/sample_rate:
//         check if camera moved > min_move_dist from last recorded position.
//         if yes: record a new Keyframe from camera.position() + camera.rotation().
//         last_sample_t = current_time.
//
//   pub fn add_manual_keyframe(
//     &mut self,
//     time:   f32,
//     camera: &WorldCamera,
//   )
//     — records a single keyframe at the given time regardless of sample_rate.
//     — used for manual "mark this pose" recording.
//     — if state == Idle: automatically starts recording.
//
//   pub fn state(&self) -> RecordingState { self.state }
//   pub fn keyframe_count(&self) -> usize { self.recorded.len() }
//   pub fn recording_duration(&self) -> f32 { self.current_time }
//
// NOTES FOR AI:
//   - sample_rate of 10 fps is sufficient for smooth spline interpolation.
//     Higher rates produce more keyframes but the spline interpolates them
//     anyway — 10fps keyframes with smooth Catmull-Rom gives 60fps output.
//   - min_move_dist prevents recording duplicate keyframes when the camera
//     is stationary. Default 0.1M in geometric units.
//   - The built CameraSpline (from stop_recording) is returned to the UI
//     (path_editor.rs) which stores it and passes it to CinematicMode.
//   - Recording does not affect rendering — it only reads camera state.
// =============================================================================

use glam::{Quat, Vec3};
use crate::{
    errors::CameraError,
    path::{keyframe::Keyframe, spline::CameraSpline},
    world_camera::WorldCamera,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
    Finished,
}

pub struct PathRecorder {
    state:         RecordingState,
    recorded:      Vec<Keyframe>,
    start_time:    f32,
    current_time:  f32,
    sample_rate:   f32,
    last_sample_t: f32,
    min_move_dist: f32,
}

impl PathRecorder {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            state:         RecordingState::Idle,
            recorded:      Vec::new(),
            start_time:    0.0,
            current_time:  0.0,
            sample_rate:   sample_rate.max(1.0),
            last_sample_t: f32::NEG_INFINITY,
            min_move_dist: 0.1,
        }
    }

    pub fn start_recording(&mut self, app_time: f32) {
        self.state         = RecordingState::Recording;
        self.recorded.clear();
        self.start_time    = app_time;
        self.current_time  = 0.0;
        self.last_sample_t = f32::NEG_INFINITY;
    }

    pub fn stop_recording(&mut self) -> Result<CameraSpline, CameraError> {
        self.state = RecordingState::Finished;
        CameraSpline::new(self.recorded.clone(), false)
    }

    pub fn pause(&mut self)  { if self.state == RecordingState::Recording { self.state = RecordingState::Paused; } }
    pub fn resume(&mut self) { if self.state == RecordingState::Paused    { self.state = RecordingState::Recording; } }

    pub fn tick(&mut self, app_time: f32, camera: &WorldCamera) {
        if self.state != RecordingState::Recording { return; }
        self.current_time = app_time - self.start_time;
        let interval = 1.0 / self.sample_rate;
        if self.current_time - self.last_sample_t >= interval {
            let pos = camera.position();
            // Check movement threshold
            let should_record = self.recorded.last()
                .map(|kf| (kf.position - pos).length() >= self.min_move_dist)
                .unwrap_or(true);
            if should_record {
                let kf = Keyframe::new(self.current_time, pos, camera.rotation());
                self.recorded.push(kf);
            }
            self.last_sample_t = self.current_time;
        }
    }

    pub fn add_manual_keyframe(&mut self, time: f32, camera: &WorldCamera) {
        if self.state == RecordingState::Idle {
            self.start_recording(time);
        }
        let kf = Keyframe::new(time - self.start_time, camera.position(), camera.rotation());
        self.recorded.push(kf);
    }

    pub fn state(&self)             -> RecordingState { self.state           }
    pub fn keyframe_count(&self)    -> usize           { self.recorded.len()  }
    pub fn recording_duration(&self)-> f32             { self.current_time   }
}