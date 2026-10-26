use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::AudioSource;

/// A source pulled on the mixing thread. Output is interleaved engine-format PCM.
/// Callbacks may control sounds and attach renderers, but must not recursively
/// pull the engine or detach themselves while rendering.
pub trait AudioRenderer: Send {
    fn render(&mut self, output: &mut [f32], frames: u32, start_frame: u64);
}

struct RendererState {
    renderer: Option<Box<dyn AudioRenderer>>,
    next_frame: Option<u64>,
}
impl std::fmt::Debug for RendererState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RendererState")
            .field("next_frame", &self.next_frame)
            .finish_non_exhaustive()
    }
}

/// An engine attachment identity. The engine owns the renderer until detached
/// or destroyed; retaining this handle never retains the engine.
#[derive(Debug, Clone)]
pub struct AudioRendererNode {
    state: Arc<Mutex<RendererState>>,
    engine: Weak<EngineShared>,
}

#[derive(Debug, Clone)]
enum MixNode {
    Sound(Arc<Mutex<SoundState>>),
    Renderer(Arc<Mutex<RendererState>>),
}

#[derive(Debug, Default)]
struct MixCache {
    output: Vec<f32>,
    consumed: usize,
}

/// Stable identity used to tag sounds for artboard-scoped teardown.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AudioArtboardId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudioEngineError;

impl std::fmt::Display for AudioEngineError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("audio engine channels must be non-zero")
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
    // Node playback time, independent of the seekable source cursor.
    mixed_frames: u64,
    scheduled_start: u64,
    clip_end: u64,
    volume: f32,
    playing: bool,
    completed: bool,
    disposed: bool,
    artboard: Option<AudioArtboardId>,
    fade: Option<FadeState>,
    fade_gain: f32,
}

#[derive(Debug, Clone, Copy)]
struct FadeState {
    remaining: u64,
    total: u64,
    stop_frame: u64,
    volume_begin: f32,
    cursor: Option<i64>,
}

impl FadeState {
    fn gain(self) -> f32 {
        let cursor = self.cursor.unwrap_or(0);
        if cursor < 0 {
            return 1.0;
        }
        if cursor == 0 {
            return self.volume_begin;
        }
        if cursor as u64 >= self.total {
            return 0.0;
        }
        let a = cursor as f32 / self.total as f32;
        self.volume_begin + (0.0 - self.volume_begin) * a
    }
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
    nodes: Vec<MixNode>,
}

#[derive(Debug)]
struct EngineShared {
    channels: u32,
    sample_rate: u32,
    state: Mutex<EngineState>,
    buffer_frames: AtomicU32,
    requested_buffer_frames: AtomicU32,
    mix: Mutex<MixCache>,
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
        for node in state.nodes.drain(..) {
            if let MixNode::Renderer(renderer) = node {
                lock(&renderer).renderer.take();
            }
        }
    }
}

/// A device-free Rive-owned frame clock and PCM mixer.
#[derive(Debug, Clone)]
pub struct AudioEngine {
    shared: Arc<EngineShared>,
}

static BUFFER_FRAMES: AtomicU32 = AtomicU32::new(0);
#[cfg(test)]
static AUDIO_TEST_LOCK: Mutex<()> = Mutex::new(());
static RUNTIME_ENGINE: Mutex<Option<AudioEngine>> = Mutex::new(None);

impl AudioEngine {
    pub const DEFAULT_CHANNELS: u32 = 2;
    pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;

    pub fn new(channels: u32, sample_rate: u32) -> Result<Self, AudioEngineError> {
        if channels == 0 {
            return Err(AudioEngineError);
        }
        let sample_rate = if sample_rate == 0 {
            Self::DEFAULT_SAMPLE_RATE
        } else {
            sample_rate
        };
        let requested_buffer_frames = BUFFER_FRAMES.load(Ordering::SeqCst);
        Ok(Self {
            shared: Arc::new(EngineShared {
                channels,
                sample_rate,
                requested_buffer_frames: AtomicU32::new(requested_buffer_frames),
                buffer_frames: AtomicU32::new(if requested_buffer_frames == 0 {
                    128
                } else {
                    requested_buffer_frames
                }),
                mix: Mutex::new(MixCache::default()),
                state: Mutex::new(EngineState {
                    frame_clock: 0,
                    running: true,
                    playing: Vec::new(),
                    completed: Vec::new(),
                    levels: vec![0.0; channels as usize],
                    nodes: Vec::new(),
                }),
            }),
        })
    }

    /// Configure engines created subsequently. Zero selects the device default,
    /// or 128 frames for an engine without a device.
    pub fn set_buffer_frames(frames: u32) {
        BUFFER_FRAMES.store(frames, Ordering::SeqCst);
    }

    pub fn buffer_frames(&self) -> u32 {
        self.shared.buffer_frames.load(Ordering::Acquire)
    }

    pub fn attach_renderer(&self, renderer: impl AudioRenderer + 'static) -> AudioRendererNode {
        let state = Arc::new(Mutex::new(RendererState {
            renderer: Some(Box::new(renderer)),
            next_frame: None,
        }));
        lock(&self.shared.state)
            .nodes
            .insert(0, MixNode::Renderer(state.clone()));
        AudioRendererNode {
            state,
            engine: Arc::downgrade(&self.shared),
        }
    }

    /// Unlink and wait until the mixing thread has left this renderer. A handle
    /// belonging to another engine is ignored; repeated detachment is harmless.
    pub fn detach_renderer(&self, node: AudioRendererNode) {
        if !Weak::ptr_eq(&node.engine, &Arc::downgrade(&self.shared)) {
            return;
        }
        lock(&self.shared.state).nodes.retain(|entry| {
            !matches!(entry,
            MixNode::Renderer(state) if Arc::ptr_eq(state, &node.state))
        });
        // Never hold engine state while waiting for a callback: callbacks can
        // themselves attach nodes or control a sound.
        let renderer = lock(&node.state).renderer.take();
        drop(renderer);
    }

    /// Construct and retain the process runtime engine used by AudioEvent
    /// playback when an Artboard has no explicitly configured engine.
    pub fn make_and_store(channels: u32, sample_rate: u32) -> Result<Self, AudioEngineError> {
        let engine = Self::new(channels, sample_rate)?;
        // Renderer destruction is user code and may reenter the runtime-engine
        // API. Release the singleton lock before dropping the previous owner.
        let previous = { lock(&RUNTIME_ENGINE).replace(engine.clone()) };
        drop(previous);
        Ok(engine)
    }

