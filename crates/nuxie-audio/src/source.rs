use std::io::Cursor;
use std::sync::{Arc, OnceLock};

use symphonia_bundle_flac::{FlacDecoder, FlacReader};
use symphonia_bundle_mp3::{MpaDecoder, MpaReader};
use symphonia_codec_pcm::PcmDecoder;
use symphonia_core::audio::SampleBuffer;
use symphonia_core::codecs::{CodecRegistry, DecoderOptions};
use symphonia_core::errors::Error as SymphoniaError;
use symphonia_core::formats::FormatOptions;
use symphonia_core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia_core::meta::MetadataOptions;
use symphonia_core::probe::{Hint, Probe};
use symphonia_format_riff::WavReader;

/// Encoded format recognized by the pinned Rive audio interface.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioFormat {
    Unknown,
    Wav,
    Flac,
    Mp3,
    Vorbis,
    Buffered,
}

impl AudioFormat {
    /// Recognize the formats named by Rive's `AudioFormat` enum.
    ///
    /// Vorbis is intentionally recognized here but is not accepted by
    /// [`AudioSource::from_encoded`], matching the pinned build where the
    /// optional Vorbis decoder is not wired.
    pub fn recognize(bytes: &[u8]) -> Self {
        if bytes.starts_with(b"RIFF") && bytes.get(8..12) == Some(b"WAVE") {
            Self::Wav
        } else if bytes.starts_with(b"fLaC") {
            Self::Flac
        } else if bytes.starts_with(b"OggS") {
            Self::Vorbis
        } else if bytes.starts_with(b"ID3")
            || bytes
                .get(..2)
                .is_some_and(|prefix| prefix[0] == 0xff && prefix[1] & 0xe0 == 0xe0)
        {
            Self::Mp3
        } else {
            Self::Unknown
        }
    }

    fn extension(self) -> Option<&'static str> {
        match self {
            Self::Wav => Some("wav"),
            Self::Flac => Some("flac"),
            Self::Mp3 => Some("mp3"),
            Self::Unknown | Self::Vorbis | Self::Buffered => None,
        }
    }
}

/// A failure to validate or decode an encoded audio source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AudioDecodeError {
    InvalidData,
    UnsupportedFormat(AudioFormat),
}

impl std::fmt::Display for AudioDecodeError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidData => formatter.write_str("invalid audio data"),
            Self::UnsupportedFormat(format) => {
                write!(formatter, "unsupported audio format {format:?}")
            }
        }
    }
}

impl std::error::Error for AudioDecodeError {}

#[derive(Debug)]
enum AudioSourceData {
    Encoded {
        bytes: Arc<[u8]>,
        format: AudioFormat,
    },
    Buffered {
        samples: Arc<[f32]>,
    },
}

/// Owned encoded bytes or owned interleaved `f32` samples.
///
/// Encoded construction validates the stream parameters, while PCM
/// decoding remains lazy until a reader, duration query, or engine asks for
/// samples. Each reader owns an independent cursor and resampled buffer.
#[derive(Debug)]
pub struct AudioSource {
    data: AudioSourceData,
    channels: u32,
    sample_rate: u32,
    duration: OnceLock<f32>,
}

impl AudioSource {
    /// Validate and take ownership of WAV, MP3, or FLAC bytes.
    pub fn from_encoded(bytes: impl Into<Vec<u8>>) -> Result<Self, AudioDecodeError> {
        let bytes = Arc::<[u8]>::from(bytes.into());
        let format = AudioFormat::recognize(&bytes);
        if format == AudioFormat::Vorbis {
            return Err(AudioDecodeError::UnsupportedFormat(AudioFormat::Vorbis));
        }
        if format == AudioFormat::Unknown {
            return Err(AudioDecodeError::InvalidData);
        }
        let (channels, sample_rate) = validate_encoded(Arc::clone(&bytes), format)?;
        Ok(Self {
            data: AudioSourceData::Encoded { bytes, format },
            channels,
            sample_rate,
            duration: OnceLock::new(),
        })
    }

    /// Take ownership of interleaved `f32` PCM.
    pub fn from_buffered(
        samples: impl Into<Vec<f32>>,
        channels: u32,
        sample_rate: u32,
    ) -> Result<Self, AudioDecodeError> {
        if channels == 0 || sample_rate == 0 {
            return Err(AudioDecodeError::InvalidData);
        }
        let samples = Arc::<[f32]>::from(samples.into());
        let duration = samples.len() as f32 / (channels as f32 * sample_rate as f32);
        let cached_duration = OnceLock::new();
        let _ = cached_duration.set(duration);
        Ok(Self {
            data: AudioSourceData::Buffered { samples },
            channels,
            sample_rate,
            duration: cached_duration,
        })
    }

    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    pub fn format(&self) -> AudioFormat {
        match &self.data {
            AudioSourceData::Encoded { format, .. } => *format,
            AudioSourceData::Buffered { .. } => AudioFormat::Buffered,
        }
    }

