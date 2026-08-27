use crate::mechanical_port::source::audio::{
    audio_sound::{AudioSound, AudioSoundRef},
    audio_source::AudioSource,
};
use std::{cell::RefCell, rc::Rc};
pub type AudioEngineRef = Rc<RefCell<AudioEngine>>;
pub const DEFAULT_NUM_CHANNELS: u32 = 2;
pub const DEFAULT_SAMPLE_RATE: u32 = 48_000;
pub struct AudioEngine {
    channels: u32,
    sample_rate: u32,
    running: bool,
    time_frames: u64,
    playing: Vec<AudioSoundRef>,
    completed: Vec<AudioSoundRef>,
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
    pub fn stop_all(&mut self) {
        self.running = false;
        for s in &self.playing {
            s.borrow_mut().stop(0)
        }
        self.remove_disposed()
    }
    pub fn stop_artboard(&mut self, artboard: usize) {
        for s in &self.playing {
            if s.borrow().artboard == Some(artboard) {
                s.borrow_mut().stop(0)
            }
        }
        self.remove_disposed()
    }
    pub fn play(
        engine: &AudioEngineRef,
        source: Rc<AudioSource>,
        start: u64,
        end: u64,
        sound_start: u64,
        artboard: Option<usize>,
    ) -> AudioSoundRef {
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
        sound.borrow_mut().play();
        engine.borrow_mut().playing.insert(0, sound.clone());
        sound
    }
    pub fn play_seconds(
        engine: &AudioEngineRef,
        source: Rc<AudioSource>,
        start: f32,
        end: u64,
        sound_start: u64,
        artboard: Option<usize>,
    ) -> AudioSoundRef {
        let frame = (start * source.sample_rate() as f32) as u64;
        Self::play(engine, source, frame, end, sound_start, artboard)
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
    pub fn playing_sound_count(&self) -> usize {
        self.playing.len()
    }
    pub fn playing_sounds_head(&self) -> Option<AudioSoundRef> {
        self.playing.first().cloned()
    }
    pub fn levels(&self, out: &mut [f32]) {
        for (o, v) in out.iter_mut().zip(&self.levels) {
            *o = *v
        }
    }
    pub fn level(&self, c: u32) -> f32 {
        self.levels.get(c as usize).copied().unwrap_or(0.0)
    }
    pub fn sum_audio_frames(&mut self, frames: &mut [f32], num_frames: u64) -> bool {
        frames.fill(0.0);
        self.advance(num_frames);
        true
    }
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
}
impl Drop for AudioEngine {
    fn drop(&mut self) {
        self.stop_all();
    }
}