    /// Return the retained runtime engine, lazily creating the pinned default
    /// two-channel, 48 kHz headless engine when necessary.
    pub fn runtime_engine() -> Self {
        let mut runtime = lock(&RUNTIME_ENGINE);
        runtime
            .get_or_insert_with(|| {
                Self::new(Self::DEFAULT_CHANNELS, Self::DEFAULT_SAMPLE_RATE)
                    .expect("the fixed runtime audio defaults are valid")
            })
            .clone()
    }

    /// Release the process-owned reference without stopping retained engines.
    pub fn release_runtime_engine() {
        let previous = lock(&RUNTIME_ENGINE).take();
        drop(previous);
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

    /// Resume device output. Explicit manual pulls remain available regardless
    /// of the device's running state.
    pub fn start(&self) {
        lock(&self.shared.state).running = true;
    }

    /// Suspend device output without unlinking sounds or disabling manual pulls.
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
            mixed_frames: 0,
            scheduled_start,
            clip_end,
            volume: 1.0,
            playing: true,
            completed: false,
            disposed: false,
            artboard,
            fade: None,
            fade_gain: 1.0,
        }));
        let mut engine = lock(&self.shared.state);
        dispose_completed(&mut engine.completed);
        engine.playing.insert(0, Arc::clone(&state));
        engine.nodes.insert(0, MixNode::Sound(Arc::clone(&state)));
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
        // Upstream promotes the f32 seconds to double before rounding to the
        // nearest millisecond; miniaudio then converts milliseconds to frames.
        let milliseconds = (f64::from(start_time_seconds) * 1000.0).round() as u64;
        let scheduled_start = milliseconds.wrapping_mul(u64::from(self.sample_rate())) / 1000;
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
        for (output, sample) in frames
            .iter_mut()
            .zip(mixed)
            .take(frames_read as usize * self.channels() as usize)
        {
            *output += sample;
        }
        frames_read
    }

    fn mix_audio_frames(&self, frames: &mut [f32], preserve_output: bool) -> u64 {
        let channels = self.channels() as usize;
        if !frames.len().is_multiple_of(channels) {
            return 0;
        }
        let mut cache = lock(&self.shared.mix);
        if !preserve_output {
            frames.fill(0.0);
        }
        let mut written = 0;
        while written < frames.len() {
            if cache.consumed == cache.output.len() {
                let samples = self.buffer_frames() as usize * channels;
                cache.output.resize(samples, 0.0);
                cache.output.fill(0.0);
                if !self.mix_block(&mut cache.output) {
                    cache.output.clear();
                    cache.consumed = 0;
                    break;
                }
                cache.consumed = 0;
            }
            let count = (frames.len() - written).min(cache.output.len() - cache.consumed);
            frames[written..written + count]
                .copy_from_slice(&cache.output[cache.consumed..cache.consumed + count]);
            cache.consumed += count;
            written += count;
        }
        (written / channels) as u64
    }

    fn mix_block(&self, frames: &mut [f32]) -> bool {
        let channels = self.channels() as usize;
        let frame_count = frames.len() / channels;
        let (block_start, nodes) = {
            let engine = lock(&self.shared.state);
            (engine.frame_clock, engine.nodes.clone())
        };
        if nodes.is_empty() {
            return false;
        }
        let mut newly_completed = Vec::new();
        for node in &nodes {
            match node {
                MixNode::Renderer(state) => {
                    let mut state = lock(state);
                    let next = state.next_frame.unwrap_or(block_start);
                    if let Some(renderer) = state.renderer.as_mut() {
                        let mut output = vec![0.0; frames.len()];
                        renderer.render(&mut output, frame_count as u32, next);
                        for (dst, src) in frames.iter_mut().zip(output) {
                            *dst += src;
                        }
                        state.next_frame = Some(next.wrapping_add(frame_count as u64));
                    }
                }
                MixNode::Sound(sound) => {
                    let mut sound_state = lock(sound);
                    if sound_state.disposed || !sound_state.playing {
                        continue;
                    }
                    // Scheduled starts are relative to this whole processing block,
                    // even when the caller consumes it through smaller cached pulls.
                    let output_start = sound_state.scheduled_start.saturating_sub(block_start);
                    if output_start >= frame_count as u64 {
                        continue;
                    }
                    let current_gain = sound_state.fade_gain;
                    if let Some(fade) = sound_state.fade.as_mut() {
                        if fade.cursor.is_none() {
                            // miniaudio consumes pending fade settings once, using
                            // the graph clock for the initial offset. Thereafter
                            // its fader advances only when this sound renders.
                            fade.volume_begin = current_gain;
                            let start = fade.stop_frame.wrapping_sub(fade.total);
                            let offset = (start as i64)
                                .wrapping_sub(block_start as i64)
                                .min(i32::MAX as i64);
                            fade.cursor = Some(offset.wrapping_neg());
                            fade.total = fade.total.min(u32::MAX as u64);
                        }
                    }
                    for output_frame in output_start as usize..frame_count {
                        if sound_state.fade.is_some_and(|fade| {
                            block_start + output_frame as u64 >= fade.stop_frame
                        }) {
                            sound_state.playing = false;
                            break;
                        }
                        if sound_state.cursor >= sound_state.clip_end
                            || sound_state.cursor >= sound_state.frame_count()
                        {
                            sound_state.completed = true;
                            sound_state.playing = false;
                            newly_completed.push(Arc::clone(sound));
                            break;
                        }
                        let gain = sound_state
                            .fade
                            .map_or(sound_state.fade_gain, |fade| fade.gain());
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
                        sound_state.mixed_frames = sound_state.mixed_frames.wrapping_add(1);
                        sound_state.cursor = sound_state.cursor.saturating_add(1);
                        sound_state.reported_cursor = sound_state.cursor;
                        if let Some(fade) = sound_state.fade.as_mut() {
                            fade.remaining = fade.remaining.saturating_sub(1);
                            fade.cursor =
                                fade.cursor.map(|cursor| (cursor + 1).min(u32::MAX as i64));
                            sound_state.fade_gain = fade.gain();
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
            }
            let mut engine = lock(&self.shared.state);
            if !newly_completed.is_empty() {
                engine.playing.retain(|candidate| {
                    !newly_completed
                        .iter()
                        .any(|done| Arc::ptr_eq(candidate, done))
                });
                engine.nodes.retain(|node| !matches!(node, MixNode::Sound(sound) if newly_completed.iter().any(|done| Arc::ptr_eq(sound, done))));
                engine.completed.append(&mut newly_completed);
            }
        }
        let mut engine = lock(&self.shared.state);
        for frame in frames.chunks_exact(channels) {
            for (channel, sample) in frame.iter().copied().enumerate() {
                if let Some(level) = engine.levels.get_mut(channel) {
                    *level = level.max(sample);
                }
            }
        }
        engine.frame_clock = engine.frame_clock.wrapping_add(frame_count as u64);
        true
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
        engine.nodes.retain(|node| !matches!(node, MixNode::Sound(sound) if stopped.iter().any(|done| Arc::ptr_eq(sound, done))));
        engine.completed.extend(stopped);
    }

    pub fn playing_sound_count(&self) -> usize {
        lock(&self.shared.state).playing.len()
    }

    /// Current newest playing sound, mirroring the pinned TESTING seam.
    #[doc(hidden)]
    pub fn playing_sounds_head(&self) -> Option<AudioSound> {
        lock(&self.shared.state)
            .playing
            .first()
            .cloned()
            .map(|state| AudioSound {
                state,
                engine: Arc::downgrade(&self.shared),
            })
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

include!("audio_sound.rs");

#[cfg(test)]
mod tests {
    use super::*;

    fn engine_with_config(
        channels: u32,
        rate: u32,
        frames: u32,
    ) -> Result<AudioEngine, AudioEngineError> {
        AudioEngine::set_buffer_frames(frames);
        let result = AudioEngine::new(channels, rate);
        AudioEngine::set_buffer_frames(0);
        result
    }

    struct BlockLog(Arc<Mutex<Vec<(u32, u64)>>>);
    impl AudioRenderer for BlockLog {
        fn render(&mut self, out: &mut [f32], frames: u32, start: u64) {
            out.fill(0.0);
            lock(&self.0).push((frames, start));
        }
    }

    #[test]
    fn replacing_runtime_engine_drops_renderers_outside_singleton_lock() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        struct ReentersOnDrop(Arc<std::sync::atomic::AtomicBool>);
        impl AudioRenderer for ReentersOnDrop {
            fn render(&mut self, output: &mut [f32], _: u32, _: u64) {
                output.fill(0.0);
            }
        }
        impl Drop for ReentersOnDrop {
            fn drop(&mut self) {
                // try_lock makes a regression fail immediately, not hang. Then
                // exercise the public accessor once the lock's availability is
                // established, confirming the replacement is already published.
                let guard = RUNTIME_ENGINE
                    .try_lock()
                    .expect("singleton unlocked during renderer drop");
                drop(guard);
                assert_eq!(AudioEngine::runtime_engine().sample_rate(), 44100);
                self.0.store(true, Ordering::Release);
            }
        }
        AudioEngine::release_runtime_engine();
        let dropped = Arc::new(std::sync::atomic::AtomicBool::new(false));
        {
            let old = AudioEngine::make_and_store(2, 48000).unwrap();
            old.attach_renderer(ReentersOnDrop(dropped.clone()));
        }
        let replacement = AudioEngine::make_and_store(2, 44100).unwrap();
        assert!(dropped.load(Ordering::Acquire));
        assert_eq!(replacement.sample_rate(), 44100);
        AudioEngine::release_runtime_engine();
    }

    #[test]
    fn partial_pulls_render_whole_configured_blocks() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(2, 48000, 2048).unwrap();
        assert_eq!(engine.buffer_frames(), 2048);
        let blocks = Arc::new(Mutex::new(Vec::new()));
        let node = engine.attach_renderer(BlockLog(blocks.clone()));
        for _ in 0..4 {
            mix_stereo(&engine, 800);
        }
        assert_eq!(*lock(&blocks), vec![(2048, 0), (2048, 2048)]);
        engine.detach_renderer(node);
    }

    #[test]
    fn configured_runtime_engine_and_default_headless_rate() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        AudioEngine::release_runtime_engine();
        AudioEngine::set_buffer_frames(4096);
        assert_eq!(AudioEngine::runtime_engine().buffer_frames(), 4096);
        assert_eq!(
            AudioEngine::make_and_store(2, 44100)
                .unwrap()
                .buffer_frames(),
            4096
        );
        AudioEngine::set_buffer_frames(0);
        assert_eq!(AudioEngine::runtime_engine().buffer_frames(), 4096);
        AudioEngine::release_runtime_engine();
        assert_eq!(AudioEngine::runtime_engine().buffer_frames(), 128);
        AudioEngine::release_runtime_engine();
        assert_eq!(AudioEngine::new(2, 0).unwrap().sample_rate(), 48000);
        assert_eq!(AudioEngine::new(2, 44100).unwrap().sample_rate(), 44100);
        assert_eq!(AudioEngine::runtime_engine().sample_rate(), 48000);
        assert_eq!(
            AudioEngine::make_and_store(2, 0).unwrap().sample_rate(),
            48000
        );
        AudioEngine::release_runtime_engine();
    }

    #[test]
    fn renderer_attached_during_mix_starts_at_next_block_clock() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        struct Attaches {
            engine: AudioEngine,
            first: BlockLog,
            late: Option<BlockLog>,
            node: Arc<Mutex<Option<AudioRendererNode>>>,
        }
        impl AudioRenderer for Attaches {
            fn render(&mut self, out: &mut [f32], frames: u32, start: u64) {
                self.first.render(out, frames, start);
                if let Some(late) = self.late.take() {
                    *lock(&self.node) = Some(self.engine.attach_renderer(late));
                }
            }
        }
        let engine = engine_with_config(2, 48000, 4096).unwrap();
        let first = Arc::new(Mutex::new(Vec::new()));
        let late = Arc::new(Mutex::new(Vec::new()));
        let late_node = Arc::new(Mutex::new(None));
        let node = engine.attach_renderer(Attaches {
            engine: engine.clone(),
            first: BlockLog(first.clone()),
            late: Some(BlockLog(late.clone())),
            node: late_node.clone(),
        });
        mix_stereo(&engine, 4096);
        mix_stereo(&engine, 4096);
        assert_eq!(*lock(&first), vec![(4096, 0), (4096, 4096)]);
        assert_eq!(*lock(&late), vec![(4096, 4096)]);
        engine.detach_renderer(lock(&late_node).take().unwrap());
        engine.detach_renderer(node);
    }

    #[test]
    fn sound_stopped_during_its_first_block_fades_at_absolute_stop_frame() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        struct Stops(Arc<Mutex<Option<AudioSound>>>, bool);
        impl AudioRenderer for Stops {
            fn render(&mut self, out: &mut [f32], _: u32, _: u64) {
                out.fill(0.0);
                if !self.1 {
                    self.1 = true;
                    lock(&self.0).as_ref().unwrap().stop(8000);
                }
            }
        }
        let engine = engine_with_config(2, 48000, 4096).unwrap();
        let sound = Arc::new(Mutex::new(None));
        let node = engine.attach_renderer(Stops(sound.clone(), false));
        *lock(&sound) = engine.play(stereo_source(vec![1.0; 48000 * 2]), 1000, 0, 0, None);
        let out = mix_stereo(&engine, 3 * 4096);
        assert_eq!(first_heard(&out), Some(1000));
        assert_eq!(
            out.chunks_exact(2).rposition(|frame| frame[0] != 0.0),
            Some(7999)
        );
        engine.detach_renderer(node);
    }

    #[test]
    fn renderer_mixes_with_sounds_and_detach_retains_already_mixed_cache() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        struct Constant;
        impl AudioRenderer for Constant {
            fn render(&mut self, out: &mut [f32], _: u32, _: u64) {
                out.fill(0.25);
            }
        }
        let engine = engine_with_config(2, 48000, 4).unwrap();
        let node = engine.attach_renderer(Constant);
        engine
            .play(stereo_source(vec![0.5; 16]), 0, 0, 0, None)
            .unwrap();
        assert_eq!(mix_stereo(&engine, 1), vec![0.75; 2]);
        engine.detach_renderer(node);
        assert_eq!(mix_stereo(&engine, 3), vec![0.75; 6]);
        assert_eq!(mix_stereo(&engine, 4), vec![0.5; 8]);
        assert_eq!(engine.time_in_frames(), 8);
        let mut idle = [99.0; 8];
        assert_eq!(engine.read_audio_frames(&mut idle), 0);
        assert_eq!(idle, [0.0; 8]);
        assert_eq!(engine.time_in_frames(), 8);
    }

    #[test]
    fn detach_waits_for_inflight_callback_and_prevents_future_calls() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        use std::sync::mpsc;
        struct Blocking {
            entered: mpsc::Sender<()>,
            leave: mpsc::Receiver<()>,
            dropped: mpsc::Sender<()>,
        }
        impl AudioRenderer for Blocking {
            fn render(&mut self, output: &mut [f32], _: u32, _: u64) {
                self.entered.send(()).unwrap();
                self.leave.recv().unwrap();
                output.fill(0.5);
            }
        }
        impl Drop for Blocking {
            fn drop(&mut self) {
                self.dropped.send(()).unwrap();
            }
        }
        let engine = engine_with_config(2, 48000, 4).unwrap();
        let (entered, entry) = mpsc::channel();
        let (leave, release) = mpsc::channel();
        let (dropped, drop_rx) = mpsc::channel();
        let node = engine.attach_renderer(Blocking {
            entered,
            leave: release,
            dropped,
        });
        let mix_engine = engine.clone();
        let mixing = std::thread::spawn(move || mix_stereo(&mix_engine, 4));
        entry.recv().unwrap();
        let detach_engine = engine.clone();
        let state = node.state.clone();
        let detaching = std::thread::spawn(move || detach_engine.detach_renderer(node));
        // Wait until detach has unlinked the node, then verify it still cannot
        // destroy an executing callback. No timing-based sleep/assertion.
        while lock(&engine.shared.state).nodes.iter().any(
            |entry| matches!(entry, MixNode::Renderer(candidate) if Arc::ptr_eq(candidate, &state)),
        ) {
            std::thread::yield_now();
        }
        assert!(drop_rx.try_recv().is_err());
        leave.send(()).unwrap();
        assert_eq!(mixing.join().unwrap(), vec![0.5; 8]);
        detaching.join().unwrap();
        drop_rx.recv().unwrap();
        let mut idle = [99.0; 8];
        assert_eq!(engine.read_audio_frames(&mut idle), 0);
        assert_eq!(idle, [0.0; 8]);
    }

    #[test]
    fn replacing_a_fade_starts_from_its_current_gain_without_completing_sound() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 48000, 1).unwrap();
        let sound = engine
            .play(mono_source(&[1.0; 100], 48000), 0, 0, 0, None)
            .unwrap();
        assert_eq!(engine.read_audio_frames(&mut [0.0]), 1);
        sound.stop(8);
        let mut first = [0.0; 4];
        assert_eq!(engine.read_audio_frames(&mut first), 4);
        assert_eq!(first, [1.0, 0.875, 0.75, 0.625]);
        sound.stop(4);
        let mut second = [0.0; 5];
        assert_eq!(engine.read_audio_frames(&mut second), 5);
        assert_eq!(second, [0.5, 0.375, 0.25, 0.125, 0.0]);
        assert!(!sound.completed());
        assert_eq!(engine.playing_sound_count(), 1);
        assert!(!lock(&sound.state).playing);
        sound.play();
        assert!(lock(&sound.state).playing);
        assert!(!sound.completed());
        assert_eq!(engine.playing_sound_count(), 1);
        assert!(sound.seek(1));
        let mut restarted = [99.0; 1];
        assert_eq!(engine.read_audio_frames(&mut restarted), 1);
        assert_eq!(restarted, [0.0]);
        assert!(!lock(&sound.state).playing);
        assert!(!sound.completed());
        assert_eq!(engine.playing_sound_count(), 1);
        sound.resume();
        let mut resumed = [99.0; 1];
        assert_eq!(engine.read_audio_frames(&mut resumed), 1);
        assert_eq!(resumed, [0.0]);
        assert!(!sound.completed());
    }

    #[test]
    fn manual_pull_remains_available_while_device_output_is_stopped() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 48000, 1).unwrap();
        engine
            .play(mono_source(&[0.5; 4], 48000), 0, 0, 0, None)
            .unwrap();
        engine.stop();
        let mut samples = [99.0; 2];
        assert_eq!(engine.read_audio_frames(&mut samples), 2);
        assert_eq!(samples, [0.5; 2]);
        assert_eq!(engine.time_in_frames(), 2);
    }

    #[test]
    fn paused_fader_only_advances_when_its_sound_renders() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 48000, 1).unwrap();
        let node = engine.attach_renderer(BlockLog(Arc::new(Mutex::new(Vec::new()))));
        let sound = engine
            .play(mono_source(&[1.0; 30], 48000), 0, 0, 0, None)
            .unwrap();
        assert_eq!(engine.read_audio_frames(&mut [0.0]), 1);
        sound.stop(8);
        let mut fading = [0.0; 2];
        assert_eq!(engine.read_audio_frames(&mut fading), 2);
        assert_eq!(fading, [1.0, 0.875]);
        sound.pause();
        let mut paused = [99.0; 2];
        assert_eq!(engine.read_audio_frames(&mut paused), 2);
        assert_eq!(paused, [0.0; 2]);
        sound.resume();
        let mut resumed = [99.0; 2];
        assert_eq!(engine.read_audio_frames(&mut resumed), 2);
        assert_eq!(resumed, [0.75, 0.625]);
        let mut end = [99.0; 3];
        assert_eq!(engine.read_audio_frames(&mut end), 3);
        assert_eq!(end, [0.5, 0.375, 0.0]);
        assert!(!sound.completed());
        assert_eq!(engine.playing_sound_count(), 1);
        engine.detach_renderer(node);
    }

    #[test]
    fn scheduling_a_fade_does_not_restart_a_paused_sound() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 48000, 1).unwrap();
        let sound = engine
            .play(mono_source(&[1.0; 30], 48000), 0, 0, 0, None)
            .unwrap();
        assert_eq!(engine.read_audio_frames(&mut [0.0]), 1);
        sound.pause();
        sound.stop(4);
        assert!(!lock(&sound.state).playing);
        let mut paused = [99.0; 1];
        assert_eq!(engine.read_audio_frames(&mut paused), 1);
        assert_eq!(paused, [0.0]);
        sound.resume();
        let mut resumed = [99.0; 1];
        assert_eq!(engine.read_audio_frames(&mut resumed), 1);
        assert_eq!(resumed, [0.75]);
    }

    #[test]
    fn partial_final_cache_reports_only_frames_read_and_silences_tail() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 48000, 4).unwrap();
        engine
            .play(mono_source(&[1.0; 2], 48000), 0, 0, 0, None)
            .unwrap();
        let mut first = [99.0; 1];
        assert_eq!(engine.read_audio_frames(&mut first), 1);
        assert_eq!(first, [1.0]);
        let mut rest = [99.0; 5];
        assert_eq!(engine.read_audio_frames(&mut rest), 3);
        assert_eq!(rest, [1.0, 0.0, 0.0, 0.0, 0.0]);
        assert_eq!(engine.time_in_frames(), 4);
        let mut summed = [3.0, -0.0];
        assert_eq!(engine.sum_audio_frames(&mut summed), 0);
        assert_eq!(summed[0], 3.0);
        assert_eq!(summed[1].to_bits(), (-0.0_f32).to_bits());
    }

    fn mono_source(samples: &[f32], sample_rate: u32) -> Arc<AudioSource> {
        Arc::new(
            AudioSource::from_buffered(samples.to_vec(), 1, sample_rate)
                .expect("valid buffered source"),
        )
    }

    #[test]
    fn seconds_schedule_rounds_milliseconds_before_engine_frames() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = AudioEngine::new(2, 48000).unwrap();
        let source = Arc::new(AudioSource::from_buffered(vec![1.0; 96000], 2, 48000).unwrap());
        let sound = engine
            .play_seconds(source.clone(), 0.251, 0, 0, None)
            .unwrap();
        assert_eq!(lock(&sound.state).scheduled_start, 12048);
        // Frame rounding alone would produce 25, rather than one millisecond.
        let sound = engine.play_seconds(source, 0.00051, 0, 0, None).unwrap();
        assert_eq!(lock(&sound.state).scheduled_start, 48);
    }

    fn stereo_source(samples: Vec<f32>) -> Arc<AudioSource> {
        Arc::new(AudioSource::from_buffered(samples, 2, 48000).unwrap())
    }

    fn mix_stereo(engine: &AudioEngine, frames: usize) -> Vec<f32> {
        let mut output = vec![0.0; frames * 2];
        assert_eq!(engine.read_audio_frames(&mut output), frames as u64);
        output
    }

    fn first_heard(stereo: &[f32]) -> Option<usize> {
        stereo.chunks_exact(2).position(|frame| frame[0] != 0.0)
    }

    #[test]
    fn scheduled_sound_starts_on_its_frame_inside_a_pull() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        for frame in [1000, 5000] {
            let engine = engine_with_config(2, 48000, 4096).unwrap();
            assert!(
                engine
                    .play(stereo_source(vec![1.0; 48000 * 2]), frame, 0, 0, None)
                    .is_some()
            );
            assert_eq!(
                first_heard(&mix_stereo(&engine, 2 * 4096)),
                Some(frame as usize)
            );
        }
        // The source float 0.251 is slightly below 251 milliseconds.
        let engine = engine_with_config(2, 48000, 4096).unwrap();
        assert!(
            engine
                .play_seconds(stereo_source(vec![1.0; 48000 * 2]), 0.251, 0, 0, None)
                .is_some()
        );
        assert_eq!(first_heard(&mix_stereo(&engine, 3 * 4096)), Some(12048));
        let engine = engine_with_config(2, 48000, 4096).unwrap();
        let node = engine.attach_renderer(BlockLog(Arc::new(Mutex::new(Vec::new()))));
        mix_stereo(&engine, 4096);
        assert_eq!(engine.time_in_frames(), 4096);
        assert!(
            engine
                .play(stereo_source(vec![1.0; 48000 * 2]), 100, 0, 0, None)
                .is_some()
        );
        assert_eq!(first_heard(&mix_stereo(&engine, 4096)), Some(0));
        engine.detach_renderer(node);
    }

    #[test]
    fn scheduled_sound_completes_in_the_pull_it_ends_in() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(2, 48000, 4096).unwrap();
        let sound = engine
            .play(stereo_source(vec![1.0; 1000 * 2]), 1000, 0, 0, None)
            .unwrap();
        mix_stereo(&engine, 4096);
        assert!(sound.completed());
        assert_eq!(engine.playing_sound_count(), 0);
    }

    #[test]
    fn scheduled_sound_under_newer_sound_keeps_exact_frames_for_all_pull_sizes() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let length = 4096;
        // Native peak monitoring is always present, so there is no separate
        // monitor-node attachment path to exercise. Preserve every source
        // read-size/start combination and every PCM sample assertion.
        for read in [1000, 2048, 4096] {
            for start in [1, 479, 480, 1000, 2047] {
                let engine = AudioEngine::new(2, 48000).unwrap();
                let ramp = (0..length * 2)
                    .map(|i| (i / 2 + 1) as f32 / length as f32)
                    .collect();
                assert!(
                    engine
                        .play(stereo_source(ramp), start, 0, 0, None)
                        .is_some()
                );
                assert!(
                    engine
                        .play(stereo_source(vec![0.0; length * 2]), 0, 0, 0, None)
                        .is_some()
                );
                let mut heard = Vec::new();
                while heard.len() < (start as usize + length) * 2 {
                    let mut output = vec![0.0; read * 2];
                    let processing_end = (start as usize + length).div_ceil(128) * 128;
                    let expected = read.min(processing_end.saturating_sub(heard.len() / 2));
                    assert_eq!(engine.read_audio_frames(&mut output), expected as u64);
                    heard.extend(output);
                }
                assert_eq!(first_heard(&heard), Some(start as usize));
                let wrong = heard[..(start as usize + length) * 2]
                    .iter()
                    .enumerate()
                    .filter(|&(i, sample)| {
                        let frame = i / 2;
                        let want = if frame < start as usize {
                            0.0
                        } else {
                            (frame - start as usize + 1) as f32 / length as f32
                        };
                        (*sample - want).abs() > 1e-6
                    })
                    .count();
                assert_eq!(wrong, 0, "read={read}, start={start}");
                assert_eq!(engine.playing_sound_count(), 0);
            }
        }
    }

    #[test]
    fn sound_fades_to_silence_over_its_fade_frames() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(2, 48000, 4096).unwrap();
        let source = Arc::new(AudioSource::from_buffered(vec![1.0; 96000], 2, 48000).unwrap());
        let sound = engine.play(source, 0, 0, 0, None).unwrap();
        assert_eq!(engine.read_audio_frames(&mut vec![0.0; 4096 * 2]), 4096);
        sound.stop(1000);
        let mut out = vec![0.0; 4096 * 2];
        assert_eq!(engine.read_audio_frames(&mut out), 4096);
        let wrong = out
            .iter()
            .enumerate()
            .filter(|&(i, sample)| {
                let frame = i / 2;
                let want = if frame < 1000 {
                    1.0 - frame as f32 / 1000.0
                } else {
                    0.0
                };
                (*sample - want).abs() > 1e-6
            })
            .count();
        assert_eq!(wrong, 0);
    }

    #[test]
    fn scheduled_stop_before_start_is_immediate_even_when_fade_outlasts_start() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        for fade in [1000, 4000] {
            let engine = engine_with_config(2, 48000, 4096).unwrap();
            let source = Arc::new(AudioSource::from_buffered(vec![1.0; 4000], 2, 48000).unwrap());
            let sound = engine.play(source, 3000, 0, 0, None).unwrap();
            sound.stop(fade);
            let mut out = vec![0.5; 2 * 4096 * 2];
            assert_eq!(engine.read_audio_frames(&mut out[..4096 * 2]), 4096);
            assert_eq!(
                out[4096 * 2..].iter().filter(|&&v| v == 0.5).count(),
                4096 * 2
            );
            assert!(out[..4096 * 2].iter().all(|&v| v == 0.0));
            let mut rest = vec![0.0; 2 * 4096 * 2];
            assert_eq!(engine.read_audio_frames(&mut rest), 2 * 4096);
            assert!(rest.iter().all(|&v| v == 0.0));
            assert!(!sound.completed());
            assert_eq!(engine.playing_sound_count(), 1);
        }
    }

    #[test]
    fn scheduled_stop_distinguishes_node_playback_from_source_seek() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = AudioEngine::new(1, 48000).unwrap();
        let sound = engine
            .play(mono_source(&[1.0; 4000], 48000), 3000, 0, 100, None)
            .unwrap();
        sound.stop(4000);
        assert!(!lock(&sound.state).playing);
        // The node has mixed while the engine clock still trails the block.
        // Retain the focused guard test alongside the full callback regression.
        let sound = engine
            .play(mono_source(&[1.0; 4000], 48000), 3000, 0, 0, None)
            .unwrap();
        lock(&sound.state).mixed_frames = 1;
        sound.stop(4000);
        assert_eq!(lock(&sound.state).fade.unwrap().remaining, 4000);
        assert!(lock(&sound.state).playing);
    }

    #[test]
    fn absolute_frame_schedule_clip_volume_and_completion_match_pinned_glue() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 4, 1).expect("engine");
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
        assert_eq!(engine.read_audio_frames(&mut after), 0);
        assert_eq!(after, [0.0]);
        // Once the last sound unlinks, idle pulls do not move the graph clock.
        assert_eq!(engine.time_in_frames(), 5);
        assert!(sound.completed());
        assert_eq!(engine.playing_sound_count(), 0);
    }

    #[test]
    fn artboard_stop_unlinks_only_matching_sounds_and_defers_disposal() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 4, 1).expect("engine");
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
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(2, 4, 1).expect("engine");
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
        let _serial = lock(&AUDIO_TEST_LOCK);
        let sound = {
            let engine = engine_with_config(1, 4, 1).expect("engine");
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
        let _serial = lock(&AUDIO_TEST_LOCK);
        let sounds = {
            let engine = engine_with_config(1, 4, 1).expect("engine");
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
    fn stored_runtime_engine_is_the_default_headless_playback_owner() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let stored = AudioEngine::make_and_store(
            AudioEngine::DEFAULT_CHANNELS,
            AudioEngine::DEFAULT_SAMPLE_RATE,
        )
        .expect("stored runtime engine");
        let runtime = AudioEngine::runtime_engine();
        runtime
            .play(
                mono_source(&[1.0; 8], AudioEngine::DEFAULT_SAMPLE_RATE),
                0,
                0,
                0,
                None,
            )
            .expect("runtime-owned sound");

        assert_eq!(stored.playing_sound_count(), 1);
        stored.stop_all_sounds();
    }

    #[test]
    fn pause_seek_resume_and_sum_preserve_the_control_contract() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 4, 1).expect("engine");
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
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 4, 1).expect("engine");
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
    fn interior_absolute_start_plays_at_its_frame_in_the_current_pull() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 4, 1).expect("engine");
        let sound = engine
            .play(mono_source(&[1.0; 8], 4), 1, 3, 0, None)
            .expect("sound");
        let mut oversized = [9.0; 4];
        engine.read_audio_frames(&mut oversized);
        // 4252d730 updates miniaudio scheduling from pull-boundary to exact
        // frame starts; the existing two-frame clip still has the same length.
        assert_eq!(oversized, [0.0, 1.0, 1.0, 0.0]);
        assert!(sound.completed());
    }

    #[test]
    fn play_seconds_preserves_pinned_mixed_unit_clipping() {
        let _serial = lock(&AUDIO_TEST_LOCK);
        let engine = engine_with_config(1, 4, 1).expect("engine");
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
                0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0
            ]
        );
        assert!(sound.completed());
    }
}