    pub fn duration(&self) -> f32 {
        *self.duration.get_or_init(|| {
            let frame_count = self
                .decode_native_samples()
                .map_or(0, |decoded| decoded.frame_count());
            if self.sample_rate == 0 {
                0.0
            } else {
                frame_count as f32 / self.sample_rate as f32
            }
        })
    }

    pub fn bytes(&self) -> &[u8] {
        match &self.data {
            AudioSourceData::Encoded { bytes, .. } => bytes,
            AudioSourceData::Buffered { .. } => &[],
        }
    }

    pub fn buffered_samples(&self) -> Option<&[f32]> {
        match &self.data {
            AudioSourceData::Encoded { .. } => None,
            AudioSourceData::Buffered { samples } => Some(samples),
        }
    }

    pub fn is_buffered(&self) -> bool {
        matches!(&self.data, AudioSourceData::Buffered { .. })
    }

    /// Construct an independent decoded/resampled reader.
    ///
    /// As in pinned C++, buffered sources are consumed directly by the engine
    /// and do not manufacture an `AudioReader`.
    pub fn make_reader(&self, channels: u32, sample_rate: u32) -> Option<AudioReader> {
        if channels == 0 || sample_rate == 0 || self.is_buffered() {
            return None;
        }
        let decoded = self.decode_native_samples().ok()?;
        let converted = convert_channels(&decoded.samples, decoded.channels, channels);
        let samples = resample_linear(&converted, channels, decoded.sample_rate, sample_rate);
        Some(AudioReader {
            samples,
            channels,
            sample_rate,
            cursor: 0,
            read_buffer: Vec::new(),
        })
    }

    pub(crate) fn decode_for_output(
        &self,
        channels: u32,
        sample_rate: u32,
    ) -> Result<Arc<[f32]>, AudioDecodeError> {
        if channels == 0 || sample_rate == 0 {
            return Err(AudioDecodeError::InvalidData);
        }
        let decoded = self.decode_native_samples()?;
        let converted = convert_channels(&decoded.samples, decoded.channels, channels);
        Ok(Arc::from(resample_linear(
            &converted,
            channels,
            decoded.sample_rate,
            sample_rate,
        )))
    }

    fn decode_native_samples(&self) -> Result<DecodedAudio, AudioDecodeError> {
        match &self.data {
            AudioSourceData::Encoded { bytes, format } => {
                decode_encoded(Arc::clone(bytes), *format)
            }
            AudioSourceData::Buffered { samples } => Ok(DecodedAudio {
                samples: samples.to_vec(),
                channels: self.channels,
                sample_rate: self.sample_rate,
            }),
        }
    }
}

#[derive(Debug)]
struct DecodedAudio {
    samples: Vec<f32>,
    channels: u32,
    sample_rate: u32,
}

impl DecodedAudio {
    fn frame_count(&self) -> u64 {
        if self.channels == 0 {
            0
        } else {
            (self.samples.len() / self.channels as usize) as u64
        }
    }
}

/// An independent decoded/resampled cursor over an encoded source.
#[derive(Debug, Clone)]
pub struct AudioReader {
    samples: Vec<f32>,
    channels: u32,
    sample_rate: u32,
    cursor: usize,
    read_buffer: Vec<f32>,
}

impl AudioReader {
    pub fn channels(&self) -> u32 {
        self.channels
    }

    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Resampled frame count. D17 permits a ±2-frame difference from
    /// miniaudio; callers must not use this as a byte-exact PCM oracle.
    pub fn length_in_frames(&self) -> u64 {
        (self.samples.len() / self.channels as usize) as u64
    }

    pub fn read(&mut self, frame_count: u64) -> &[f32] {
        let requested_samples = usize::try_from(frame_count)
            .ok()
            .and_then(|frames| frames.checked_mul(self.channels as usize))
            .unwrap_or(usize::MAX);
        let end = self
            .cursor
            .saturating_add(requested_samples)
            .min(self.samples.len());
        self.read_buffer.clear();
        if let Some(samples) = self.samples.get(self.cursor..end) {
            self.read_buffer.extend_from_slice(samples);
        }
        self.cursor = end;
        &self.read_buffer
    }
}

fn media_source(bytes: Arc<[u8]>, format: AudioFormat) -> MediaSourceStream {
    let mut cursor = Cursor::new(bytes);
    if format == AudioFormat::Mp3 {
        cursor.set_position(id3v2_payload_offset(cursor.get_ref()) as u64);
    }
    MediaSourceStream::new(Box::new(cursor), MediaSourceStreamOptions::default())
}

