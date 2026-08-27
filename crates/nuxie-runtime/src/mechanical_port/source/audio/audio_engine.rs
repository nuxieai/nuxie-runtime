use crate::mechanical_port::source::audio::{
    audio_sound::{AudioSound, AudioSoundRef},
    audio_source::AudioSource,
};
use std::{cell::RefCell, rc::Rc};
pub type AudioEngineRef = Rc<RefCell<AudioEngine>>;
pub const DEFAULT_NUM_CHANNELS: u32 = 2;
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;
thread_local! {
    static RUNTIME_AUDIO_ENGINE: RefCell<Option<AudioEngineRef>> = const { RefCell::new(None) };
}
pub struct AudioEngine {
    channels: u32,
    sample_rate: u32,
    running: bool,
    time_frames: u64,
    playing: Vec<AudioSoundRef>,
    completed: Vec<AudioSoundRef>,
    #[cfg(feature = "rive_audio_tools")]
    levels: Vec<f32>,
}
impl AudioEngine {
    pub fn make(channels: u32, sample_rate: u32) -> Option<AudioEngineRef> {
        if channels == 0 || sample_rate == 0 {
            return None;
        }
        Some(Rc::new(RefCell::new(Self {
            channels,
            sample_rate,
            running: false,
            time_frames: 0,
            playing: Vec::new(),
            completed: Vec::new(),
            #[cfg(feature = "rive_audio_tools")]
            levels: vec![0.0; channels as usize],
        })))
    }
    pub fn channels(&self) -> u32 {
        self.channels
    }
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub fn time_in_frames(&self) -> u64 {
        self.time_frames
    }
    pub fn time_in_seconds(&self) -> f32 {
        self.time_frames as f32 / self.sample_rate as f32
    }
    pub fn start(&mut self) {
        self.running = true
    }
    pub fn stop(&mut self) {
        self.running = false
    }
    pub fn stop_all(&mut self) {
        for sound in self.playing.drain(..) {
            sound.borrow_mut().stop(0);
            self.completed.push(sound);
        }
    }
    pub fn stop_artboard(&mut self, artboard: usize) {
        let mut playing = Vec::with_capacity(self.playing.len());
        for sound in self.playing.drain(..) {
            if sound.borrow().artboard == Some(artboard) {
                sound.borrow_mut().stop(0);
                self.completed.push(sound);
            } else {
                playing.push(sound);
            }
        }
        self.playing = playing;
    }
    pub fn play(
        engine: &AudioEngineRef,
        source: Rc<AudioSource>,
        start: u64,
        end: u64,
        sound_start: u64,
        artboard: Option<usize>,
    ) -> Option<AudioSoundRef> {
        if end != 0 && start >= end {
            return None;
        }
        Self::internal_play(engine, source, start, end, sound_start, artboard)
    }
    fn internal_play(
        engine: &AudioEngineRef,
        source: Rc<AudioSource>,
        start: u64,
        end: u64,
        sound_start: u64,
        artboard: Option<usize>,
    ) -> Option<AudioSoundRef> {
        for sound in engine.borrow_mut().completed.drain(..) {
            sound.borrow_mut().dispose();
        }
        let end = if end == 0 {
            if source.sample_rate() == 0 {
                u64::MAX
            } else {
                (source.duration() * source.sample_rate() as f32) as u64
            }
        } else {
            end
        };
        let sound = AudioSound::new(engine, source, start, end, sound_start, artboard);
        sound.borrow_mut().start_internal();
        engine.borrow_mut().playing.insert(0, sound.clone());
        Some(sound)
    }
    pub fn play_seconds(
        engine: &AudioEngineRef,
        source: Rc<AudioSource>,
        start: f32,
        end: u64,
        sound_start: u64,
        artboard: Option<usize>,
    ) -> Option<AudioSoundRef> {
        if end != 0 && start >= end as f32 {
            return None;
        }
        let frame = (start * engine.borrow().sample_rate() as f32) as u64;
        Self::internal_play(engine, source, frame, end, sound_start, artboard)
    }
    pub fn advance(&mut self, frames: u64) {
        if !self.running {
            return;
        }
        self.time_frames = self.time_frames.saturating_add(frames);
        for s in &self.playing {
            s.borrow_mut().advance(frames)
        }
        self.remove_disposed()
    }
    pub(crate) fn remove_disposed(&mut self) {
        let mut keep = Vec::new();
        for s in self.playing.drain(..) {
            if s.borrow().completed() {
                self.completed.push(s)
            } else {
                keep.push(s)
            }
        }
        self.playing = keep;
    }
    #[cfg(feature = "testing")]
    pub fn playing_sound_count(&self) -> usize {
        self.playing.len()
    }
    #[cfg(feature = "testing")]
    pub fn playing_sounds_head(&self) -> Option<AudioSoundRef> {
        self.playing.first().cloned()
    }
    #[cfg(feature = "rive_audio_tools")]
    pub fn measure_levels(&mut self, frames: &[f32], frame_count: u32) {
        let mut samples = frames.iter().copied();
        for _ in 0..frame_count {
            for channel in 0..self.channels as usize {
                let Some(sample) = samples.next() else {
                    return;
                };
                self.levels[channel] = self.levels[channel].max(sample);
            }
        }
    }
    #[cfg(feature = "rive_audio_tools")]
    pub fn levels(&mut self, out: &mut [f32]) {
        for (o, v) in out.iter_mut().zip(&mut self.levels) {
            *o = *v;
            *v = 0.0;
        }
    }
    #[cfg(feature = "rive_audio_tools")]
    pub fn level(&mut self, c: u32) -> f32 {
        let Some(level) = self.levels.get_mut(c as usize) else {
            return 0.0;
        };
        let value = *level;
        *level = 0.0;
        value
    }
    #[cfg(feature = "external_rive_audio_engine")]
    pub fn sum_audio_frames(&mut self, frames: &mut [f32], num_frames: u64) -> bool {
        frames.fill(0.0);
        self.advance(num_frames);
        true
    }
    #[cfg(feature = "external_rive_audio_engine")]
    pub fn read_audio_frames(
        &mut self,
        frames: &mut [f32],
        num_frames: u64,
        frames_read: Option<&mut u64>,
    ) -> bool {
        let ok = self.sum_audio_frames(frames, num_frames);
        if let Some(out) = frames_read {
            *out = if ok { num_frames } else { 0 }
        }
        ok
    }
    pub fn make_and_store(channels: u32, sample_rate: u32) -> Option<AudioEngineRef> {
        let engine = Self::make(channels, sample_rate)?;
        RUNTIME_AUDIO_ENGINE.with(|runtime| {
            #[cfg(feature = "rive_tools")]
            if let Some(previous) = runtime.borrow().as_ref() {
                previous.borrow_mut().stop_all();
            }
            *runtime.borrow_mut() = Some(engine.clone());
        });
        Some(engine)
    }
    pub fn runtime_engine(make_when_necessary: bool) -> Option<AudioEngineRef> {
        RUNTIME_AUDIO_ENGINE.with(|runtime| {
            if make_when_necessary && runtime.borrow().is_none() {
                *runtime.borrow_mut() = Self::make(DEFAULT_NUM_CHANNELS, DEFAULT_SAMPLE_RATE);
            }
            runtime.borrow().clone()
        })
    }
}
impl Drop for AudioEngine {
    fn drop(&mut self) {
        for sound in self.playing.drain(..) {
            sound.borrow_mut().dispose();
        }
        for sound in self.completed.drain(..) {
            sound.borrow_mut().dispose();
        }
    }
}