#[cfg(feature = "audio-device")]
mod device {
    //! Optional CPAL output for the deterministic headless mixer.
    //!
    //! The pinned C++ engine selects between a miniaudio-owned device and an
    //! external/manual pull at construction time. Rust keeps the manual-pull
    //! mixer authoritative and lets this sink drain it from CPAL's callback.

    use std::fmt;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use cpal::{FromSample, Sample, SampleFormat, SizedSample, SupportedBufferSize};

    use crate::AudioEngine;

    /// Failure to attach the headless mixer to the default output device.
    #[derive(Debug)]
    pub enum AudioDeviceError {
        NoDefaultOutputDevice,
        DefaultOutputConfig(cpal::DefaultStreamConfigError),
        FormatMismatch {
            engine_channels: u32,
            engine_sample_rate: u32,
            device_channels: u32,
            device_sample_rate: u32,
        },
        UnsupportedSampleFormat(SampleFormat),
        BuildStream(cpal::BuildStreamError),
        PlayStream(cpal::PlayStreamError),
    }

    impl fmt::Display for AudioDeviceError {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::NoDefaultOutputDevice => {
                    formatter.write_str("no default audio output device")
                }
                Self::DefaultOutputConfig(error) => {
                    write!(
                        formatter,
                        "failed to query the default audio output format: {error}"
                    )
                }
                Self::FormatMismatch {
                    engine_channels,
                    engine_sample_rate,
                    device_channels,
                    device_sample_rate,
                } => write!(
                    formatter,
                    "audio engine format {engine_channels}ch/{engine_sample_rate}Hz does not match the default output format {device_channels}ch/{device_sample_rate}Hz",
                ),
                Self::UnsupportedSampleFormat(format) => {
                    write!(
                        formatter,
                        "unsupported default audio sample format {format}"
                    )
                }
                Self::BuildStream(error) => {
                    write!(formatter, "failed to build audio output: {error}")
                }
                Self::PlayStream(error) => {
                    write!(formatter, "failed to start audio output: {error}")
                }
            }
        }
    }

    impl std::error::Error for AudioDeviceError {}

    /// A live default-device stream draining an [`AudioEngine`].
    ///
    /// Dropping the sink drops CPAL's stream. The engine remains usable for
    /// deterministic manual pulls after the sink is gone, provided the caller did
    /// not pull it concurrently while the stream was live.
    pub struct AudioDeviceSink {
        engine: AudioEngine,
        _stream: cpal::Stream,
        output_failed: Arc<AtomicBool>,
    }

    impl fmt::Debug for AudioDeviceSink {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter
                .debug_struct("AudioDeviceSink")
                .field("channels", &self.engine.channels())
                .field("sample_rate", &self.engine.sample_rate())
                .finish_non_exhaustive()
        }
    }

    impl AudioDeviceSink {
        /// Create a headless mixer matching the default output format and start a
        /// device sink that drains it.
        pub fn open_default() -> Result<Self, AudioDeviceError> {
            let (device, supported) = default_output()?;
            let engine = AudioEngine::new(supported.channels() as u32, supported.sample_rate().0)
                .expect("CPAL output formats always have non-zero channels and sample rate");
            Self::build(device, supported, engine)
        }

        /// Start a default-device sink for an existing deterministic mixer.
        ///
        /// The default format must exactly match because resampling belongs to the
        /// source readers and the mixer owns the authoritative PCM-frame clock.
        pub fn attach_default(engine: AudioEngine) -> Result<Self, AudioDeviceError> {
            let (device, supported) = default_output()?;
            let device_channels = supported.channels() as u32;
            let device_sample_rate = supported.sample_rate().0;
            if engine.channels() != device_channels || engine.sample_rate() != device_sample_rate {
                return Err(AudioDeviceError::FormatMismatch {
                    engine_channels: engine.channels(),
                    engine_sample_rate: engine.sample_rate(),
                    device_channels,
                    device_sample_rate,
                });
            }
            Self::build(device, supported, engine)
        }

        pub fn engine(&self) -> &AudioEngine {
            &self.engine
        }

        /// Whether CPAL has reported an asynchronous stream failure.
        pub fn output_failed(&self) -> bool {
            self.output_failed.load(Ordering::Acquire)
        }

        fn build(
            device: cpal::Device,
            supported: cpal::SupportedStreamConfig,
            engine: AudioEngine,
        ) -> Result<Self, AudioDeviceError> {
            let mut config = supported.config();
            let requested = engine
                .shared
                .requested_buffer_frames
                .load(Ordering::Acquire);
            if requested != 0 {
                let frames = negotiated_buffer_frames(requested, supported.buffer_size());
                config.buffer_size = cpal::BufferSize::Fixed(frames);
                engine.shared.buffer_frames.store(frames, Ordering::Release);
                engine
                    .shared
                    .requested_buffer_frames
                    .store(frames, Ordering::Release);
            }
            // CPAL does not expose the native internal device rate/period. The
            // sink's existing exact-format contract keeps engine and stream rates
            // equal. Observe its first actual callback before mixing, rather than
            // claim access to miniaudio's private device negotiation/reopen path.
            let scratch_frames = engine.buffer_frames() as usize;
            let output_failed = Arc::new(AtomicBool::new(false));
            let stream = match supported.sample_format() {
                SampleFormat::I8 => build_stream::<i8>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::I16 => build_stream::<i16>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::I24 => build_stream::<cpal::I24>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::I32 => build_stream::<i32>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::I64 => build_stream::<i64>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::U8 => build_stream::<u8>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::U16 => build_stream::<u16>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::U32 => build_stream::<u32>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::U64 => build_stream::<u64>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::F32 => build_stream::<f32>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                SampleFormat::F64 => build_stream::<f64>(
                    &device,
                    &config,
                    engine.clone(),
                    scratch_frames,
                    Arc::clone(&output_failed),
                ),
                format => return Err(AudioDeviceError::UnsupportedSampleFormat(format)),
            }
            .map_err(AudioDeviceError::BuildStream)?;
            stream.play().map_err(AudioDeviceError::PlayStream)?;
            Ok(Self {
                engine,
                _stream: stream,
                output_failed,
            })
        }
    }

    fn negotiated_buffer_frames(requested: u32, supported: &SupportedBufferSize) -> u32 {
        if requested == 0 {
            return 0;
        }
        match supported {
            SupportedBufferSize::Range { min, max } => requested.clamp(*min, *max),
            SupportedBufferSize::Unknown => requested,
        }
    }

    fn observe_device_period(engine: &AudioEngine, frames: u32) {
        if frames != 0 {
            engine.shared.buffer_frames.store(frames, Ordering::Release);
            engine
                .shared
                .requested_buffer_frames
                .store(frames, Ordering::Release);
        }
    }

    fn default_output() -> Result<(cpal::Device, cpal::SupportedStreamConfig), AudioDeviceError> {
        let device = cpal::default_host()
            .default_output_device()
            .ok_or(AudioDeviceError::NoDefaultOutputDevice)?;
        let supported = device
            .default_output_config()
            .map_err(AudioDeviceError::DefaultOutputConfig)?;
        Ok((device, supported))
    }

    fn build_stream<T>(
        device: &cpal::Device,
        config: &cpal::StreamConfig,
        engine: AudioEngine,
        scratch_frames: usize,
        output_failed: Arc<AtomicBool>,
    ) -> Result<cpal::Stream, cpal::BuildStreamError>
    where
        T: SizedSample + FromSample<f32>,
    {
        let scratch_samples = scratch_frames
            .saturating_mul(config.channels as usize)
            .max(config.channels as usize);
        let mut mixed = vec![0.0; scratch_samples];
        let mut first_callback = true;
        device.build_output_stream(
            config,
            move |output: &mut [T], _| {
                if first_callback && !output.is_empty() {
                    let actual = (output.len() / engine.channels() as usize) as u32;
                    observe_device_period(&engine, actual);
                    first_callback = false;
                }
                for output_chunk in output.chunks_mut(mixed.len()) {
                    write_mixed_output(output_chunk, &engine, &mut mixed[..output_chunk.len()]);
                }
            },
            move |_| output_failed.store(true, Ordering::Release),
            None,
        )
    }

    fn write_mixed_output<T>(output: &mut [T], engine: &AudioEngine, mixed: &mut [f32])
    where
        T: Sample + FromSample<f32>,
    {
        debug_assert_eq!(output.len(), mixed.len());
        // Upstream start/stop controls the output device, not explicit manual
        // pulls. CPAL stays allocated, but a stopped sink does not pull or move
        // the graph clock.
        if !super::lock(&engine.shared.state).running || engine.read_audio_frames(mixed) == 0 {
            mixed.fill(0.0);
        }
        for (output, sample) in output.iter_mut().zip(mixed.iter().copied()) {
            *output = T::from_sample(sample.clamp(-1.0, 1.0));
        }
    }

    #[cfg(test)]
    mod tests {
        use std::sync::Arc;

        use crate::AudioSource;

        use super::*;

        #[test]
        fn device_buffer_defaults_clamps_and_observes_actual_period() {
            let _serial = super::super::lock(&super::super::AUDIO_TEST_LOCK);
            let range = SupportedBufferSize::Range {
                min: 128,
                max: 1024,
            };
            assert_eq!(negotiated_buffer_frames(0, &range), 0);
            assert_eq!(negotiated_buffer_frames(64, &range), 128);
            assert_eq!(negotiated_buffer_frames(512, &range), 512);
            assert_eq!(negotiated_buffer_frames(4096, &range), 1024);
            assert_eq!(
                negotiated_buffer_frames(2048, &SupportedBufferSize::Unknown),
                2048
            );
            let engine = AudioEngine::new(2, 48000).unwrap();
            observe_device_period(&engine, 480);
            assert_eq!(engine.buffer_frames(), 480);
            assert_eq!(
                engine
                    .shared
                    .requested_buffer_frames
                    .load(Ordering::Acquire),
                480
            );
            observe_device_period(&engine, 0);
            assert_eq!(engine.buffer_frames(), 480);
        }

        #[test]
        fn device_callback_drains_the_headless_mixer_and_converts_samples() {
            let _serial = super::super::lock(&super::super::AUDIO_TEST_LOCK);
            let engine = AudioEngine::new(1, 4).expect("engine");
            engine.shared.buffer_frames.store(1, Ordering::Release);
            let source = Arc::new(
                AudioSource::from_buffered(vec![-2.0, -0.5, 0.5, 2.0], 1, 4)
                    .expect("buffered source"),
            );
            engine.play(source, 0, 0, 0, None).expect("sound");

            let mut output = [0_i16; 4];
            let mut mixed = [0.0; 4];
            write_mixed_output(&mut output, &engine, &mut mixed);

            assert_eq!(output, [i16::MIN, -16_384, 16_384, i16::MAX]);
            assert_eq!(engine.time_in_frames(), 4);
        }

        /// Hardware-dependent by design: ordinary and differential coverage uses
        /// `write_mixed_output`/`read_audio_frames` and never opens a device.
        #[test]
        #[ignore = "requires a host default audio output device"]
        fn default_output_device_smoke() {
            let sink = AudioDeviceSink::open_default().expect("default output sink");
            assert!(sink.engine().channels() > 0);
            assert!(sink.engine().sample_rate() > 0);
        }
    }
}

#[cfg(feature = "audio-device")]
pub use device::{AudioDeviceError, AudioDeviceSink};