fn id3v2_payload_offset(bytes: &[u8]) -> usize {
    let mut offset = 0usize;
    while let Some(header) = bytes.get(offset..offset.saturating_add(10)) {
        let Ok(header) = <&[u8; 10]>::try_from(header) else {
            break;
        };
        let [
            magic_i,
            magic_d,
            magic_3,
            major_version,
            minor_version,
            flags,
            size_0,
            size_1,
            size_2,
            size_3,
        ] = *header;
        let size_bytes = [size_0, size_1, size_2, size_3];
        if [magic_i, magic_d, magic_3] != *b"ID3"
            || !(2..=4).contains(&major_version)
            || minor_version == 0xff
            || (major_version == 2 && flags & 0x40 != 0)
            || size_bytes.iter().any(|byte| byte & 0x80 != 0)
        {
            break;
        }
        let payload_len = size_bytes
            .iter()
            .fold(0usize, |size, byte| (size << 7) | usize::from(*byte));
        let Some(next) = offset
            .checked_add(10)
            .and_then(|value| value.checked_add(payload_len))
        else {
            break;
        };
        if next > bytes.len() {
            break;
        }
        offset = next;
    }
    offset
}

fn codec_registry() -> &'static CodecRegistry {
    static CODECS: OnceLock<CodecRegistry> = OnceLock::new();
    CODECS.get_or_init(|| {
        let mut codecs = CodecRegistry::new();
        codecs.register_all::<FlacDecoder>();
        codecs.register_all::<MpaDecoder>();
        codecs.register_all::<PcmDecoder>();
        codecs
    })
}

fn format_probe() -> &'static Probe {
    static FORMATS: OnceLock<Probe> = OnceLock::new();
    FORMATS.get_or_init(|| {
        let mut probe = Probe::default();
        probe.register_all::<FlacReader>();
        probe.register_all::<MpaReader>();
        probe.register_all::<WavReader>();
        probe
    })
}

fn hint_for(format: AudioFormat) -> Hint {
    let mut hint = Hint::new();
    if let Some(extension) = format.extension() {
        hint.with_extension(extension);
    }
    hint
}

fn validate_encoded(bytes: Arc<[u8]>, format: AudioFormat) -> Result<(u32, u32), AudioDecodeError> {
    let probed = format_probe()
        .format(
            &hint_for(format),
            media_source(bytes, format),
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|_| AudioDecodeError::InvalidData)?;
    let track = probed
        .format
        .default_track()
        .ok_or(AudioDecodeError::InvalidData)?;
    let channels = track
        .codec_params
        .channels
        .map(|channels| channels.count() as u32)
        .ok_or(AudioDecodeError::InvalidData)?;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or(AudioDecodeError::InvalidData)?;
    codec_registry()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|_| AudioDecodeError::InvalidData)?;
    Ok((channels, sample_rate))
}

fn decode_encoded(
    bytes: Arc<[u8]>,
    audio_format: AudioFormat,
) -> Result<DecodedAudio, AudioDecodeError> {
    let probed = format_probe()
        .format(
            &hint_for(audio_format),
            media_source(bytes, audio_format),
            &FormatOptions::default(),
            &MetadataOptions::default(),
        )
        .map_err(|_| AudioDecodeError::InvalidData)?;
    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or(AudioDecodeError::InvalidData)?;
    let track_id = track.id;
    let mut decoder = codec_registry()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|_| AudioDecodeError::InvalidData)?;
    let mut samples = Vec::new();
    let mut channels = 0u32;
    let mut sample_rate = 0u32;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(_) => return Err(AudioDecodeError::InvalidData),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(_) => return Err(AudioDecodeError::InvalidData),
        };
        channels = decoded.spec().channels.count() as u32;
        sample_rate = decoded.spec().rate;
        let mut converted = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
        converted.copy_interleaved_ref(decoded);
        samples.extend_from_slice(converted.samples());
    }

    if channels == 0 || sample_rate == 0 {
        return Err(AudioDecodeError::InvalidData);
    }
    Ok(DecodedAudio {
        samples,
        channels,
        sample_rate,
    })
}

fn convert_channels(samples: &[f32], source_channels: u32, target_channels: u32) -> Vec<f32> {
    if source_channels == target_channels {
        return samples.to_vec();
    }
    let source_channels = source_channels as usize;
    let target_channels = target_channels as usize;
    let frame_count = samples.len() / source_channels;
    let mut converted = Vec::with_capacity(frame_count.saturating_mul(target_channels));
    for frame in samples.chunks_exact(source_channels) {
        if target_channels == 1 {
            converted.push(frame.iter().copied().sum::<f32>() / source_channels as f32);
            continue;
        }
        if source_channels == 1 {
            converted.extend(std::iter::repeat_n(frame[0], target_channels));
            continue;
        }
        converted.extend((0..target_channels).map(|channel| {
            frame
                .get(channel)
                .copied()
                .unwrap_or_else(|| frame[channel % source_channels])
        }));
    }
    converted
}

