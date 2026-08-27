use crate::mechanical_port::source::audio::{audio_engine::AudioEngine, audio_source::AudioSource};
use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};
pub type AudioSoundRef = Rc<RefCell<AudioSound>>;
pub struct AudioSound {
    pub source: Rc<AudioSource>,
    engine: Weak<RefCell<AudioEngine>>,
    pub artboard: Option<usize>,
    disposed: bool,
    playing: bool,
    completed: bool,
    volume: f32,
    cursor: u64,
    start_frame: u64,
    end_frame: u64,
    sound_start_frame: u64,
}
impl AudioSound {
    pub(crate) fn new(
        engine: &Rc<RefCell<AudioEngine>>,
        source: Rc<AudioSource>,
        start: u64,
        end: u64,
        sound_start: u64,
        artboard: Option<usize>,
    ) -> AudioSoundRef {
        Rc::new(RefCell::new(Self {
            source,
            engine: Rc::downgrade(engine),
            artboard,
            disposed: false,
            playing: false,
            completed: false,
            volume: 1.0,
            cursor: start,
            start_frame: start,
            end_frame: end,
            sound_start_frame: sound_start,
        }))
    }
    pub fn dispose(&mut self) {
        if self.disposed {
            return;
        }
        self.disposed = true;
        self.playing = false;
    }
    pub fn volume(&self) -> f32 {
        self.volume
    }
    pub fn set_volume(&mut self, v: f32) {
        self.volume = v
    }
    pub fn completed(&self) -> bool {
        self.completed
    }
    pub fn stop(&mut self, _fade_frames: u64) {
        self.playing = false;
        self.completed = true
    }
    pub fn play(&mut self) {
        if !self.disposed {
            self.playing = true;
            self.completed = false
        }
    }
    pub fn pause(&mut self) {
        self.playing = false
    }
    pub fn resume(&mut self) {
        if !self.disposed && !self.completed {
            self.playing = true
        }
    }
    pub fn seek(&mut self, frame: u64) -> bool {
        if frame > self.end_frame {
            return false;
        }
        self.cursor = frame;
        true
    }
    pub fn seek_seconds(&mut self, t: f32) -> bool {
        self.seek((t * self.source.sample_rate() as f32) as u64)
    }
    pub fn time_in_frames(&self) -> u64 {
        self.cursor
    }
    pub fn time_in_seconds(&self) -> f32 {
        if self.source.sample_rate() == 0 {
            0.0
        } else {
            self.cursor as f32 / self.source.sample_rate() as f32
        }
    }
    pub(crate) fn advance(&mut self, n: u64) {
        if self.playing {
            self.cursor = self.cursor.saturating_add(n);
            if self.cursor >= self.end_frame {
                self.cursor = self.end_frame;
                self.completed = true;
                self.playing = false
            }
        }
    }
    pub fn clip(&self) -> (u64, u64, u64) {
        (self.start_frame, self.end_frame, self.sound_start_frame)
    }
}
impl Drop for AudioSound {
    fn drop(&mut self) {
        self.dispose();
        if let Some(engine) = self.engine.upgrade() {
            engine.borrow_mut().remove_disposed();
        }
    }
}
