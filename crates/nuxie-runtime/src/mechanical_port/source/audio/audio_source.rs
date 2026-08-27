use crate::mechanical_port::source::audio::{audio_format::AudioFormat, audio_reader::AudioReader};
use std::rc::Rc;
#[derive(Clone)]
pub enum AudioBacking {
    Encoded(Rc<[u8]>),
    Borrowed(*const u8, usize),
    Buffered(Rc<[f32]>),
}
#[derive(Clone)]
pub struct AudioSource {
    backing: AudioBacking,
    channels: u32,
    sample_rate: u32,
    duration: f32,
    format: AudioFormat,
}
impl AudioSource {
    pub fn make_audio_source(bytes: Rc<[u8]>) -> Option<Rc<Self>> {
        let format = detect_format(&bytes);
        if format == AudioFormat::Unknown {
            return None;
        }
        Some(Rc::new(Self {
            backing: AudioBacking::Encoded(bytes),
            channels: 0,
            sample_rate: 0,
            duration: 0.0,
            format,
        }))
    }
    pub fn from_borrowed(bytes: &mut [u8]) -> Self {
        let format = detect_format(bytes);
        Self {
            backing: AudioBacking::Borrowed(bytes.as_ptr(), bytes.len()),
            channels: 0,
            sample_rate: 0,
            duration: 0.0,
            format,
        }
    }
    pub fn buffered(samples: Rc<[f32]>, channels: u32, sample_rate: u32) -> Self {
        let duration = if channels == 0 || sample_rate == 0 {
            0.0
        } else {
            samples.len() as f32 / channels as f32 / sample_rate as f32
        };
        Self {
            backing: AudioBacking::Buffered(samples),
            channels,
            sample_rate,
            duration,
            format: AudioFormat::Buffered,
        }
    }
    pub fn channels(&self) -> u32 {
        self.channels
    }
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    pub fn duration(&self) -> f32 {
        self.duration
    }
    pub fn format(&self) -> AudioFormat {
        self.format
    }
    pub fn is_buffered(&self) -> bool {
        matches!(self.backing, AudioBacking::Buffered(_))
    }
    pub fn bytes(&self) -> &[u8] {
        match &self.backing {
            AudioBacking::Encoded(v) => v,
            AudioBacking::Borrowed(p, n) => unsafe { core::slice::from_raw_parts(*p, *n) },
            AudioBacking::Buffered(_) => &[],
        }
    }
    pub fn buffered_samples(&self) -> &[f32] {
        match &self.backing {
            AudioBacking::Buffered(v) => v,
            _ => &[],
        }
    }
    pub fn make_reader(self: &Rc<Self>, channels: u32, sample_rate: u32) -> AudioReader {
        AudioReader::new(self.clone(), channels, sample_rate)
    }
}
fn detect_format(b: &[u8]) -> AudioFormat {
    if b.starts_with(b"RIFF") && b.get(8..12) == Some(b"WAVE") {
        AudioFormat::Wav
    } else if b.starts_with(b"fLaC") {
        AudioFormat::Flac
    } else if b.starts_with(b"OggS") {
        AudioFormat::Vorbis
    } else if b.starts_with(b"ID3") || b.first() == Some(&0xff) {
        AudioFormat::Mp3
    } else {
        AudioFormat::Unknown
    }
}
