use std::sync::atomic::AtomicBool;

pub struct AudioReader {
    backend: Option<nuxie_audio::AudioReader>,
}

impl AudioReader {
    pub(crate) fn new(backend: Option<nuxie_audio::AudioReader>) -> Self {
        Self { backend }
    }
    pub fn is_valid(&self) -> bool {
        self.backend.is_some()
    }
    pub fn length_in_frames(&self) -> u64 {
        self.backend
            .as_ref()
            .map_or(0, nuxie_audio::AudioReader::length_in_frames)
    }
    pub fn channels(&self) -> u32 {
        self.backend
            .as_ref()
            .map_or(0, nuxie_audio::AudioReader::channels)
    }
    pub fn sample_rate(&self) -> u32 {
        self.backend
            .as_ref()
            .map_or(0, nuxie_audio::AudioReader::sample_rate)
    }
    pub fn read(&mut self, frame_count: u64) -> &[f32] {
        self.backend
            .as_mut()
            .map_or(&[], |reader| reader.read(frame_count))
    }
    pub fn read_into(&mut self, out: &mut [f32]) -> Result<u64, nuxie_audio::AudioReadError> {
        self.backend
            .as_mut()
            .ok_or(nuxie_audio::AudioReadError {
                frames_read: 0,
                error: nuxie_audio::AudioDecodeError::InvalidData,
            })?
            .read_into(out)
    }
    pub fn seek(&mut self, frame: u64) -> Option<u64> {
        self.backend.as_mut()?.seek(frame)
    }
    pub fn rewind(&mut self) -> bool {
        self.backend.as_mut().is_some_and(|reader| reader.rewind())
    }
    pub fn read_all(
        &mut self,
        max_frames: u64,
        out: &mut Vec<f32>,
        stop: Option<&AtomicBool>,
    ) -> bool {
        match self.backend.as_mut() {
            Some(reader) => reader.read_all(max_frames, out, stop),
            None => {
                out.clear();
                false
            }
        }
    }
}
