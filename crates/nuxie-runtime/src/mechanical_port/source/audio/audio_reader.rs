use crate::mechanical_port::source::audio::audio_source::AudioSource;
use std::rc::Rc;
pub struct AudioReader {
    source: Rc<AudioSource>,
    channels: u32,
    sample_rate: u32,
    cursor: u64,
    read_buffer: Vec<f32>,
}
impl AudioReader {
    pub(crate) fn new(source: Rc<AudioSource>, channels: u32, sample_rate: u32) -> Self {
        Self {
            source,
            channels,
            sample_rate,
            cursor: 0,
            read_buffer: Vec::new(),
        }
    }
    pub fn length_in_frames(&self) -> u64 {
        if self.channels == 0 {
            0
        } else {
            self.source.buffered_samples().len() as u64 / self.channels as u64
        }
    }
    pub fn channels(&self) -> u32 {
        self.channels
    }
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub fn read(&mut self, frame_count: u64) -> &[f32] {
        let count = frame_count.saturating_mul(self.channels as u64) as usize;
        let start = self.cursor.saturating_mul(self.channels as u64) as usize;
        let samples = self.source.buffered_samples();
        let end = (start + count).min(samples.len());
        self.read_buffer.clear();
        self.read_buffer
            .extend_from_slice(&samples[start.min(end)..end]);
        self.cursor += ((end - start) as u64) / (self.channels.max(1) as u64);
        &self.read_buffer
    }
}
