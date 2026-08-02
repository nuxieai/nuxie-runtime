use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::AudioSource;

/// Stable identity used to tag sounds for artboard-scoped teardown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioArtboardId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioEngineError;

impl std::fmt::Display for AudioEngineError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("audio engine channels and sample rate must be non-zero")
    }
}

impl std::error::Error for AudioEngineError {}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Debug)]
struct SoundState {
    samples: Arc<[f32]>,
    channels: u32,
    cursor: u64,
    reported_cursor: u64,
    scheduled_start: u64,
    clip_end: u64,
    volume: f32,
    playing: bool,
    completed: bool,
    disposed: bool,
    artboard: Option<AudioArtboardId>,
    fade: Option<FadeState>,
}

#[derive(Debug, Clone, Copy)]
struct FadeState {
    remaining: u64,
    total: u64,
}

impl SoundState {
    fn frame_count(&self) -> u64 {
        if self.channels == 0 {
            0
        } else {
            (self.samples.len() / self.channels as usize) as u64
        }
    }

    fn dispose(&mut self) {
        self.disposed = true;
        self.playing = false;
        self.completed = true;
        self.fade = None;
    }
}

#[derive(Debug)]
struct EngineState {
    frame_clock: u64,
    running: bool,
    playing: Vec<Arc<Mutex<SoundState>>>,
    completed: Vec<Arc<Mutex<SoundState>>>,
    levels: Vec<f32>,
}

#[derive(Debug)]
struct EngineShared {
    channels: u32,
    sample_rate: u32,
    state: Mutex<EngineState>,
}

impl Drop for EngineShared {
    fn drop(&mut self) {
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        for sound in state.playing.iter().chain(state.completed.iter()) {
            lock(sound).dispose();
        }
        state.playing.clear();
        state.completed.clear();
    }
}

/// A device-free Rive-owned frame clock and PCM mixer.
#[derive(Debug, Clone)]
pub struct AudioEngine {
    shared: Arc<EngineShared>,
}

impl AudioEngine {
    pub const DEFAULT_CHANNELS: u32 = 2;
    pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;

    pub fn new(channels: u32, sample_rate: u32) -> Result<Self, AudioEngineError> {
        if channels == 0 || sample_rate == 0 {
            return Err(AudioEngineError);
        }
        Ok(Self {
            shared: Arc::new(EngineShared {
                channels,
                sample_rate,
                state: Mutex::new(EngineState {
                    frame_clock: 0,
                    running: true,
                    playing: Vec::new(),
                    completed: Vec::new(),
                    levels: vec![0.0; channels as usize],
                }),
            }),
        })
    }

    pub fn channels(&self) -> u32 {
        self.shared.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.shared.sample_rate
    }

    pub fn time_in_frames(&self) -> u64 {
        lock(&self.shared.state).frame_clock
    }

    pub fn time_in_seconds(&self) -> f32 {
        self.time_in_frames() as f32 / self.sample_rate() as f32
    }

    /// Start the headless engine clock/mixer after [`Self::stop`].
    pub fn start(&self) {
        lock(&self.shared.state).running = true;
    }

    /// Suspend the headless engine clock/mixer without unlinking sounds.
    pub fn stop(&self) {
        lock(&self.shared.state).running = false;
    }

    /// Schedule a source against the engine's absolute PCM-frame clock.
    ///
    /// `start_time` and `end_time` are absolute engine frames.
    /// `sound_start_time` is the initial source cursor in engine-rate frames.
    /// A zero `end_time` means play to source end.
    pub fn play(
        &self,
        source: Arc<AudioSource>,
        start_time: u64,
        end_time: u64,
        sound_start_time: u64,
        artboard: Option<AudioArtboardId>,
    ) -> Option<AudioSound> {
        if end_time != 0 && start_time >= end_time {
            return None;
        }
        let clip_end = (end_time != 0)
            .then(|| sound_start_time.saturating_add(end_time.saturating_sub(start_time)));
        self.internal_play(source, start_time, clip_end, sound_start_time, artboard)
    }

