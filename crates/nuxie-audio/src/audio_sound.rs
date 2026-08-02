/// Retained control handle for one scheduled sound.
#[derive(Debug, Clone)]
pub struct AudioSound {
    state: Arc<Mutex<SoundState>>,
    engine: Weak<EngineShared>,
}

impl AudioSound {
    pub fn seek(&self, time_in_frames: u64) -> bool {
        let mut state = lock(&self.state);
        if state.disposed || time_in_frames > state.frame_count() {
            return false;
        }
        state.cursor = time_in_frames;
        state.completed = false;
        true
    }

    pub fn seek_seconds(&self, time_in_seconds: f32) -> bool {
        let Some(engine) = self.engine.upgrade() else {
            return false;
        };
        if !time_in_seconds.is_finite() || time_in_seconds < 0.0 {
            return false;
        }
        self.seek((time_in_seconds * engine.sample_rate as f32).round() as u64)
    }

    pub fn time_in_frames(&self) -> u64 {
        let state = lock(&self.state);
        if state.disposed {
            0
        } else {
            state.reported_cursor
        }
    }

    pub fn time_in_seconds(&self) -> f32 {
        let Some(engine) = self.engine.upgrade() else {
            return 0.0;
        };
        self.time_in_frames() as f32 / engine.sample_rate as f32
    }

    pub fn stop(&self, fade_time_in_frames: u64) {
        let mut state = lock(&self.state);
        if state.disposed {
            return;
        }
        if fade_time_in_frames == 0 {
            state.playing = false;
            state.fade = None;
        } else {
            state.playing = true;
            state.fade = Some(FadeState {
                remaining: fade_time_in_frames,
                total: fade_time_in_frames,
            });
        }
    }

    pub fn play(&self) {
        let Some(engine) = self.engine.upgrade() else {
            return;
        };
        let current_frame = lock(&engine.state).frame_clock;
        let mut state = lock(&self.state);
        if state.disposed {
            return;
        }
        // Pinned AudioSound::play only seeks/starts its miniaudio node. It does
        // not relink a sound already removed by completion or artboard stop.
        state.cursor = 0;
        state.reported_cursor = 0;
        state.scheduled_start = current_frame;
        state.playing = true;
        state.completed = false;
        state.fade = None;
    }

    pub fn pause(&self) {
        self.stop(0);
    }

    pub fn resume(&self) {
        let mut state = lock(&self.state);
        if !state.disposed {
            state.playing = true;
            state.fade = None;
        }
    }

    pub fn volume(&self) -> f32 {
        lock(&self.state).volume
    }

    pub fn set_volume(&self, value: f32) {
        lock(&self.state).volume = value;
    }

    pub fn completed(&self) -> bool {
        let state = lock(&self.state);
        state.disposed || state.completed
    }
}
