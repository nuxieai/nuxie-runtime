use std::sync::{Arc, Mutex, MutexGuard};

use crate::mechanical_port::source::audio::{
    audio_sound::AudioSoundRef, audio_source::AudioSource,
};

pub type AudioEngineRef = Arc<AudioEngine>;
pub const DEFAULT_NUM_CHANNELS: u32 = nuxie_audio::AudioEngine::DEFAULT_CHANNELS;
pub const DEFAULT_SAMPLE_RATE: u32 = nuxie_audio::AudioEngine::DEFAULT_SAMPLE_RATE;

static RUNTIME_AUDIO_ENGINE: Mutex<Option<AudioEngineRef>> = Mutex::new(None);

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[derive(Debug, Clone)]
pub struct AudioEngine {
    backend: nuxie_audio::AudioEngine,
    #[cfg(feature = "tools")]
    levels: Arc<Mutex<Vec<f32>>>,
}

impl AudioEngine {
    pub fn make(channels: u32, sample_rate: u32) -> Option<AudioEngineRef> {
        Some(Arc::new(Self {
            backend: nuxie_audio::AudioEngine::new(channels, sample_rate).ok()?,
            #[cfg(feature = "tools")]
            levels: Arc::new(Mutex::new(vec![0.0; channels as usize])),
        }))
    }

    pub fn channels(&self) -> u32 {
        self.backend.channels()
    }

    pub fn sample_rate(&self) -> u32 {
        self.backend.sample_rate()
    }

    pub fn time_in_frames(&self) -> u64 {
        self.backend.time_in_frames()
    }

    pub fn time_in_seconds(&self) -> f32 {
        self.backend.time_in_seconds()
    }

    pub fn start(&self) {
        self.backend.start();
    }

    pub fn stop(&self) {
        self.backend.stop();
    }

    pub fn stop_all(&self) {
        self.backend.stop_all_sounds();
    }

    pub fn stop_artboard(&self, artboard: usize) {
        self.backend
            .stop_artboard(nuxie_audio::AudioArtboardId(artboard as u64));
    }

    pub fn play(
        engine: &AudioEngineRef,
        source: Arc<AudioSource>,
        start: u64,
        end: u64,
        sound_start: u64,
        artboard: Option<usize>,
    ) -> Option<AudioSoundRef> {
        engine.backend.play(
            source.backend()?,
            start,
            end,
            sound_start,
            artboard.map(|identity| nuxie_audio::AudioArtboardId(identity as u64)),
        )
    }

    pub fn play_seconds(
        engine: &AudioEngineRef,
        source: Arc<AudioSource>,
        start: f32,
        end: u64,
        sound_start: u64,
        artboard: Option<usize>,
    ) -> Option<AudioSoundRef> {
        engine.backend.play_seconds(
            source.backend()?,
            start,
            end,
            sound_start,
            artboard.map(|identity| nuxie_audio::AudioArtboardId(identity as u64)),
        )
    }

    pub fn advance(&self, frames: u64) {
        let sample_count = usize::try_from(frames)
            .ok()
            .and_then(|frames| frames.checked_mul(self.channels() as usize));
        if let Some(sample_count) = sample_count {
            self.backend.read_audio_frames(&mut vec![0.0; sample_count]);
        }
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn playing_sound_count(&self) -> usize {
        self.backend.playing_sound_count()
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn playing_sounds_head(&self) -> Option<AudioSoundRef> {
        self.backend.playing_sounds_head()
    }

    #[cfg(feature = "tools")]
    pub fn measure_levels(&self, frames: &[f32], frame_count: u32) {
        let channel_count = self.channels() as usize;
        if channel_count == 0 {
            return;
        }
        let mut levels = lock(&self.levels);
        for frame in frames
            .chunks_exact(channel_count)
            .take(frame_count as usize)
        {
            for (level, sample) in levels.iter_mut().zip(frame) {
                *level = level.max(*sample);
            }
        }
    }

    #[cfg(feature = "tools")]
    pub fn levels(&self, out: &mut [f32]) {
        let mut levels = lock(&self.levels);
        for (output, level) in out.iter_mut().zip(levels.iter_mut()) {
            *output = *level;
            *level = 0.0;
        }
    }

    #[cfg(feature = "tools")]
    pub fn level(&self, channel: u32) -> f32 {
        let mut levels = lock(&self.levels);
        let Some(level) = levels.get_mut(channel as usize) else {
            return 0.0;
        };
        let value = *level;
        *level = 0.0;
        value
    }

    pub fn sum_audio_frames(&self, frames: &mut [f32], num_frames: u64) -> bool {
        let expected = usize::try_from(num_frames)
            .ok()
            .and_then(|count| count.checked_mul(self.channels() as usize));
        expected == Some(frames.len()) && self.backend.sum_audio_frames(frames) == num_frames
    }

    pub fn read_audio_frames(
        &self,
        frames: &mut [f32],
        num_frames: u64,
        frames_read: Option<&mut u64>,
    ) -> bool {
        let expected = usize::try_from(num_frames)
            .ok()
            .and_then(|count| count.checked_mul(self.channels() as usize));
        let read = if expected == Some(frames.len()) {
            self.backend.read_audio_frames(frames)
        } else {
            0
        };
        if let Some(frames_read) = frames_read {
            *frames_read = read;
        }
        read == num_frames
    }

    pub fn make_and_store(channels: u32, sample_rate: u32) -> Option<AudioEngineRef> {
        let backend = nuxie_audio::AudioEngine::make_and_store(channels, sample_rate).ok()?;
        let engine = Arc::new(Self {
            backend,
            #[cfg(feature = "tools")]
            levels: Arc::new(Mutex::new(vec![0.0; channels as usize])),
        });
        if let Some(previous) = lock(&RUNTIME_AUDIO_ENGINE).replace(engine.clone()) {
            #[cfg(feature = "tools")]
            previous.stop_all();
            #[cfg(not(feature = "tools"))]
            let _ = previous;
        }
        Some(engine)
    }

    pub fn runtime_engine(make_when_necessary: bool) -> Option<AudioEngineRef> {
        let mut runtime = lock(&RUNTIME_AUDIO_ENGINE);
        if runtime.is_none() && make_when_necessary {
            let backend = nuxie_audio::AudioEngine::runtime_engine();
            #[cfg(feature = "tools")]
            let channels = backend.channels();
            *runtime = Some(Arc::new(Self {
                backend,
                #[cfg(feature = "tools")]
                levels: Arc::new(Mutex::new(vec![0.0; channels as usize])),
            }));
        }
        runtime.clone()
    }
}