    fn internal_play(
        &self,
        source: Arc<AudioSource>,
        scheduled_start: u64,
        requested_clip_end: Option<u64>,
        sound_start_time: u64,
        artboard: Option<AudioArtboardId>,
    ) -> Option<AudioSound> {
        let samples = source
            .decode_for_output(self.channels(), self.sample_rate())
            .ok()?;
        let total_frames = (samples.len() / self.channels() as usize) as u64;
        let clip_end = requested_clip_end.unwrap_or(total_frames).min(total_frames);
        let state = Arc::new(Mutex::new(SoundState {
            samples,
            channels: self.channels(),
            cursor: sound_start_time.min(total_frames),
            reported_cursor: 0,
            scheduled_start,
            clip_end,
            volume: 1.0,
            playing: true,
            completed: false,
            disposed: false,
            artboard,
            fade: None,
        }));
        let mut engine = lock(&self.shared.state);
        dispose_completed(&mut engine.completed);
        engine.playing.insert(0, Arc::clone(&state));
        Some(AudioSound {
            state,
            engine: Arc::downgrade(&self.shared),
        })
    }

    /// Seconds convenience matching C++'s absolute start-time helper.
    pub fn play_seconds(
        &self,
        source: Arc<AudioSource>,
        start_time_seconds: f32,
        end_time: u64,
        sound_start_time: u64,
        artboard: Option<AudioArtboardId>,
    ) -> Option<AudioSound> {
        if !start_time_seconds.is_finite() || start_time_seconds < 0.0 {
            return None;
        }
        if end_time != 0 && start_time_seconds >= end_time as f32 {
            return None;
        }
        let scheduled_start = (start_time_seconds * self.sample_rate() as f32).round() as u64;
        // Preserve pinned playSeconds' mixed-unit clip calculation: startTime
        // remains seconds while endTime/soundStartTime are PCM frames before
        // the expression is truncated through C++ `int`.
        let clip_end = (end_time != 0).then(|| {
            (sound_start_time as f32 + end_time as f32 - start_time_seconds)
                .trunc()
                .max(0.0) as u64
        });
        self.internal_play(
            source,
            scheduled_start,
            clip_end,
            sound_start_time,
            artboard,
        )
    }

    /// Fill `frames` with mixed interleaved PCM and advance the frame clock.
    ///
    /// The slice length must be a multiple of [`Self::channels`]. The return
    /// value is the number of PCM frames consumed, not the number of samples.
    pub fn read_audio_frames(&self, frames: &mut [f32]) -> u64 {
        frames.fill(0.0);
        self.mix_audio_frames(frames, false)
    }

    /// Add mixed interleaved PCM to `frames` and advance the frame clock.
    pub fn sum_audio_frames(&self, frames: &mut [f32]) -> u64 {
        let mut mixed = vec![0.0; frames.len()];
        let frames_read = self.mix_audio_frames(&mut mixed, false);
        for (output, sample) in frames.iter_mut().zip(mixed) {
            *output += sample;
        }
        frames_read
    }

    fn mix_audio_frames(&self, frames: &mut [f32], preserve_output: bool) -> u64 {
        let channels = self.channels() as usize;
        if channels == 0 || !frames.len().is_multiple_of(channels) {
            return 0;
        }
        let frame_count = frames.len() / channels;
        let mut engine = lock(&self.shared.state);
        if !engine.running {
            return 0;
        }
        if !preserve_output {
            frames.fill(0.0);
        }
        let block_start = engine.frame_clock;
        let mut newly_completed = Vec::new();

        for sound in &engine.playing {
            let mut sound_state = lock(sound);
            if sound_state.disposed || !sound_state.playing {
                continue;
            }
            // The pinned miniaudio node graph evaluates an absolute start at
            // manual-pull boundaries. If a pull begins before the timestamp,
            // even a timestamp crossed inside that pull remains silent until
            // the following pull. The live oracle binds this behavior.
            if block_start < sound_state.scheduled_start {
                continue;
            }
            for output_frame in 0..frame_count {
                if sound_state.cursor >= sound_state.clip_end
                    || sound_state.cursor >= sound_state.frame_count()
                {
                    sound_state.completed = true;
                    sound_state.playing = false;
                    newly_completed.push(Arc::clone(sound));
                    break;
                }
                let gain = sound_state.fade.map_or(1.0, |fade| {
                    if fade.total == 0 {
                        0.0
                    } else {
                        fade.remaining as f32 / fade.total as f32
                    }
                });
                let source_offset = sound_state.cursor as usize * channels;
                let output_offset = output_frame * channels;
                for channel in 0..channels {
                    if let (Some(source), Some(output)) = (
                        sound_state.samples.get(source_offset + channel),
                        frames.get_mut(output_offset + channel),
                    ) {
                        *output += *source * sound_state.volume * gain;
                    }
                }
                sound_state.cursor = sound_state.cursor.saturating_add(1);
                sound_state.reported_cursor = sound_state.cursor;
                if let Some(fade) = sound_state.fade.as_mut() {
                    fade.remaining = fade.remaining.saturating_sub(1);
                    if fade.remaining == 0 {
                        sound_state.playing = false;
                        sound_state.completed = true;
                        newly_completed.push(Arc::clone(sound));
                        break;
                    }
                }
                if sound_state.cursor >= sound_state.clip_end
                    || sound_state.cursor >= sound_state.frame_count()
                {
                    sound_state.completed = true;
                    sound_state.playing = false;
                    newly_completed.push(Arc::clone(sound));
                    break;
                }
            }
        }

        if !newly_completed.is_empty() {
            engine.playing.retain(|candidate| {
                !newly_completed
                    .iter()
                    .any(|completed| Arc::ptr_eq(candidate, completed))
            });
            engine.completed.extend(newly_completed);
        }
        for frame in frames.chunks_exact(channels) {
            for (channel, sample) in frame.iter().copied().enumerate() {
                if let Some(level) = engine.levels.get_mut(channel) {
                    *level = level.max(sample);
                }
            }
        }
        engine.frame_clock = engine.frame_clock.saturating_add(frame_count as u64);
        frame_count as u64
    }

