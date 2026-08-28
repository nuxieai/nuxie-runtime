use std::sync::Arc;

use crate::mechanical_port::source::{
    audio::{audio_format::AudioFormat, audio_reader::AudioReader},
    simple_array::SimpleArray,
};

pub type AudioSourceRef = Arc<AudioSource>;

#[derive(Debug)]
pub struct AudioSource {
    backend: Option<Arc<nuxie_audio::AudioSource>>,
    encoded_bytes: Arc<[u8]>,
    format: AudioFormat,
}

impl AudioSource {
    pub fn make_audio_source(bytes: SimpleArray<u8>) -> Option<AudioSourceRef> {
        Self::from_encoded(bytes.as_slice())
    }

    pub fn from_encoded(bytes: &[u8]) -> Option<AudioSourceRef> {
        let backend = Arc::new(nuxie_audio::AudioSource::from_encoded(bytes.to_vec()).ok()?);
        Some(Arc::new(Self {
            format: backend.format().into(),
            encoded_bytes: Arc::from(bytes),
            backend: Some(backend),
        }))
    }

    /// Safe-Rust adaptation of the C++ span constructor: the stored source
    /// owns a copy because its lifetime is independent of the import buffer.
    pub fn from_borrowed(bytes: &mut [u8]) -> Self {
        let encoded_bytes = Arc::<[u8]>::from(&*bytes);
        let backend = nuxie_audio::AudioSource::from_encoded(bytes.to_vec())
            .ok()
            .map(Arc::new);
        let format = backend
            .as_ref()
            .map(|source| source.format().into())
            .unwrap_or_else(|| nuxie_audio::AudioFormat::recognize(bytes).into());
        Self {
            backend,
            encoded_bytes,
            format,
        }
    }

    pub fn buffered(samples: Arc<[f32]>, channels: u32, sample_rate: u32) -> Self {
        assert!(channels != 0);
        assert!(sample_rate != 0);
        let backend = nuxie_audio::AudioSource::from_buffered(
            samples.as_ref().to_vec(),
            channels,
            sample_rate,
        )
        .expect("non-zero buffered audio dimensions are valid");
        Self {
            backend: Some(Arc::new(backend)),
            encoded_bytes: Arc::from([]),
            format: AudioFormat::Buffered,
        }
    }

    pub fn channels(&self) -> u32 {
        self.backend.as_ref().map_or(0, |source| source.channels())
    }

    pub fn sample_rate(&self) -> u32 {
        self.backend
            .as_ref()
            .map_or(0, |source| source.sample_rate())
    }

    pub fn duration(&self) -> f32 {
        self.backend
            .as_ref()
            .map_or(0.0, |source| source.duration())
    }

    pub fn format(&self) -> AudioFormat {
        self.format
    }

    pub fn is_buffered(&self) -> bool {
        self.format == AudioFormat::Buffered
    }

    pub fn bytes(&self) -> &[u8] {
        &self.encoded_bytes
    }

    pub fn buffered_samples(&self) -> &[f32] {
        self.backend
            .as_ref()
            .and_then(|source| source.buffered_samples())
            .unwrap_or_default()
    }

    pub fn make_reader(&self, channels: u32, sample_rate: u32) -> AudioReader {
        AudioReader::new(
            self.backend
                .as_ref()
                .and_then(|source| source.make_reader(channels, sample_rate)),
            channels,
            sample_rate,
        )
    }

    pub(crate) fn backend(&self) -> Option<Arc<nuxie_audio::AudioSource>> {
        self.backend.clone()
    }
}
