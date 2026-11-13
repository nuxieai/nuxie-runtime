//! Shared PCM stream protocol from upstream HostAudioStream. Luau binding names
//! are a local adaptation; the 16-byte header and source-frame units are not.
use super::luau_audio_shared::{
    MAX_SHARED_BYTES, SharedBudget, SharedRegion, SharedRing, SharedValue,
};
use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::sync::Arc;

pub(super) const HEADER_BYTES: usize = 16;
pub(super) const RECORD_LAST: u32 = 1;
pub(super) const RECORD_DISCONTINUITY: u32 = 2;
#[derive(Clone, Debug)]
pub(super) struct PcmStreamDescriptor {
    pub ring: Arc<SharedRing>,
    pub seeks: Arc<SharedRegion>,
    pub sample_rate: u32,
    pub channels: u32,
    pub chunk_frames: u32,
}
impl PcmStreamDescriptor {
    pub fn new(
        capacity: usize,
        sample_rate: u32,
        channels: u32,
        chunk_frames: u32,
        budget: Arc<SharedBudget>,
    ) -> Result<Self, String> {
        let descriptor = Self {
            ring: Arc::new(SharedRing::with_budget(capacity, budget.clone())?),
            seeks: Arc::new(SharedRegion::with_budget(8, budget)?),
            sample_rate,
            channels,
            chunk_frames,
        };
        descriptor.record_capacity()?;
        Ok(descriptor)
    }
    /// Bytes of one chunk's interleaved f32 samples, without the header.
    /// With the header it is at most `MAX_SHARED_BYTES`.
    pub fn chunk_sample_bytes(&self) -> Result<usize, String> {
        if self.sample_rate == 0 || self.channels == 0 || self.chunk_frames == 0 {
            return Err("invalid PCM stream format".into());
        }
        (self.channels as usize)
            .checked_mul(self.chunk_frames as usize)
            .and_then(|n| n.checked_mul(4))
            .filter(|n| n.checked_add(HEADER_BYTES).is_some_and(|n| n <= MAX_SHARED_BYTES))
            .ok_or_else(|| "PCM stream chunk too large".into())
    }
    pub fn record_capacity(&self) -> Result<usize, String> {
        let bytes = self.chunk_sample_bytes()?.saturating_add(HEADER_BYTES);
        if self.seeks.size() < 8 || self.ring.capacity() < bytes.saturating_add(4) {
            return Err("invalid PCM stream storage".into());
        }
        Ok(bytes)
    }
    /// Bytes of one interleaved frame.
    pub fn frame_bytes(&self) -> Result<NonZeroUsize, String> {
        NonZeroUsize::new((self.channels as usize).saturating_mul(4))
            .ok_or_else(|| "invalid PCM stream format".into())
    }
    pub fn generation(&self) -> Result<u32, String> {
        self.seeks.atomic_load(0)
    }
    pub fn bump_generation(&self) -> Result<u32, String> {
        self.seeks.atomic_add(0, 1).map(|n| n.wrapping_add(1))
    }
    pub fn reader_count(&self) -> Result<u32, String> {
        self.seeks.atomic_load(4)
    }
    /// One ring hold belongs to the controller. All other retained descriptors,
    /// including queued processor initialization arguments, keep pumping alive.
    pub fn external_holds(&self) -> usize {
        SharedRing::references(&self.ring).saturating_sub(1)
    }
    pub fn shared_value(&self) -> SharedValue {
        SharedValue::Object(BTreeMap::from([
            ("pcmStream".into(), SharedValue::Boolean(true)),
            ("ring".into(), SharedValue::Ring(self.ring.clone())),
            ("seeks".into(), SharedValue::Region(self.seeks.clone())),
            (
                "sampleRate".into(),
                SharedValue::Number(self.sample_rate as f64),
            ),
            ("channels".into(), SharedValue::Number(self.channels as f64)),
            (
                "chunkFrames".into(),
                SharedValue::Number(self.chunk_frames as f64),
            ),
        ]))
    }
    pub fn from_shared(value: &SharedValue) -> Result<Self, String> {
        let SharedValue::Object(fields) = value else {
            return Err("expected PCM stream descriptor".into());
        };
        if !matches!(fields.get("pcmStream"), Some(SharedValue::Boolean(true))) {
            return Err("expected PCM stream descriptor".into());
        }
        let number = |key| -> Result<u32, String> {
            match fields.get(key) {
                Some(SharedValue::Number(n))
                    if n.is_finite() && *n > 0.0 && n.fract() == 0.0 && *n <= u32::MAX as f64 =>
                {
                    Ok(*n as u32)
                }
                _ => Err("invalid PCM stream format".into()),
            }
        };
        let Some(SharedValue::Ring(ring)) = fields.get("ring") else {
            return Err("missing PCM stream ring".into());
        };
        let Some(SharedValue::Region(seeks)) = fields.get("seeks") else {
            return Err("missing PCM stream generation".into());
        };
        let descriptor = Self {
            ring: ring.clone(),
            seeks: seeks.clone(),
            sample_rate: number("sampleRate")?,
            channels: number("channels")?,
            chunk_frames: number("chunkFrames")?,
        };
        descriptor.record_capacity()?;
        Ok(descriptor)
    }
}
pub(super) struct PcmStreamReader {
    descriptor: PcmStreamDescriptor,
    record: Vec<u8>,
    record_size: usize,
    offset: usize,
    generation: u32,
    frame: f64,
    ended: bool,
    last: bool,
}
impl PcmStreamReader {
    pub fn new(descriptor: PcmStreamDescriptor) -> Result<Self, String> {
        let size = descriptor.record_capacity()?;
        let mut record = Vec::new();
        record
            .try_reserve_exact(size)
            .map_err(|_| "cannot allocate PCM reader")?;
        record.resize(size, 0);
        let generation = descriptor.generation()?;
        descriptor.seeks.atomic_add(4, 1)?;
        Ok(Self {
            descriptor,
            record,
            record_size: 0,
            offset: HEADER_BYTES,
            generation,
            frame: 0.0,
            ended: false,
            last: false,
        })
    }
    pub fn descriptor(&self) -> &PcmStreamDescriptor {
        &self.descriptor
    }
    pub fn frame(&self) -> f64 {
        self.frame
    }
    pub fn ended(&self) -> bool {
        self.ended && self.descriptor.generation().ok() == Some(self.generation)
    }
    /// Writes interleaved f32 little-endian PCM. Returns complete output frames.
    /// No allocation, decoder call, or waiting occurs in this consumer.
    pub fn read_into_bytes(&mut self, out: &mut [u8]) -> Result<usize, String> {
        let frame_bytes = self.descriptor.frame_bytes()?;
        let capacity = (out.len() / frame_bytes).saturating_mul(frame_bytes.get());
        let mut written = 0usize;
        while written < capacity {
            let generation = self.descriptor.generation()?;
            if generation != self.generation {
                prefix_mut(out, written).fill(0);
                written = 0;
                self.generation = generation;
                self.record_size = 0;
                self.offset = HEADER_BYTES;
                self.ended = false;
                self.last = false;
            }
            if self.offset >= self.record_size {
                let Some(size) = self
                    .descriptor
                    .ring
                    .read_record(&mut self.record)?
                    .checked_sub(1)
                else {
                    break;
                };
                let Some((header, samples)) = self
                    .record
                    .get(..size)
                    .and_then(<[u8]>::split_first_chunk::<HEADER_BYTES>)
                    .filter(|(_, samples)| samples.len() % frame_bytes == 0)
                else {
                    return Err("invalid PCM stream record".into());
                };
                let RecordHeader { seek, flags, frame } = RecordHeader::read(header);
                if seek != self.generation {
                    continue;
                }
                let empty = samples.is_empty();
                self.frame = frame;
                self.offset = HEADER_BYTES;
                self.record_size = size;
                self.last = flags & RECORD_LAST != 0;
                self.ended = self.last && empty;
                if self.ended {
                    break;
                }
                if empty {
                    continue;
                }
            }
            // Copy what both the record and the output have room for.
            let source = self
                .record
                .get(self.offset..self.record_size)
                .unwrap_or_default();
            let target = out.get_mut(written..capacity).unwrap_or_default();
            let mut count = 0usize;
            for (byte, value) in target.iter_mut().zip(source) {
                *byte = *value;
                count = count.saturating_add(1);
            }
            // A concurrent explicit seek invalidates even a previously dequeued
            // chunk. Do not publish samples copied across that generation edge.
            if self.descriptor.generation()? != self.generation {
                prefix_mut(out, written.saturating_add(count)).fill(0);
                return Ok(0);
            }
            self.offset = self.offset.saturating_add(count);
            written = written.saturating_add(count);
            self.frame += (count / frame_bytes) as f64;
            if self.offset == self.record_size && self.last {
                self.ended = true;
                break;
            }
        }
        Ok(written / frame_bytes)
    }
}
/// A PCM record's 16-byte header.
struct RecordHeader {
    seek: u32,
    flags: u32,
    frame: f64,
}
impl RecordHeader {
    fn read(bytes: &[u8; HEADER_BYTES]) -> Self {
        let [s0, s1, s2, s3, f0, f1, f2, f3, frame @ ..] = *bytes;
        Self {
            seek: u32::from_le_bytes([s0, s1, s2, s3]),
            flags: u32::from_le_bytes([f0, f1, f2, f3]),
            frame: f64::from_le_bytes(frame),
        }
    }
}
/// The first `len` items of `slice`, or all of it when it is shorter.
pub(super) fn prefix_mut<T>(slice: &mut [T], len: usize) -> &mut [T] {
    let len = len.min(slice.len());
    slice.split_at_mut_checked(len).map_or(&mut [], |(head, _)| head)
}
/// The first `len` items of `slice`, or all of it when it is shorter.
pub(super) fn prefix<T>(slice: &[T], len: usize) -> &[T] {
    let len = len.min(slice.len());
    slice.split_at_checked(len).map_or(&[], |(head, _)| head)
}
impl Drop for PcmStreamReader {
    fn drop(&mut self) {
        let _ = self.descriptor.seeks.atomic_add(4, u32::MAX);
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use super::*;
    fn descriptor() -> PcmStreamDescriptor {
        PcmStreamDescriptor::new(256, 48_000, 1, 4, Arc::new(SharedBudget::default())).unwrap()
    }
    fn record(seek: u32, flags: u32, frame: f64, samples: &[f32]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend(seek.to_le_bytes());
        bytes.extend(flags.to_le_bytes());
        bytes.extend(frame.to_le_bytes());
        for sample in samples {
            bytes.extend(sample.to_le_bytes());
        }
        bytes
    }
    #[test]
    fn partial_records_terminal_and_seek_discard_dequeued_pcm() {
        let descriptor = descriptor();
        let mut reader = PcmStreamReader::new(descriptor.clone()).unwrap();
        assert_eq!(descriptor.reader_count().unwrap(), 1);
        descriptor
            .ring
            .write_record(&record(0, 0, 10.0, &[1.0, 2.0, 3.0, 4.0]))
            .unwrap();
        let mut output = [0; 8];
        assert_eq!(reader.read_into_bytes(&mut output).unwrap(), 2);
        assert_eq!(f32::from_le_bytes(output[4..8].try_into().unwrap()), 2.0);
        assert_eq!(reader.frame(), 12.0);
        descriptor.bump_generation().unwrap();
        descriptor
            .ring
            .write_record(&record(0, 0, 14.0, &[99.0]))
            .unwrap();
        descriptor
            .ring
            .write_record(&record(1, RECORD_DISCONTINUITY, 50.0, &[5.0]))
            .unwrap();
        descriptor
            .ring
            .write_record(&record(1, RECORD_LAST, 51.0, &[]))
            .unwrap();
        assert_eq!(reader.read_into_bytes(&mut output).unwrap(), 1);
        assert_eq!(f32::from_le_bytes(output[..4].try_into().unwrap()), 5.0);
        assert!(reader.ended());
        assert_eq!(reader.frame(), 51.0);
        descriptor.bump_generation().unwrap();
        assert!(!reader.ended());
        drop(reader);
        assert_eq!(descriptor.reader_count().unwrap(), 0);
    }
    #[test]
    fn descriptor_holds_cover_pending_transfer_and_readers() {
        let descriptor = descriptor();
        assert_eq!(descriptor.external_holds(), 0);
        let value = descriptor.shared_value();
        assert_eq!(descriptor.external_holds(), 1);
        let reconstructed = PcmStreamDescriptor::from_shared(&value).unwrap();
        let reader = PcmStreamReader::new(reconstructed).unwrap();
        assert_eq!(descriptor.external_holds(), 2);
        drop(value);
        assert_eq!(descriptor.external_holds(), 1);
        drop(reader);
        assert_eq!(descriptor.external_holds(), 0);
    }
    #[test]
    fn raw_ring_oversized_record_is_not_consumed() {
        let descriptor = descriptor();
        descriptor
            .ring
            .write_record(&record(0, 0, 0.0, &[1.0]))
            .unwrap();
        assert_eq!(descriptor.ring.read_record(&mut [0; 16]).unwrap(), 21);
        let mut output = [0; 20];
        assert_eq!(descriptor.ring.read_record(&mut output).unwrap(), 21);
        assert_eq!(descriptor.ring.read_record(&mut output).unwrap(), 0);
    }
}