    /// Stop and unlink every active sound tagged with `artboard`.
    pub fn stop_artboard(&self, artboard: AudioArtboardId) {
        self.stop_matching(Some(artboard));
    }

    /// Stop and unlink every active sound.
    pub fn stop_all_sounds(&self) {
        self.stop_matching(None);
    }

    fn stop_matching(&self, artboard: Option<AudioArtboardId>) {
        let mut engine = lock(&self.shared.state);
        let mut retained = Vec::with_capacity(engine.playing.len());
        let mut stopped = Vec::new();
        for sound in engine.playing.drain(..) {
            let matches = artboard.is_none() || lock(&sound).artboard == artboard;
            if matches {
                lock(&sound).playing = false;
                stopped.push(sound);
            } else {
                retained.push(sound);
            }
        }
        engine.playing = retained;
        engine.completed.extend(stopped);
    }

    pub fn playing_sound_count(&self) -> usize {
        lock(&self.shared.state).playing.len()
    }

    /// Return and reset the peak-positive sample for one channel.
    pub fn level(&self, channel: u32) -> f32 {
        let mut engine = lock(&self.shared.state);
        let Some(level) = engine.levels.get_mut(channel as usize) else {
            return 0.0;
        };
        let value = *level;
        *level = 0.0;
        value
    }

    /// Copy and reset peak-positive samples into the supplied channels.
    pub fn levels(&self, levels: &mut [f32]) {
        let mut engine = lock(&self.shared.state);
        for (output, level) in levels.iter_mut().zip(engine.levels.iter_mut()) {
            *output = *level;
            *level = 0.0;
        }
    }
}

