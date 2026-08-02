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
            Self::NoDefaultOutputDevice => formatter.write_str("no default audio output device"),
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
            Self::BuildStream(error) => write!(formatter, "failed to build audio output: {error}"),
            Self::PlayStream(error) => write!(formatter, "failed to start audio output: {error}"),
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
        let config = supported.config();
        let scratch_frames = match supported.buffer_size() {
            SupportedBufferSize::Range { max, .. } => *max as usize,
            // Unknown-size backends are drained in fixed chunks, so this is a
            // latency/memory choice rather than a maximum callback size.
            SupportedBufferSize::Unknown => 4_096,
        };
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
    device.build_output_stream(
        config,
        move |output: &mut [T], _| {
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
    if engine.read_audio_frames(mixed) == 0 {
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
    fn device_callback_drains_the_headless_mixer_and_converts_samples() {
        let engine = AudioEngine::new(1, 4).expect("engine");
        let source = Arc::new(
            AudioSource::from_buffered(vec![-2.0, -0.5, 0.5, 2.0], 1, 4).expect("buffered source"),
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
