use std::io::Cursor;
use std::sync::{Arc, OnceLock};

use symphonia_bundle_flac::{FlacDecoder, FlacReader};
use symphonia_bundle_mp3::{MpaDecoder, MpaReader};
use symphonia_codec_pcm::PcmDecoder;
use symphonia_core::audio::SampleBuffer;
use symphonia_core::codecs::{CodecRegistry, Decoder, DecoderOptions};
use symphonia_core::errors::Error as SymphoniaError;
use symphonia_core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
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

/// A decoder failure may follow complete frames already written to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AudioReadError {
    pub frames_read: u64,
    pub error: AudioDecodeError,
}
impl std::fmt::Display for AudioReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.error.fmt(f)
    }
}
impl std::error::Error for AudioReadError {}

fn read_frames(
    out: &mut [f32],
    channels: usize,
    mut next: impl FnMut(&mut [f32]) -> Result<bool, AudioDecodeError>,
) -> Result<u64, AudioReadError> {
    let mut read = 0;
    for frame in out.chunks_exact_mut(channels) {
        match next(frame) {
            Ok(true) => read += 1,
            Ok(false) => break,
            Err(error) => {
                return Err(AudioReadError {
                    frames_read: read,
                    error,
                });
            }
        }
    }
    Ok(read)
}

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
            // The reader's MP3 frame scan corrects a miscounted Xing/Info
            // header, as upstream's decoder seek table does. No whole-source
            // PCM allocation is needed merely to report duration.
            let Some(reader) = self.make_reader(0, 0) else {
                return 0.0;
            };
            if reader.sample_rate() == 0 {
                0.0
            } else {
                reader.length_in_frames() as f32 / reader.sample_rate() as f32
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

    /// Construct a lazy independent reader. Zero format fields keep the source format.
    pub fn make_reader(&self, channels: u32, sample_rate: u32) -> Option<AudioReader> {
        let input = match &self.data {
            AudioSourceData::Buffered { samples } => ReaderInput::Buffered {
                samples: samples.clone(),
                cursor: 0,
            },
            AudioSourceData::Encoded { bytes, format } => {
                ReaderInput::Encoded(EncodedReader::new(bytes.clone(), *format).ok()?)
            }
        };
        Some(AudioReader {
            channels: if channels == 0 {
                self.channels
            } else {
                channels
            },
            sample_rate: if sample_rate == 0 {
                self.sample_rate
            } else {
                sample_rate
            },
            input_channels: self.channels,
            input_rate: self.sample_rate,
            input,
            left: Vec::new(),
            right: Vec::new(),
            has_left: false,
            has_right: false,
            source_position: 0,
            output_position: 0,
            phase: 0,
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

// A decoder owns only its current packet; creating a reader never decodes the asset.
struct EncodedReader {
    format: Box<dyn FormatReader>,
    decoder: Box<dyn Decoder>,
    track: u32,
    length: u64,
    pending: Vec<f32>,
    pending_at: usize,
    position: u64,
}
impl EncodedReader {
    fn new(bytes: Arc<[u8]>, audio_format: AudioFormat) -> Result<Self, AudioDecodeError> {
        let format = open_encoded_for_reading(bytes, audio_format)?;
        let track = format
            .default_track()
            .ok_or(AudioDecodeError::InvalidData)?;
        let decoder = codec_registry()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|_| AudioDecodeError::InvalidData)?;
        let track_id = track.id;
        let length = track.codec_params.n_frames.unwrap_or(0);
        Ok(Self {
            format,
            decoder,
            track: track_id,
            length,
            pending: Vec::new(),
            pending_at: 0,
            position: 0,
        })
    }
    fn frame(&mut self, channels: usize, out: &mut Vec<f32>) -> Result<bool, AudioDecodeError> {
        while self.pending_at + channels > self.pending.len() {
            let packet = match self.format.next_packet() {
                Ok(packet) => packet,
                Err(SymphoniaError::IoError(error))
                    if error.kind() == std::io::ErrorKind::UnexpectedEof =>
                {
                    return Ok(false);
                }
                Err(_) => return Err(AudioDecodeError::InvalidData),
            };
            if packet.track_id() != self.track {
                continue;
            }
            let decoded = self
                .decoder
                .decode(&packet)
                .map_err(|_| AudioDecodeError::InvalidData)?;
            if decoded.spec().channels.count() != channels {
                return Err(AudioDecodeError::InvalidData);
            }
            let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, *decoded.spec());
            buffer.copy_interleaved_ref(decoded);
            self.pending.clear();
            self.pending.extend_from_slice(buffer.samples());
            self.pending_at = 0;
        }
        out.clear();
        out.extend_from_slice(&self.pending[self.pending_at..self.pending_at + channels]);
        self.pending_at += channels;
        self.position += 1;
        Ok(true)
    }
    fn seek(&mut self, frame: u64, channels: usize) -> Option<u64> {
        // Decode preroll to restore MP3 synthesis/filter state as well as its bit reservoir.
        let target = frame.saturating_sub(4096);
        let seek = self
            .format
            .seek(
                SeekMode::Accurate,
                SeekTo::TimeStamp {
                    ts: target,
                    track_id: self.track,
                },
            )
            .or_else(|_| {
                self.format.seek(
                    SeekMode::Accurate,
                    SeekTo::TimeStamp {
                        ts: 0,
                        track_id: self.track,
                    },
                )
            })
            .ok()?;
        self.decoder.reset();
        self.pending.clear();
        self.pending_at = 0;
        self.position = seek.actual_ts;
        let mut discard = Vec::with_capacity(channels);
        while self.position < frame {
            if !self.frame(channels, &mut discard).ok()? {
                break;
            }
        }
        Some(self.position)
    }
}
enum ReaderInput {
    Buffered { samples: Arc<[f32]>, cursor: usize },
    Encoded(EncodedReader),
}
impl ReaderInput {
    fn length(&self, channels: u32) -> u64 {
        match self {
            Self::Buffered { samples, .. } => (samples.len() / channels as usize) as u64,
            Self::Encoded(reader) => reader.length,
        }
    }
    fn frame(&mut self, channels: u32, out: &mut Vec<f32>) -> Result<bool, AudioDecodeError> {
        match self {
            Self::Buffered { samples, cursor } => {
                let end = cursor.saturating_add(channels as usize);
                let Some(frame) = samples.get(*cursor..end) else {
                    return Ok(false);
                };
                out.clear();
                out.extend_from_slice(frame);
                *cursor = end;
                Ok(true)
            }
            Self::Encoded(reader) => reader.frame(channels as usize, out),
        }
    }
    fn seek(&mut self, frame: u64, channels: u32) -> Option<u64> {
        match self {
            Self::Buffered { samples, cursor } => {
                let landed = frame.min((samples.len() / channels as usize) as u64);
                *cursor = landed as usize * channels as usize;
                Some(landed)
            }
            Self::Encoded(reader) => reader.seek(frame, channels as usize),
        }
    }
}

/// Incremental reader with D17's linear converter, independent of input seek state.
/// Rewinding the input preserves the converter's phase and endpoint samples.
pub struct AudioReader {
    input: ReaderInput,
    channels: u32,
    sample_rate: u32,
    input_channels: u32,
    input_rate: u32,
    left: Vec<f32>,
    right: Vec<f32>,
    has_left: bool,
    has_right: bool,
    source_position: u64,
    output_position: u64,
    phase: u64,
    read_buffer: Vec<f32>,
}
impl std::fmt::Debug for AudioReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AudioReader")
            .field("channels", &self.channels)
            .field("sample_rate", &self.sample_rate)
            .finish_non_exhaustive()
    }
}
impl AudioReader {
    pub fn channels(&self) -> u32 {
        self.channels
    }
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }
    /// MP3 length is scanned from complete audio frames, not its header count.
    pub fn length_in_frames(&self) -> u64 {
        self.converted_length(self.input.length(self.input_channels))
    }
    fn converted_length(&self, frames: u64) -> u64 {
        ((u128::from(frames) * u128::from(self.sample_rate) + u128::from(self.input_rate / 2))
            / u128::from(self.input_rate))
        .min(u128::from(u64::MAX)) as u64
    }
    fn channel(&self, frame: &[f32], channel: usize) -> f32 {
        if self.channels == self.input_channels {
            frame[channel]
        } else if self.channels == 1 {
            frame.iter().copied().sum::<f32>() / self.input_channels as f32
        } else if self.input_channels == 1 {
            frame[0]
        } else {
            frame[channel % self.input_channels as usize]
        }
    }
    fn next_frame(&mut self, out: &mut [f32]) -> Result<bool, AudioDecodeError> {
        if self.input_rate == self.sample_rate {
            // No resampling: never evaluate an unused difference which can overflow.
            if !self.input.frame(self.input_channels, &mut self.left)? {
                return Ok(false);
            }
            for (channel, sample) in out.iter_mut().enumerate() {
                *sample = self.channel(&self.left, channel);
            }
            return Ok(true);
        }
        if !self.has_left {
            self.has_left = self.input.frame(self.input_channels, &mut self.left)?;
            if !self.has_left {
                return Ok(false);
            }
        }
        loop {
            if !self.has_right {
                self.has_right = self.input.frame(self.input_channels, &mut self.right)?;
            }
            if self.phase < u64::from(self.sample_rate) || !self.has_right {
                break;
            }
            std::mem::swap(&mut self.left, &mut self.right);
            self.has_right = false;
            self.source_position += 1;
            self.phase -= u64::from(self.sample_rate);
        }
        if !self.has_right
            && self.output_position >= self.converted_length(self.source_position + 1)
        {
            return Ok(false);
        }
        let fraction = self.phase as f32 / self.sample_rate as f32;
        for (channel, sample) in out.iter_mut().enumerate() {
            let left = self.channel(&self.left, channel);
            let right = if self.has_right {
                self.channel(&self.right, channel)
            } else {
                left
            };
            *sample = left + (right - left) * fraction;
        }
        self.phase += u64::from(self.input_rate);
        self.output_position += 1;
        Ok(true)
    }
    /// Fill complete interleaved frames; EOF is a successful zero-frame read.
    pub fn read_into(&mut self, out: &mut [f32]) -> Result<u64, AudioReadError> {
        read_frames(out, self.channels as usize, |frame| self.next_frame(frame))
    }
    pub fn read(&mut self, frame_count: u64) -> &[f32] {
        let Some(samples) = usize::try_from(frame_count)
            .ok()
            .and_then(|n| n.checked_mul(self.channels as usize))
        else {
            return &[];
        };
        let mut buffer = std::mem::take(&mut self.read_buffer);
        buffer.resize(samples, 0.0);
        let count = self.read_into(&mut buffer).unwrap_or(0) as usize * self.channels as usize;
        buffer.truncate(count);
        self.read_buffer = buffer;
        &self.read_buffer
    }
    pub fn seek(&mut self, frame: u64) -> Option<u64> {
        let at = frame as f64 * self.input_rate as f64 / self.sample_rate as f64;
        let input = if at < 1.8e19 { at as u64 } else { u64::MAX };
        let landed = self.input.seek(input, self.input_channels)?;
        self.has_left = false;
        self.has_right = false;
        self.source_position = 0;
        self.output_position = 0;
        self.phase = 0;
        Some(frame.min((landed as f64 * self.sample_rate as f64 / self.input_rate as f64) as u64))
    }
    pub fn rewind(&mut self) -> bool {
        self.input.seek(0, self.input_channels).is_some()
    }
    /// Append one decoder block. Completion is EOF or the first block past
    /// max_frames, whose oversized result the caller can reject.
    pub fn read_block(&mut self, max_frames: u64, out: &mut Vec<f32>, done: &mut bool) -> bool {
        let channels = self.channels as usize;
        if out.is_empty() {
            let reserve = self
                .length_in_frames()
                .min(max_frames)
                .saturating_add(4096)
                .saturating_mul(self.channels as u64)
                .min((isize::MAX as usize / std::mem::size_of::<f32>()) as u64);
            out.reserve(reserve as usize);
        }
        let at = out.len();
        out.resize(at + 4096 * channels, 0.0);
        let Ok(read) = self.read_into(&mut out[at..]) else {
            return false;
        };
        out.truncate(at + read as usize * channels);
        *done = read == 0 || (out.len() / channels) as u64 > max_frames;
        true
    }

    pub fn read_all(&mut self, max_frames: u64, out: &mut Vec<f32>) -> bool {
        out.clear();
        let mut done = false;
        while !done {
            if !self.read_block(max_frames, out, &mut done) {
                out.clear();
                return false;
            }
        }
        true
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
        let footer_len = usize::from(major_version == 4 && flags & 0x10 != 0) * 10;
        let Some(next) = offset
            .checked_add(10)
            .and_then(|value| value.checked_add(payload_len))
            .and_then(|value| value.checked_add(footer_len))
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
            &FormatOptions {
                enable_gapless: true,
                ..Default::default()
            },
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

// Like upstream makeReader and encoded playback seekPointCount, scan the MP3 packet headers
// before reading so length and gapless trimming agree with the audio present.
// Source validation remains lazy and does not perform this scan.
fn open_encoded_for_reading(
    bytes: Arc<[u8]>,
    audio_format: AudioFormat,
) -> Result<Box<dyn FormatReader>, AudioDecodeError> {
    let options = FormatOptions {
        enable_gapless: true,
        ..Default::default()
    };
    if audio_format == AudioFormat::Mp3 {
        let mut format = MpaReader::try_new(media_source(bytes, audio_format), &options)
            .map_err(|_| AudioDecodeError::InvalidData)?;
        format
            .establish_exact_frame_count()
            .map_err(|_| AudioDecodeError::InvalidData)?;
        return Ok(Box::new(format));
    }
    format_probe()
        .format(
            &hint_for(audio_format),
            media_source(bytes, audio_format),
            &options,
            &MetadataOptions::default(),
        )
        .map(|probed| probed.format)
        .map_err(|_| AudioDecodeError::InvalidData)
}

fn decode_encoded(
    bytes: Arc<[u8]>,
    audio_format: AudioFormat,
) -> Result<DecodedAudio, AudioDecodeError> {
    // Upstream encoded playback also enables the decoder seek table, so it
    // receives the same exact MP3 count as standalone sample readers.
    let mut format = open_encoded_for_reading(bytes, audio_format)?;
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

    fn with_info_list(mut wav: Vec<u8>) -> Vec<u8> {
        const INFO_LIST: &[u8] = b"LIST\x10\x00\x00\x00INFOINAM\x04\x00\x00\x00tag\x00";
        assert!(wav.starts_with(b"RIFF"));
        assert_eq!(wav.get(8..12), Some(b"WAVE".as_slice()));
        let riff_len = u32::from_le_bytes(wav[4..8].try_into().expect("RIFF length"));
        let tagged_riff_len = riff_len
            .checked_add(u32::try_from(INFO_LIST.len()).expect("INFO LIST fits RIFF"))
            .expect("tagged RIFF length fits u32");
        wav.splice(12..12, INFO_LIST.iter().copied());
        wav[4..8].copy_from_slice(&tagged_riff_len.to_le_bytes());
        wav
    }

    #[test]
    fn read_error_keeps_the_number_of_complete_frames_already_written() {
        let mut out = [9.0; 8];
        let mut produced = 0;
        let error = read_frames(&mut out, 2, |frame| {
            if produced == 2 {
                return Err(AudioDecodeError::InvalidData);
            }
            frame.fill(produced as f32 + 1.0);
            produced += 1;
            Ok(true)
        })
        .unwrap_err();
        assert_eq!(error.frames_read, 2);
        assert_eq!(error.error, AudioDecodeError::InvalidData);
        assert_eq!(out, [1.0, 1.0, 2.0, 2.0, 9.0, 9.0, 9.0, 9.0]);
    }
    #[test]
    fn equal_rate_pcm_is_copied_without_interpolating_or_reading_ahead() {
        let source = AudioSource::from_buffered(vec![f32::MAX, -f32::MAX], 1, 48000).unwrap();
        let mut reader = source.make_reader(0, 0).unwrap();
        let mut out = [0.0];
        assert_eq!(reader.read_into(&mut out).unwrap(), 1);
        assert_eq!(out[0], f32::MAX);
        let ReaderInput::Buffered { cursor, .. } = &reader.input else {
            panic!("buffered source")
        };
        assert_eq!(*cursor, 1, "the next input frame has not been read");
        assert_eq!(reader.read_into(&mut out).unwrap(), 1);
        assert_eq!(out[0], -f32::MAX);
    }

    // Upstream audio_test.cpp at 6292144fe.
    #[test]
    fn mp3_reader_seeks_to_the_frames_a_whole_decode_has() {
        let source = AudioSource::from_encoded(pinned_fixture("audio/song.mp3")).unwrap();
        let mut whole = Vec::new();
        assert!(
            source
                .make_reader(0, 0)
                .unwrap()
                .read_all(u64::MAX, &mut whole)
        );
        let mut reader = source.make_reader(0, 0).unwrap();
        let channels = reader.channels() as usize;
        let total = (whole.len() / channels) as u64;
        assert!(total > 100000);
        for frame in [total * 3 / 4, total / 3, 12345] {
            assert_eq!(reader.seek(frame), Some(frame));
            let mut read = vec![0.0; 4096 * channels];
            assert_eq!(reader.read_into(&mut read).unwrap(), 4096);
            assert_eq!(
                read,
                &whole[frame as usize * channels..(frame as usize + 4096) * channels],
                "frame {frame}"
            );
        }
        let mut fresh = source.make_reader(0, 0).unwrap();
        let mut first = vec![0.0; 4096 * channels];
        assert_eq!(fresh.read_into(&mut first).unwrap(), 4096);
        assert_eq!(fresh.seek(4096 + 1105), Some(4096 + 1105));
        assert_eq!(fresh.read_into(&mut first).unwrap(), 4096);
        assert_eq!(
            first,
            &whole[(4096 + 1105) * channels..(8192 + 1105) * channels]
        );
        assert_eq!(reader.seek(total + 1000), Some(total));
        assert_eq!(reader.read_into(&mut vec![0.0; 16 * channels]).unwrap(), 0);
    }
    // Upstream audio_test.cpp at aec8aea4: iTunes counts its Info frame.
    #[test]
    fn mp3_whose_header_counts_a_frame_too_many_reports_what_decodes() {
        let mut bytes = pinned_fixture("audio/song.mp3");
        let original = AudioSource::from_encoded(bytes.clone()).unwrap();
        let mut whole = Vec::new();
        assert!(
            original
                .make_reader(0, 0)
                .unwrap()
                .read_all(u64::MAX, &mut whole)
        );
        let tag = bytes
            .windows(4)
            .position(|v| v == b"Info")
            .expect("Info header");
        assert_ne!(bytes[tag + 7] & 1, 0);
        let count = tag + 8;
        let frames = u32::from_be_bytes(bytes[count..count + 4].try_into().unwrap()) + 1;
        bytes[count..count + 4].copy_from_slice(&frames.to_be_bytes());
        let source = AudioSource::from_encoded(bytes).unwrap();
        let mut reader = source.make_reader(0, 0).unwrap();
        let expected_frames = whole.len() as u64 / reader.channels() as u64;
        // Length must be correct even before PCM decoding, not patched at EOF.
        assert_eq!(reader.length_in_frames(), expected_frames);
        let mut decoded = Vec::new();
        assert!(reader.read_all(u64::MAX, &mut decoded));
        assert_eq!(decoded.len(), whole.len());
        assert_eq!(decoded, whole);
        // AudioEngine's encoded-playback path must use the corrected count too,
        // including gapless end trimming, not merely report a corrected duration.
        let playback = source
            .decode_for_output(reader.channels(), reader.sample_rate())
            .unwrap();
        assert_eq!(playback.as_ref(), decoded.as_slice());
        assert_eq!(
            playback.len() as u64 / reader.channels() as u64,
            expected_frames
        );
        assert_eq!(reader.length_in_frames(), expected_frames);
        assert_eq!(
            source.duration(),
            reader.length_in_frames() as f32 / reader.sample_rate() as f32
        );
    }

    #[test]
    fn read_block_appends_one_chunk_and_finishes_only_at_eof_or_past_cap() {
        let source = AudioSource::from_buffered(vec![0.25; 8192], 1, 48000).unwrap();
        let mut reader = source.make_reader(0, 0).unwrap();
        let mut samples = Vec::new();
        let mut done = false;
        assert!(reader.read_block(4096, &mut samples, &mut done));
        assert_eq!(samples.len(), 4096);
        assert!(!done, "exact cap is not past cap");
        assert!(reader.read_block(4096, &mut samples, &mut done));
        assert_eq!(samples, vec![0.25; 8192]);
        assert!(done);
        let mut reader = source.make_reader(0, 0).unwrap();
        samples.clear();
        done = false;
        assert!(reader.read_block(u64::MAX, &mut samples, &mut done));
        assert!(!done);
        assert!(reader.read_block(u64::MAX, &mut samples, &mut done));
        assert!(!done);
        assert!(reader.read_block(u64::MAX, &mut samples, &mut done));
        assert!(done);
        assert_eq!(samples, vec![0.25; 8192]);
    }

    #[test]
    fn audio_decode_stops_past_its_cap() {
        let source = AudioSource::from_encoded(pinned_fixture("audio/what.wav")).unwrap();
        let mut samples = Vec::new();
        assert!(
            source
                .make_reader(1, 0)
                .unwrap()
                .read_all(1000, &mut samples)
        );
        assert!(samples.len() > 1000 && samples.len() <= 1000 + 4096);
    }
    #[test]
    fn reader_creation_is_lazy_and_buffered_zero_format_is_native() {
        let source = AudioSource::from_encoded(pinned_fixture("audio/what.wav")).unwrap();
        let reader = source.make_reader(0, 0).unwrap();
        let ReaderInput::Encoded(input) = &reader.input else {
            panic!("encoded reader")
        };
        assert!(input.pending.is_empty());
        assert_eq!(input.position, 0);
        let source = AudioSource::from_buffered(vec![1.0, 2.0, 3.0, 4.0], 2, 48000).unwrap();
        let mut reader = source.make_reader(0, 0).unwrap();
        assert_eq!((reader.channels(), reader.sample_rate()), (2, 48000));
        assert_eq!(reader.length_in_frames(), 2);
        let mut out = [9.0; 6];
        assert_eq!(reader.read_into(&mut out).unwrap(), 2);
        assert_eq!(out, [1.0, 2.0, 3.0, 4.0, 9.0, 9.0]);
        assert_eq!(reader.seek(10), Some(2));
        assert_eq!(reader.read_into(&mut out).unwrap(), 0);
        assert_eq!(reader.seek(1), Some(1));
        assert_eq!(reader.read_into(&mut out).unwrap(), 1);
        assert_eq!(out[..2], [3.0, 4.0]);
    }
    #[test]
    fn resampled_rewind_keeps_phase_where_seek_resets_it() {
        let samples = vec![0.0, 1.0, 0.25];
        let source = AudioSource::from_buffered(samples.clone(), 1, 4).unwrap();
        let mut reader = source.make_reader(1, 5).unwrap();
        let mut first = [0.0; 4];
        assert_eq!(reader.read_into(&mut first).unwrap(), 4);
        assert!(reader.rewind());
        // Compare inside the next loop, before its independently clamped EOF.
        let mut looped = [0.0; 3];
        assert_eq!(reader.read_into(&mut looped).unwrap(), 3);
        let repeated = AudioSource::from_buffered(samples.repeat(3), 1, 4).unwrap();
        let mut reference = repeated.make_reader(1, 5).unwrap();
        let mut expected = [0.0; 7];
        assert_eq!(reference.read_into(&mut expected).unwrap(), 7);
        assert_eq!(looped, expected[4..]);
        assert_eq!(reader.seek(0), Some(0));
        let mut reset = [0.0; 4];
        reader.read_into(&mut reset).unwrap();
        assert_eq!(reset, first);
    }
    #[test]
    fn reader_resampling_matches_the_approved_output_converter() {
        // D17 substitutes this Rust converter for miniaudio's filter. Compare
        // incremental reads to the existing whole-source output conversion.
        let source = AudioSource::from_encoded(pinned_fixture("audio/what.wav")).unwrap();
        let expected = source.decode_for_output(1, 16000).unwrap();
        let mut reader = source.make_reader(1, 16000).unwrap();
        let mut samples = Vec::new();
        assert!(reader.read_all(u64::MAX, &mut samples));
        assert_eq!(samples.as_slice(), expected.as_ref());
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
        assert!(mono_48k.length_in_frames().abs_diff(10_545) <= 2);
        let stereo_32k = source.make_reader(2, 32_000).expect("32 kHz reader");
        assert!(stereo_32k.length_in_frames().abs_diff(7_030) <= 2);
    }

    #[test]
    fn wav_info_list_is_skipped_without_changing_audio() {
        let bytes = pinned_fixture("audio/what.wav");
        let plain = AudioSource::from_encoded(bytes.clone()).expect("plain WAV validates");
        let tagged =
            AudioSource::from_encoded(with_info_list(bytes)).expect("tagged WAV validates");

        assert_eq!(tagged.format(), plain.format());
        assert_eq!(tagged.channels(), plain.channels());
        assert_eq!(tagged.sample_rate(), plain.sample_rate());
        assert_eq!(tagged.duration(), plain.duration());

        let mut expected = plain
            .make_reader(plain.channels(), plain.sample_rate())
            .expect("plain native reader");
        let mut actual = tagged
            .make_reader(tagged.channels(), tagged.sample_rate())
            .expect("tagged native reader");
        let frame_count = expected.length_in_frames();
        assert_eq!(actual.length_in_frames(), frame_count);
        let expected_samples = expected.read(frame_count).to_vec();
        let actual_samples = actual.read(frame_count);
        assert_eq!(actual_samples.len(), expected_samples.len());

        let profile = |samples: &[f32]| {
            let channels = plain.channels() as usize;
            let mut peaks = vec![0.0_f32; channels];
            for frame in samples.chunks_exact(channels) {
                for (peak, sample) in peaks.iter_mut().zip(frame) {
                    *peak = peak.max(sample.abs());
                }
            }
            let window_energy = samples
                .chunks(channels * 512)
                .map(|window| {
                    window
                        .iter()
                        .map(|sample| f64::from(*sample) * f64::from(*sample))
                        .sum::<f64>()
                        / window.len() as f64
                })
                .collect::<Vec<_>>();
            (peaks, window_energy)
        };
        let (expected_peaks, expected_energy) = profile(&expected_samples);
        let (actual_peaks, actual_energy) = profile(actual_samples);
        assert!(expected_peaks.iter().any(|peak| *peak > 0.0));
        for (expected, actual) in expected_peaks.iter().zip(actual_peaks) {
            assert!((expected - actual).abs() <= 1.0e-6);
        }
        assert_eq!(actual_energy.len(), expected_energy.len());
        for (expected, actual) in expected_energy.iter().zip(actual_energy) {
            let tolerance = 1.0e-10_f64.max(expected.abs() * 1.0e-8);
            assert!((expected - actual).abs() <= tolerance);
        }
        assert!(expected.read(1).is_empty());
        assert!(actual.read(1).is_empty());
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
        assert!(buffered.make_reader(2, 48_000).is_some());

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

        let mut chained = b"ID3\x04\x00\x10\x00\x00\x00\x03tag".to_vec();
        chained.extend_from_slice(b"3DI\x04\x00\x10\x00\x00\x00\x03");
        chained.extend_from_slice(b"ID3\x04\x00\x00\x00\x00\x00\x03two");
        chained.extend_from_slice(b"\xff\xfb");
        assert_eq!(id3v2_payload_offset(&chained), 36);
    }
}