fn dispose_completed(completed: &mut Vec<Arc<Mutex<SoundState>>>) {
    for sound in completed.drain(..) {
        lock(&sound).dispose();
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    fn mono_source(samples: &[f32], sample_rate: u32) -> Arc<AudioSource> {
        Arc::new(
            AudioSource::from_buffered(samples.to_vec(), 1, sample_rate)
                .expect("valid buffered source"),
        )
    }

    #[test]
    fn absolute_frame_schedule_clip_volume_and_completion_match_pinned_glue() {
        let engine = AudioEngine::new(1, 4).expect("engine");
        let sound = engine
            .play(mono_source(&[0.0, 1.0, 2.0, 3.0, 4.0], 4), 2, 5, 1, None)
            .expect("sound");
        sound.set_volume(0.5);

        let mut before = [99.0; 2];
        assert_eq!(engine.read_audio_frames(&mut before), 2);
        assert_eq!(before, [0.0, 0.0]);
        let mut active = [99.0; 3];
        assert_eq!(engine.read_audio_frames(&mut active), 3);
        assert_eq!(active, [0.5, 1.0, 1.5]);
        let mut after = [99.0; 1];
        assert_eq!(engine.read_audio_frames(&mut after), 1);
        assert_eq!(after, [0.0]);
        assert_eq!(engine.time_in_frames(), 6);
        assert!(sound.completed());
        assert_eq!(engine.playing_sound_count(), 0);
    }

    #[test]
    fn artboard_stop_unlinks_only_matching_sounds_and_defers_disposal() {
        let engine = AudioEngine::new(1, 4).expect("engine");
        let source = mono_source(&[1.0; 8], 4);
        let first = engine
            .play(source.clone(), 0, 0, 0, Some(AudioArtboardId(1)))
            .expect("first");
        let _second = engine
            .play(source.clone(), 0, 0, 0, Some(AudioArtboardId(1)))
            .expect("second");
        let _other = engine
            .play(source.clone(), 0, 0, 0, Some(AudioArtboardId(2)))
            .expect("other");
        assert_eq!(engine.playing_sound_count(), 3);

        engine.stop_artboard(AudioArtboardId(1));
        assert_eq!(engine.playing_sound_count(), 1);
        assert!(!first.completed());

        let _cleanup_trigger = engine.play(source, 0, 0, 0, None).expect("new sound");
        assert!(first.completed());
        assert_eq!(engine.playing_sound_count(), 2);
    }

    #[test]
    fn levels_are_peak_positive_and_reset_when_observed() {
        let engine = AudioEngine::new(2, 4).expect("engine");
        let source = Arc::new(
            AudioSource::from_buffered(vec![-0.8, 0.25, 0.5, -0.9], 2, 4).expect("source"),
        );
        let _sound = engine.play(source, 0, 0, 0, None).expect("sound");
        let mut output = [0.0; 4];
        engine.read_audio_frames(&mut output);
        assert_eq!(engine.level(0), 0.5);
        assert_eq!(engine.level(0), 0.0);
        assert_eq!(engine.level(1), 0.25);
    }

    #[test]
    fn sound_handles_are_safe_after_engine_teardown() {
        let sound = {
            let engine = AudioEngine::new(1, 4).expect("engine");
            engine
                .play(mono_source(&[1.0; 8], 4), 0, 0, 0, None)
                .expect("sound")
        };
        sound.stop(0);
        assert!(sound.completed());
        assert!(!sound.seek(1));
    }

    #[test]
    fn many_sound_handles_are_safe_after_engine_teardown() {
        let sounds = {
            let engine = AudioEngine::new(1, 4).expect("engine");
            (0..20)
                .map(|_| {
                    engine
                        .play(mono_source(&[1.0; 8], 4), 0, 0, 0, None)
                        .expect("sound")
                })
                .collect::<Vec<_>>()
        };
        assert!(sounds.iter().all(AudioSound::completed));
    }

    #[test]
    fn pause_seek_resume_and_sum_preserve_the_control_contract() {
        let engine = AudioEngine::new(1, 4).expect("engine");
        let sound = engine
            .play(mono_source(&[1.0, 2.0, 3.0, 4.0], 4), 0, 0, 0, None)
            .expect("sound");
        sound.pause();
        let mut paused = [7.0; 2];
        assert_eq!(engine.sum_audio_frames(&mut paused), 2);
        assert_eq!(paused, [7.0, 7.0]);
        assert!(sound.seek(1));
        assert_eq!(sound.time_in_frames(), 0);
        sound.resume();
        let mut resumed = [0.0; 2];
        engine.read_audio_frames(&mut resumed);
        assert_eq!(resumed, [2.0, 3.0]);
        assert_eq!(sound.time_in_frames(), 3);
        assert!(!sound.completed());
    }

    #[test]
    fn play_does_not_relink_a_sound_already_unlinked_for_completion() {
        let engine = AudioEngine::new(1, 4).expect("engine");
        let source = mono_source(&[1.0], 4);
        let sound = engine.play(source.clone(), 0, 0, 0, None).expect("sound");
        engine.read_audio_frames(&mut [0.0]);
        assert_eq!(engine.playing_sound_count(), 0);

        sound.play();
        assert_eq!(engine.playing_sound_count(), 0);
        let _cleanup_trigger = engine.play(source, 0, 0, 0, None).expect("next sound");
        assert!(
            sound.completed(),
            "next engine play performs deferred disposal"
        );
    }

    #[test]
    fn interior_absolute_start_waits_for_the_next_manual_pull() {
        let engine = AudioEngine::new(1, 4).expect("engine");
        let sound = engine
            .play(mono_source(&[1.0; 8], 4), 1, 3, 0, None)
            .expect("sound");
        let mut oversized = [9.0; 4];
        engine.read_audio_frames(&mut oversized);
        assert_eq!(oversized, [0.0; 4]);
        assert!(!sound.completed());
    }

    #[test]
    fn play_seconds_preserves_pinned_mixed_unit_clipping() {
        let engine = AudioEngine::new(1, 4).expect("engine");
        let sound = engine
            .play_seconds(
                mono_source(&[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0], 4),
                0.5,
                6,
                1,
                None,
            )
            .expect("sound");
        let mut output = [0.0; 16];
        for block in output.chunks_exact_mut(8) {
            engine.read_audio_frames(block);
        }
        assert_eq!(
            output,
            [
                0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 0.0, 0.0, 0.0
            ]
        );
        assert!(sound.completed());
    }
}
