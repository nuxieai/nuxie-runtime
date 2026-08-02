//! Pure-Rust audio decoding and headless playback for the Rive runtime port.
//!
//! Symphonia replaces the pinned build's miniaudio decoders. The public
//! scheduling, clipping, control, completion, volume, and frame-clock
//! contracts remain exact. Decoder PCM values are not byte-exact contracts,
//! and resampled reader lengths may differ from miniaudio by up to two frames
//! under approved adaptation D17.

mod audio_engine;
mod source;

pub use audio_engine::{AudioArtboardId, AudioEngine, AudioEngineError, AudioSound};
pub use source::{AudioDecodeError, AudioFormat, AudioReader, AudioSource};

#[cfg(feature = "audio-device")]
pub use audio_engine::{AudioDeviceError, AudioDeviceSink};
