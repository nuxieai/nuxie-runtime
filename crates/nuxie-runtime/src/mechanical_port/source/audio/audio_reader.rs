pub struct AudioReader {
    backend: Option<nuxie_audio::AudioReader>,
    channels: u32,
    sample_rate: u32,
}

impl AudioReader {
    pub(crate) fn new(
        backend: Option<nuxie_audio::AudioReader>,
        channels: u32,
        sample_rate: u32,
    ) -> Self {
        Self {
            backend,
            channels,
            sample_rate,
        }
    }

    pub fn length_in_frames(&self) -> u64 {
        self.backend
            .as_ref()
            .map_or(0, nuxie_audio::AudioReader::length_in_frames)
    }

    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn read(&mut self, frame_count: u64) -> &[f32] {
        self.backend
            .as_mut()
            .map_or(&[], |reader| reader.read(frame_count))
    }
}