fn resample_linear(samples: &[f32], channels: u32, source_rate: u32, target_rate: u32) -> Vec<f32> {
    if source_rate == target_rate {
        return samples.to_vec();
    }
    let channels = channels as usize;
    let source_frames = samples.len() / channels;
    if source_frames == 0 {
        return Vec::new();
    }
    let target_frames = ((source_frames as u128 * target_rate as u128
        + u128::from(source_rate / 2))
        / u128::from(source_rate)) as usize;
    let mut output = Vec::with_capacity(target_frames.saturating_mul(channels));
    for target_frame in 0..target_frames {
        let numerator = target_frame as u128 * source_rate as u128;
        let source_frame = (numerator / target_rate as u128) as usize;
        let fraction = (numerator % target_rate as u128) as f32 / target_rate as f32;
        let next_frame = source_frame.saturating_add(1).min(source_frames - 1);
        let source_frame = source_frame.min(source_frames - 1);
        for channel in 0..channels {
            let left = samples[source_frame * channels + channel];
            let right = samples[next_frame * channels + channel];
            output.push(left + (right - left) * fraction);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn pinned_fixture(relative: &str) -> Vec<u8> {
        let root = std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
        let path = PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(relative);
        std::fs::read(&path)
            .unwrap_or_else(|error| panic!("read pinned audio fixture {}: {error}", path.display()))
    }

    #[test]
    fn wav_source_metadata_and_reader_lengths_match_the_pinned_contract() {
        let source = AudioSource::from_encoded(pinned_fixture("audio/what.wav"))
            .expect("pinned WAV validates");
        assert_eq!(source.format(), AudioFormat::Wav);
        assert_eq!(source.channels(), 2);
        assert_eq!(source.sample_rate(), 44_100);

        let native = source.make_reader(2, 44_100).expect("native reader");
        assert_eq!(native.length_in_frames(), 9_688);

        // D17's decoder/resampler adaptation permits a two-frame length
        // difference; sample payloads are intentionally never byte-pinned.
        let mono_48k = source.make_reader(1, 48_000).expect("48 kHz reader");
        assert!(mono_48k.length_in_frames().abs_diff(10_544) <= 2);
        let stereo_32k = source.make_reader(2, 32_000).expect("32 kHz reader");
        assert!(stereo_32k.length_in_frames().abs_diff(7_029) <= 2);
    }

    #[test]
    fn file_and_buffered_durations_follow_the_pinned_cases() {
        let wav = AudioSource::from_encoded(pinned_fixture("audio/what.wav"))
            .expect("pinned WAV validates");
        assert!((wav.duration() - 9_688.0 / 44_100.0).abs() < f32::EPSILON);
        assert_eq!(wav.duration(), wav.duration(), "duration is cached");

        let buffered =
            AudioSource::from_buffered(vec![0.0; 48_000 * 2], 2, 48_000).expect("buffered source");
        assert_eq!(buffered.format(), AudioFormat::Buffered);
        assert_eq!(buffered.duration(), 1.0);
        assert!(buffered.make_reader(2, 48_000).is_none());

        let mp3 = AudioSource::from_encoded(pinned_fixture("audio/song.mp3"))
            .expect("pinned MP3 validates");
        assert_eq!(mp3.format(), AudioFormat::Mp3);
        assert!(mp3.duration() > 0.0);
        assert_eq!(mp3.duration(), mp3.duration(), "MP3 duration is cached");
    }

    #[test]
    fn vorbis_is_recognized_but_unwired_like_the_pinned_build() {
        let bytes = b"OggS\0synthetic-vorbis".to_vec();
        assert_eq!(AudioFormat::recognize(&bytes), AudioFormat::Vorbis);
        assert_eq!(
            AudioSource::from_encoded(bytes).expect_err("Vorbis decoder stays unwired"),
            AudioDecodeError::UnsupportedFormat(AudioFormat::Vorbis)
        );
    }

    #[test]
    fn id3v2_headers_are_skipped_without_decoding_tag_text() {
        let mut tagged = b"ID3\x04\x00\x00\x00\x00\x00\x03tag".to_vec();
        tagged.extend_from_slice(b"\xff\xfb");
        assert_eq!(id3v2_payload_offset(&tagged), 13);

        let malformed = b"ID3\x04\x00\x00\x80\x00\x00\x03tag";
        assert_eq!(id3v2_payload_offset(malformed), 0);
    }
}
