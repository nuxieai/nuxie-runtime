//! Shared values for isolated Luau audio processors.
//!
//! The payload codec is a Luau adaptation: one-byte tags, little-endian u32
//! lengths/handles, f64 numbers, and recursively encoded arrays/objects. Ring
//! framing and shared-hold charges follow upstream (length + payload + holds).
//! No main-VM Lua value crosses a thread. Locks are acquired by try/spin, as in
//! upstream SharedRing's lock word; no callback waits on a sleeping mutex.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU32, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError, Weak};

pub const MAX_SHARED_BYTES: usize = 256 << 20;
pub const DEFAULT_SHARED_BUDGET: usize = 512 << 20;
pub const MAX_SHARED_OBJECTS: usize = 65_536;
pub const PROCESSOR_HOLDS_ERROR: &str = "a processor cannot pass shared objects through rings or snapshots; pass them in Audio.start's argument";

include!("luau_audio_shared_codec.rs");

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    let mut attempts = 0usize;
    loop {
        match mutex.try_lock() {
            Ok(guard) => return guard,
            Err(TryLockError::Poisoned(error)) => return error.into_inner(),
            Err(TryLockError::WouldBlock) => {
                if attempts >= 64 {
                    std::thread::yield_now();
                } else {
                    std::hint::spin_loop();
                }
                attempts = attempts.saturating_add(1);
            }
        }
    }
}

#[derive(Default)]
struct BudgetState {
    bytes: usize,
    objects: usize,
    closed: bool,
    entries: Vec<SharedWeak>,
}
enum SharedWeak {
    Closed(Weak<AtomicBool>),
    Ring(Weak<Mutex<RingState>>),
    Snapshot(Weak<SnapshotState>),
}
pub struct SharedBudget {
    maximum_bytes: usize,
    state: Mutex<BudgetState>,
}
impl Default for SharedBudget {
    fn default() -> Self {
        Self::new(DEFAULT_SHARED_BUDGET)
    }
}
impl SharedBudget {
    pub fn new(maximum_bytes: usize) -> Self {
        Self {
            maximum_bytes,
            state: Mutex::new(BudgetState::default()),
        }
    }
    fn charge(self: &Arc<Self>, bytes: usize, objects: usize) -> Result<Charge, String> {
        let mut state = lock(&self.state);
        if state.closed {
            return Err("shared owner has been released".into());
        }
        if bytes > self.maximum_bytes.saturating_sub(state.bytes) {
            return Err("shared memory budget exceeded".into());
        }
        if objects > MAX_SHARED_OBJECTS.saturating_sub(state.objects) {
            return Err("shared object limit exceeded".into());
        }
        state.bytes += bytes;
        state.objects += objects;
        Ok(Charge {
            budget: self.clone(),
            bytes,
            objects,
        })
    }
    pub fn reserve_decode(self: &Arc<Self>, bytes: usize) -> Result<DecodeReservation, String> {
        self.charge(bytes, 0).map(DecodeReservation)
    }
    fn register(&self, entry: SharedWeak) -> Result<(), String> {
        let mut state = lock(&self.state);
        if state.closed {
            return Err("shared owner has been released".into());
        }
        state.entries.retain(|entry| match entry {
            SharedWeak::Closed(value) => value.strong_count() != 0,
            SharedWeak::Ring(value) => value.strong_count() != 0,
            SharedWeak::Snapshot(value) => value.strong_count() != 0,
        });
        state.entries.push(entry);
        Ok(())
    }
    /// Retire all objects, including payload cycles kept alive by shared handles.
    /// Invoke after stopping owned processors, on VM teardown.
    pub fn close(&self) {
        let entries = {
            let mut state = lock(&self.state);
            state.closed = true;
            std::mem::take(&mut state.entries)
        };
        for entry in entries {
            match entry {
                SharedWeak::Closed(value) => {
                    if let Some(value) = value.upgrade() {
                        value.store(true, Ordering::SeqCst);
                    }
                }
                SharedWeak::Ring(value) => {
                    if let Some(value) = value.upgrade() {
                        let holds = {
                            let mut state = lock(&value);
                            state.closed = true;
                            state.used = 0;
                            std::mem::take(&mut state.holds)
                        };
                        drop(holds);
                    }
                }
                SharedWeak::Snapshot(value) => {
                    if let Some(value) = value.upgrade() {
                        let holds = {
                            let mut writer = lock(&value.writer);
                            writer.closed = true;
                            value.published.store(0, Ordering::SeqCst);
                            std::mem::take(&mut writer.holds)
                        };
                        drop(holds);
                    }
                }
            }
        }
    }
}
pub struct DecodeReservation(#[allow(dead_code)] Charge);
struct Charge {
    budget: Arc<SharedBudget>,
    bytes: usize,
    objects: usize,
}
impl Drop for Charge {
    fn drop(&mut self) {
        let mut state = lock(&self.budget.state);
        state.bytes -= self.bytes;
        state.objects -= self.objects;
    }
}
fn allocate_bytes(capacity: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| "cannot allocate shared storage")?;
    bytes.resize(capacity, 0);
    Ok(bytes)
}
struct RingState {
    bytes: Vec<u8>,
    head: usize,
    used: usize,
    // Keys are record offsets, which cannot collide while the records are live.
    // This map is touched only for messages containing main-lane shared handles.
    holds: BTreeMap<usize, Vec<SharedValue>>,
    closed: bool,
}
impl RingState {
    fn write(&mut self, at: usize, bytes: &[u8]) {
        let capacity = self.bytes.len();
        for (offset, value) in bytes.iter().enumerate() {
            self.bytes[(at + offset) % capacity] = *value;
        }
    }
    fn word(&self, at: usize) -> u32 {
        let mut bytes = [0; 4];
        for (offset, byte) in bytes.iter_mut().enumerate() {
            *byte = self.bytes[(at + offset) % self.bytes.len()];
        }
        u32::from_le_bytes(bytes)
    }
}
pub struct SharedRing {
    capacity: usize,
    state: Arc<Mutex<RingState>>,
    _charge: Charge,
}
impl std::fmt::Debug for SharedRing {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedRing")
            .field("capacity", &self.capacity)
            .finish_non_exhaustive()
    }
}
impl SharedRing {
    pub fn new(capacity: usize) -> Result<Self, String> {
        Self::with_budget(capacity, Arc::new(SharedBudget::default()))
    }
    pub fn with_budget(capacity: usize, budget: Arc<SharedBudget>) -> Result<Self, String> {
        if capacity == 0 || capacity > MAX_SHARED_BYTES {
            return Err("invalid shared ring byte capacity".into());
        }
        let charge = budget.charge(capacity, 3)?;
        let state = Arc::new(Mutex::new(RingState {
            bytes: allocate_bytes(capacity)?,
            head: 0,
            used: 0,
            holds: BTreeMap::new(),
            closed: false,
        }));
        budget.register(SharedWeak::Ring(Arc::downgrade(&state)))?;
        Ok(Self {
            capacity,
            state,
            _charge: charge,
        })
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn used(&self) -> usize {
        lock(&self.state).used
    }
    pub fn references(this: &Arc<Self>) -> usize {
        Arc::strong_count(this)
    }
    /// Raw no-hold record, with the same four-byte framing as upstream PCM.
    pub fn write_record(&self, bytes: &[u8]) -> Result<bool, String> {
        let size = bytes.len().checked_add(4).ok_or("record too large")?;
        let mut state = lock(&self.state);
        if state.closed || size > self.capacity - state.used {
            return Ok(false);
        }
        let tail = (state.head + state.used) % self.capacity;
        state.write(tail, &(bytes.len() as u32).to_le_bytes());
        state.write(tail + 4, bytes);
        state.used += size;
        Ok(true)
    }
    /// Zero means empty/busy. Otherwise size + 1, leaving an oversized record
    /// queued. Callback consumers never wait for a producer holding the lock.
    pub fn read_record(&self, out: &mut [u8]) -> Result<usize, String> {
        let mut state = match self.state.try_lock() {
            Ok(state) => state,
            Err(TryLockError::Poisoned(error)) => error.into_inner(),
            Err(TryLockError::WouldBlock) => return Ok(0),
        };
        if state.used == 0 {
            return Ok(0);
        }
        let word = state.word(state.head);
        if word & (1 << 31) != 0 {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        let size = word as usize;
        if size > out.len() {
            return Ok(size + 1);
        }
        for (offset, byte) in out[..size].iter_mut().enumerate() {
            *byte = state.bytes[(state.head + 4 + offset) % self.capacity];
        }
        state.head = (state.head + 4 + size) % self.capacity;
        state.used -= size + 4;
        Ok(size + 1)
    }
    pub fn send(&self, value: SharedValue) -> Result<bool, String> {
        let (payload, _) = value.layout()?;
        let size = payload.checked_add(4).ok_or("shared value too large")?;
        if size > self.capacity {
            return Ok(false);
        }
        let encoded = encode(&value)?;
        let mut state = lock(&self.state);
        if state.closed || size > self.capacity - state.used {
            return Ok(false);
        }
        let tail = (state.head + state.used) % self.capacity;
        let word = payload as u32 | if encoded.holds.is_empty() { 0 } else { 1 << 31 };
        state.write(tail, &word.to_le_bytes());
        state.write(tail + 4, &encoded.bytes);
        if !encoded.holds.is_empty() {
            let trailer = tail + 4 + encoded.bytes.len();
            for index in 0..encoded.holds.len() {
                state.write(trailer + index * 4, &(index as u32).to_le_bytes());
            }
            state.write(
                trailer + encoded.holds.len() * 4,
                &(encoded.holds.len() as u32).to_le_bytes(),
            );
            state.holds.insert(tail, encoded.holds);
        }
        state.used += size;
        Ok(true)
    }
    pub fn send_from_processor(&self, value: SharedValue) -> Result<bool, String> {
        if value.contains_handles() {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        self.send(value)
    }
    fn read(&self, processor: bool) -> Result<Option<SharedValue>, String> {
        // Reserve outside the lock. If another consumer changes the head to a
        // larger record, retry with sufficient space without holding the writer.
        let mut encoded = Vec::new();
        loop {
            let mut state = lock(&self.state);
            if state.used == 0 {
                return Ok(None);
            }
            let head = state.head;
            let word = state.word(head);
            let payload = (word & !(1 << 31)) as usize;
            let has_holds = word & (1 << 31) != 0;
            if processor && has_holds {
                return Err(PROCESSOR_HOLDS_ERROR.into());
            }
            let hold_count = if has_holds {
                state.word(head + payload) as usize
            } else {
                0
            };
            let trailer = if has_holds { 4 + hold_count * 4 } else { 0 };
            let size = payload - trailer;
            if encoded.capacity() < size {
                drop(state);
                encoded
                    .try_reserve_exact(size)
                    .map_err(|_| "cannot read shared ring")?;
                continue;
            }
            encoded.clear();
            for offset in 0..size {
                encoded.push(state.bytes[(head + 4 + offset) % self.capacity]);
            }
            let holds = state.holds.remove(&head).unwrap_or_default();
            state.head = (head + 4 + payload) % self.capacity;
            state.used -= 4 + payload;
            drop(state);
            return decode(&encoded, &holds).map(Some);
        }
    }
    pub fn try_receive(&self) -> Result<Option<SharedValue>, String> {
        self.read(false)
    }
    pub fn receive(&self) -> Option<SharedValue> {
        // All bytes were encoded by this module, so decoder errors indicate an
        // internal invariant failure rather than malformed user wire input.
        self.read(false)
            .expect("internally encoded shared ring record")
    }
    pub fn receive_for_processor(&self) -> Result<Option<SharedValue>, String> {
        self.read(true)
    }
}
struct SnapshotCopy {
    bytes: Box<[AtomicU8]>,
    version: AtomicU64,
    sequence: AtomicU64,
    size: AtomicUsize,
    has_holds: AtomicBool,
}
impl SnapshotCopy {
    fn new(capacity: usize) -> Result<Self, String> {
        let mut bytes = Vec::new();
        bytes
            .try_reserve_exact(capacity)
            .map_err(|_| "cannot allocate shared snapshot")?;
        bytes.extend((0..capacity).map(|_| AtomicU8::new(0)));
        Ok(Self {
            bytes: bytes.into_boxed_slice(),
            version: AtomicU64::new(0),
            sequence: AtomicU64::new(0),
            size: AtomicUsize::new(0),
            has_holds: AtomicBool::new(false),
        })
    }
}
struct SnapshotWriter {
    sequence: u64,
    holds: [Vec<SharedValue>; 2],
    closed: bool,
}
struct SnapshotState {
    copies: [SnapshotCopy; 2],
    // Zero until the first publication; otherwise copy index + 1.
    published: AtomicUsize,
    writer: Mutex<SnapshotWriter>,
}
pub struct SharedSnapshot {
    capacity: usize,
    state: Arc<SnapshotState>,
    _charge: Charge,
}
impl std::fmt::Debug for SharedSnapshot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedSnapshot")
            .field("capacity", &self.capacity)
            .finish_non_exhaustive()
    }
}
impl SharedSnapshot {
    pub fn new(capacity: usize) -> Result<Self, String> {
        Self::with_budget(capacity, Arc::new(SharedBudget::default()))
    }
    pub fn with_budget(capacity: usize, budget: Arc<SharedBudget>) -> Result<Self, String> {
        let storage = capacity
            .checked_mul(2)
            .ok_or("invalid shared snapshot byte capacity")?;
        if storage > MAX_SHARED_BYTES {
            return Err("invalid shared snapshot byte capacity".into());
        }
        let charge = budget.charge(storage, 1)?;
        let state = Arc::new(SnapshotState {
            copies: [SnapshotCopy::new(capacity)?, SnapshotCopy::new(capacity)?],
            published: AtomicUsize::new(0),
            writer: Mutex::new(SnapshotWriter {
                sequence: 0,
                holds: [Vec::new(), Vec::new()],
                closed: false,
            }),
        });
        budget.register(SharedWeak::Snapshot(Arc::downgrade(&state)))?;
        Ok(Self {
            capacity,
            state,
            _charge: charge,
        })
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn seq(&self) -> u64 {
        loop {
            let published = self.state.published.load(Ordering::SeqCst);
            if published == 0 {
                return 0;
            }
            let copy = &self.state.copies[published - 1];
            let version = copy.version.load(Ordering::SeqCst);
            if version & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }
            let sequence = copy.sequence.load(Ordering::SeqCst);
            if copy.version.load(Ordering::SeqCst) == version {
                return sequence;
            }
        }
    }
    fn write(&self, value: SharedValue, processor: bool) -> Result<bool, String> {
        if processor && value.contains_handles() {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        let (size, _) = value.layout()?;
        if size > self.capacity {
            return Ok(false);
        }
        let encoded = encode(&value)?;
        let mut writer = lock(&self.state.writer);
        if writer.closed {
            return Ok(false);
        }
        let published = self.state.published.load(Ordering::SeqCst);
        if processor && published != 0 && !writer.holds[published - 1].is_empty() {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        let index = if published == 1 { 1 } else { 0 };
        let copy = &self.state.copies[index];
        copy.version.fetch_add(1, Ordering::SeqCst);
        for (target, byte) in copy.bytes.iter().zip(&encoded.bytes) {
            target.store(*byte, Ordering::SeqCst);
        }
        copy.size.store(encoded.bytes.len(), Ordering::SeqCst);
        copy.has_holds
            .store(!encoded.holds.is_empty(), Ordering::SeqCst);
        writer.sequence = writer.sequence.wrapping_add(1);
        copy.sequence.store(writer.sequence, Ordering::SeqCst);
        let old = std::mem::replace(&mut writer.holds[index], encoded.holds);
        copy.version.fetch_add(1, Ordering::SeqCst);
        self.state.published.store(index + 1, Ordering::SeqCst);
        // The prior publication is no longer live. Keep neither duplicate holds
        // nor their destruction on the publisher lock.
        let previous = if published != 0 {
            std::mem::take(&mut writer.holds[published - 1])
        } else {
            Vec::new()
        };
        drop(writer);
        drop(old);
        drop(previous);
        Ok(true)
    }
    pub fn publish(&self, value: SharedValue) -> Result<bool, String> {
        self.write(value, false)
    }
    pub fn publish_from_processor(&self, value: SharedValue) -> Result<bool, String> {
        self.write(value, true)
    }
    fn read(&self, processor: bool) -> Result<Option<(u64, SharedValue)>, String> {
        let mut bytes = Vec::new();
        loop {
            let published = self.state.published.load(Ordering::SeqCst);
            if published == 0 {
                return Ok(None);
            }
            let copy = &self.state.copies[published - 1];
            let version = copy.version.load(Ordering::SeqCst);
            if version & 1 != 0 {
                std::hint::spin_loop();
                continue;
            }
            let sequence = copy.sequence.load(Ordering::SeqCst);
            let size = copy.size.load(Ordering::SeqCst).min(self.capacity);
            let has_holds = copy.has_holds.load(Ordering::SeqCst);
            if has_holds {
                if copy.version.load(Ordering::SeqCst) != version {
                    continue;
                }
                if processor {
                    return Err(PROCESSOR_HOLDS_ERROR.into());
                }
                // Main-lane holds are cloned under the writer lock so publication
                // cannot retire them while the main reader acquires ownership.
                let writer = lock(&self.state.writer);
                let published = self.state.published.load(Ordering::SeqCst);
                if published == 0 {
                    return Ok(None);
                }
                let copy = &self.state.copies[published - 1];
                let sequence = copy.sequence.load(Ordering::SeqCst);
                let size = copy.size.load(Ordering::SeqCst);
                bytes.clear();
                bytes
                    .try_reserve_exact(size)
                    .map_err(|_| "cannot read shared snapshot")?;
                bytes.extend(
                    copy.bytes[..size]
                        .iter()
                        .map(|value| value.load(Ordering::SeqCst)),
                );
                let holds = writer.holds[published - 1].clone();
                drop(writer);
                return decode(&bytes, &holds).map(|value| Some((sequence, value)));
            }
            bytes.clear();
            bytes
                .try_reserve_exact(size)
                .map_err(|_| "cannot read shared snapshot")?;
            bytes.extend(
                copy.bytes[..size]
                    .iter()
                    .map(|value| value.load(Ordering::SeqCst)),
            );
            // All copy words are atomic and ordered with the version markers;
            // a racing overwrite cannot pass validation with mixed payload bytes.
            if copy.version.load(Ordering::SeqCst) == version {
                return decode(&bytes, &[]).map(|value| Some((sequence, value)));
            }
        }
    }
    pub fn try_latest(&self) -> Result<Option<(u64, SharedValue)>, String> {
        self.read(false)
    }
    pub fn latest(&self) -> Option<(u64, SharedValue)> {
        self.read(false)
            .expect("internally encoded shared snapshot")
    }
    pub fn latest_for_processor(&self) -> Result<Option<(u64, SharedValue)>, String> {
        self.read(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ring_byte_capacity_includes_framing_and_preserves_fifo() {
        let ring = SharedRing::new(26).unwrap();
        assert!(ring.send(SharedValue::Number(1.0)).unwrap());
        assert!(ring.send(SharedValue::Number(2.0)).unwrap());
        assert_eq!(ring.used(), 26);
        assert!(!ring.send(SharedValue::Nil).unwrap());
        assert!(matches!(ring.receive(), Some(SharedValue::Number(1.0))));
        assert!(matches!(ring.receive(), Some(SharedValue::Number(2.0))));
        assert!(ring.receive().is_none());
        assert_eq!(ring.used(), 0);
    }
    #[test]
    fn processor_refusal_keeps_nested_shared_record() {
        let ring = SharedRing::new(256).unwrap();
        let shared = Arc::new(SharedSnapshot::new(64).unwrap());
        assert!(
            ring.send(SharedValue::Array(vec![SharedValue::Snapshot(shared)]))
                .unwrap()
        );
        let used = ring.used();
        assert!(
            ring.receive_for_processor()
                .unwrap_err()
                .contains("Audio.start")
        );
        assert_eq!(ring.used(), used);
        assert!(matches!(ring.receive(), Some(SharedValue::Array(_))));
    }
    #[test]
    fn snapshot_oversize_preserves_value_and_sequence() {
        let snapshot = SharedSnapshot::new(9).unwrap();
        assert!(snapshot.latest().is_none());
        assert!(snapshot.publish(SharedValue::Number(7.0)).unwrap());
        assert!(!snapshot.publish(SharedValue::Bytes(vec![0; 5])).unwrap());
        assert!(matches!(
            snapshot.latest(),
            Some((1, SharedValue::Number(7.0)))
        ));
        assert!(snapshot.publish(SharedValue::Number(8.0)).unwrap());
        assert!(matches!(
            snapshot.latest(),
            Some((2, SharedValue::Number(8.0)))
        ));
    }
    #[test]
    fn processor_cannot_replace_snapshot_holds() {
        let snapshot = SharedSnapshot::new(64).unwrap();
        snapshot
            .publish(SharedValue::Ring(Arc::new(SharedRing::new(16).unwrap())))
            .unwrap();
        assert!(
            snapshot
                .publish_from_processor(SharedValue::Number(1.0))
                .is_err()
        );
        assert!(snapshot.latest_for_processor().is_err());
        assert!(matches!(snapshot.latest(), Some((1, SharedValue::Ring(_)))));
    }
    #[test]
    fn shared_budget_refunds_and_close_breaks_payload_cycles() {
        let budget = Arc::new(SharedBudget::new(64));
        let ring = Arc::new(SharedRing::with_budget(64, budget.clone()).unwrap());
        assert!(SharedSnapshot::with_budget(1, budget.clone()).is_err());
        let weak = Arc::downgrade(&ring);
        assert!(ring.send(SharedValue::Ring(ring.clone())).unwrap());
        drop(ring);
        assert!(weak.upgrade().is_some());
        budget.close();
        assert!(weak.upgrade().is_none());
        assert_eq!(lock(&budget.state).bytes, 0);
        assert!(SharedRing::with_budget(1, budget).is_err());
    }
    #[test]
    fn snapshot_read_never_mixes_sequence_and_payload() {
        let snapshot = Arc::new(SharedSnapshot::new(9).unwrap());
        let writer = snapshot.clone();
        let thread = std::thread::spawn(move || {
            for n in 1..=10_000 {
                assert!(writer.publish(SharedValue::Number(n as f64)).unwrap());
            }
        });
        while !thread.is_finished() {
            if let Some((sequence, SharedValue::Number(value))) = snapshot.latest() {
                assert_eq!(sequence as f64, value);
            }
        }
        thread.join().unwrap();
        assert!(matches!(
            snapshot.latest(),
            Some((10_000, SharedValue::Number(10_000.0)))
        ));
    }
    #[test]
    fn ring_supports_multiple_producers_without_lost_records() {
        let ring = Arc::new(SharedRing::new(4 * 1000 * 13).unwrap());
        let threads: Vec<_> = (0..4)
            .map(|producer| {
                let ring = ring.clone();
                std::thread::spawn(move || {
                    for n in 0..1000 {
                        assert!(
                            ring.send(SharedValue::Number((producer * 1000 + n) as f64))
                                .unwrap()
                        );
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        let mut values = Vec::new();
        while let Some(SharedValue::Number(value)) = ring.receive() {
            values.push(value as usize);
        }
        values.sort_unstable();
        assert_eq!(values, (0..4000).collect::<Vec<_>>());
    }
    #[test]
    fn fixed_storage_matches_byte_budget_without_record_slot_amplification() {
        let budget = Arc::new(SharedBudget::new(3 * 1024));
        let ring = SharedRing::with_budget(1024, budget.clone()).unwrap();
        let snapshot = SharedSnapshot::with_budget(1024, budget.clone()).unwrap();
        assert_eq!(lock(&ring.state).bytes.len(), 1024);
        assert_eq!(lock(&ring.state).bytes.capacity(), 1024);
        assert_eq!(
            snapshot
                .state
                .copies
                .iter()
                .map(|copy| copy.bytes.len())
                .sum::<usize>(),
            2048
        );
        assert_eq!(lock(&budget.state).bytes, 3072);
        assert!(SharedRing::with_budget(1, budget.clone()).is_err());
        drop(ring);
        assert_eq!(lock(&budget.state).bytes, 2048);
        assert!(SharedRing::with_budget(1024, budget).is_ok());
    }
    #[test]
    fn snapshot_reader_does_not_wait_for_paused_publication() {
        let snapshot = Arc::new(SharedSnapshot::new(32).unwrap());
        snapshot.publish(SharedValue::Number(7.0)).unwrap();
        let guard = lock(&snapshot.state.writer);
        let current = snapshot.state.published.load(Ordering::SeqCst) - 1;
        let inactive = &snapshot.state.copies[current ^ 1];
        // A publisher can be preempted after invalidating its inactive copy.
        inactive.version.fetch_add(1, Ordering::SeqCst);
        inactive.bytes[0].store(255, Ordering::SeqCst);
        let (tx, rx) = std::sync::mpsc::channel();
        let reader = snapshot.clone();
        let thread = std::thread::spawn(move || {
            tx.send(reader.latest_for_processor()).unwrap();
        });
        let result = rx.recv_timeout(std::time::Duration::from_secs(2));
        drop(guard);
        thread.join().unwrap();
        assert!(matches!(
            result.unwrap().unwrap(),
            Some((1, SharedValue::Number(7.0)))
        ));
    }
    #[test]
    fn ring_wraps_encoded_payload_and_preserves_binary_strings() {
        let ring = SharedRing::new(26).unwrap();
        ring.send(SharedValue::Number(1.0)).unwrap();
        ring.receive().unwrap();
        let bytes = vec![0, 127, 128, 255, 0, 1, 2, 3];
        assert!(ring.send(SharedValue::Bytes(bytes.clone())).unwrap());
        match ring.receive().unwrap() {
            SharedValue::Bytes(value) => assert_eq!(value, bytes),
            _ => panic!("wrong value kind"),
        }
        assert_eq!(ring.used(), 0);
    }
    #[test]
    fn snapshot_owner_release_breaks_self_hold() {
        let budget = Arc::new(SharedBudget::default());
        let snapshot = Arc::new(SharedSnapshot::with_budget(64, budget.clone()).unwrap());
        let weak = Arc::downgrade(&snapshot);
        snapshot
            .publish(SharedValue::Snapshot(snapshot.clone()))
            .unwrap();
        drop(snapshot);
        assert!(weak.upgrade().is_some());
        budget.close();
        assert!(weak.upgrade().is_none());
        assert_eq!(lock(&budget.state).bytes, 0);
    }
}

pub struct SharedBlob {
    bytes: Box<[u8]>,
    closed: Arc<AtomicBool>,
    _charge: Charge,
}
impl std::fmt::Debug for SharedBlob {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedBlob")
            .field("size", &self.size())
            .finish()
    }
}
impl SharedBlob {
    pub fn with_budget(bytes: &[u8], budget: Arc<SharedBudget>) -> Result<Self, String> {
        if bytes.len() > MAX_SHARED_BYTES {
            return Err("invalid shared blob size".into());
        }
        let charge = budget.charge(bytes.len(), 1)?;
        let mut copy = allocate_bytes(bytes.len())?;
        copy.copy_from_slice(bytes);
        let closed = Arc::new(AtomicBool::new(false));
        budget.register(SharedWeak::Closed(Arc::downgrade(&closed)))?;
        Ok(Self {
            bytes: copy.into_boxed_slice(),
            closed,
            _charge: charge,
        })
    }
    pub fn size(&self) -> usize {
        if self.closed.load(Ordering::SeqCst) {
            0
        } else {
            self.bytes.len()
        }
    }
    pub fn read(&self, offset: usize, out: &mut [u8]) -> usize {
        let count = out.len().min(self.size().saturating_sub(offset));
        if count != 0 {
            out[..count].copy_from_slice(&self.bytes[offset..offset + count]);
        }
        count
    }
}
/// Atomic backing also makes exclusive byte copies safe against concurrent
/// processor atomic accesses; no Rust reference aliases mutable shared bytes.
pub struct SharedRegion {
    size: usize,
    words: Box<[AtomicU32]>,
    held: AtomicBool,
    poisoned: AtomicBool,
    closed: Arc<AtomicBool>,
    _charge: Charge,
}
impl std::fmt::Debug for SharedRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SharedRegion")
            .field("size", &self.size())
            .finish()
    }
}
impl SharedRegion {
    pub fn with_budget(size: usize, budget: Arc<SharedBudget>) -> Result<Self, String> {
        if size == 0 || size > MAX_SHARED_BYTES {
            return Err("invalid shared region size".into());
        }
        let charge = budget.charge(size, 1)?;
        let mut words = Vec::new();
        words
            .try_reserve_exact(size.div_ceil(4))
            .map_err(|_| "shared allocation failed")?;
        words.resize_with(size.div_ceil(4), || AtomicU32::new(0));
        let closed = Arc::new(AtomicBool::new(false));
        budget.register(SharedWeak::Closed(Arc::downgrade(&closed)))?;
        Ok(Self {
            size,
            words: words.into_boxed_slice(),
            held: AtomicBool::new(false),
            poisoned: AtomicBool::new(false),
            closed,
            _charge: charge,
        })
    }
    pub fn size(&self) -> usize {
        if self.closed.load(Ordering::SeqCst) {
            0
        } else {
            self.size
        }
    }
    pub fn poisoned(&self) -> bool {
        !self.closed.load(Ordering::SeqCst) && self.poisoned.load(Ordering::SeqCst)
    }
    fn word(&self, offset: usize) -> Result<&AtomicU32, String> {
        if offset % 4 != 0 || offset.checked_add(4).is_none_or(|end| end > self.size()) {
            return Err("invalid shared region atomic offset".into());
        }
        Ok(&self.words[offset / 4])
    }
    pub fn atomic_load(&self, offset: usize) -> Result<u32, String> {
        Ok(self.word(offset)?.load(Ordering::SeqCst))
    }
    pub fn atomic_store(&self, offset: usize, value: u32) -> Result<u32, String> {
        self.word(offset)?.store(value, Ordering::SeqCst);
        Ok(0)
    }
    pub fn atomic_add(&self, offset: usize, value: u32) -> Result<u32, String> {
        Ok(self.word(offset)?.fetch_add(value, Ordering::SeqCst))
    }
    pub fn atomic_cas(&self, offset: usize, expected: u32, desired: u32) -> Result<u32, String> {
        Ok(self
            .word(offset)?
            .compare_exchange(expected, desired, Ordering::SeqCst, Ordering::SeqCst)
            .unwrap_or_else(|old| old))
    }
    pub fn try_lock(
        self: &Arc<Self>,
        timeout: std::time::Duration,
    ) -> Result<Option<RegionGuard>, String> {
        let start = std::time::Instant::now();
        loop {
            if self.size() == 0 {
                return Err("shared owner has been released".into());
            }
            if self
                .held
                .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
                .is_ok()
            {
                return Ok(Some(RegionGuard(self.clone())));
            }
            if start.elapsed() >= timeout {
                return Ok(None);
            }
            std::thread::yield_now();
        }
    }
}
pub struct RegionGuard(Arc<SharedRegion>);
impl RegionGuard {
    pub fn poison(&self) {
        self.0.poisoned.store(true, Ordering::SeqCst);
    }
    pub fn read(&self, offset: usize, out: &mut [u8]) -> Result<(), String> {
        if offset
            .checked_add(out.len())
            .is_none_or(|end| end > self.0.size())
        {
            return Err("shared region read out of bounds".into());
        }
        for (index, byte) in out.iter_mut().enumerate() {
            let address = offset + index;
            *byte = (self.0.words[address / 4].load(Ordering::SeqCst) >> ((address % 4) * 8)) as u8;
        }
        Ok(())
    }
    pub fn write(&self, offset: usize, input: &[u8]) -> Result<(), String> {
        if offset
            .checked_add(input.len())
            .is_none_or(|end| end > self.0.size())
        {
            return Err("shared region write out of bounds".into());
        }
        for (index, byte) in input.iter().enumerate() {
            let address = offset + index;
            let shift = (address % 4) * 8;
            self.0.words[address / 4]
                .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
                    Some((old & !(255 << shift)) | ((*byte as u32) << shift))
                })
                .expect("unconditional update");
        }
        Ok(())
    }
}
impl Drop for RegionGuard {
    fn drop(&mut self) {
        self.0.held.store(false, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod region_blob_tests {
    use super::*;
    #[test]
    fn blob_copies_clamps_and_retires() {
        let budget = Arc::new(SharedBudget::new(16));
        let mut source = vec![1, 2, 3];
        let blob = Arc::new(SharedBlob::with_budget(&source, budget.clone()).unwrap());
        source[0] = 99;
        let mut output = [9; 4];
        assert_eq!(blob.read(1, &mut output), 2);
        assert_eq!(output, [2, 3, 9, 9]);
        assert_eq!(blob.read(usize::MAX, &mut output), 0);
        assert_eq!(
            SharedBlob::with_budget(&[], budget.clone()).unwrap().size(),
            0
        );
        budget.close();
        assert_eq!(blob.size(), 0);
        assert_eq!(blob.read(0, &mut output), 0);
    }
    #[test]
    fn retained_region_reports_unpoisoned_after_owner_retirement() {
        let budget = Arc::new(SharedBudget::default());
        let region = Arc::new(SharedRegion::with_budget(4, budget.clone()).unwrap());
        let guard = region.try_lock(std::time::Duration::ZERO).unwrap().unwrap();
        guard.poison();
        drop(guard);
        assert!(region.poisoned());
        budget.close();
        assert!(!region.poisoned());
        assert_eq!(region.size(), 0);
    }

    #[test]
    fn region_atomic_bounds_old_values_and_exclusive_bytes() {
        let region =
            Arc::new(SharedRegion::with_budget(9, Arc::new(SharedBudget::default())).unwrap());
        assert_eq!(region.atomic_store(0, u32::MAX).unwrap(), 0);
        assert_eq!(region.atomic_add(0, 2).unwrap(), u32::MAX);
        assert_eq!(region.atomic_load(0).unwrap(), 1);
        assert_eq!(region.atomic_cas(0, 2, 3).unwrap(), 1);
        assert_eq!(region.atomic_cas(0, 1, 3).unwrap(), 1);
        assert_eq!(region.atomic_load(0).unwrap(), 3);
        assert!(region.atomic_load(1).is_err());
        assert!(region.atomic_load(8).is_err());
        assert!(region.atomic_load(usize::MAX).is_err());
        let guard = region.try_lock(std::time::Duration::ZERO).unwrap().unwrap();
        assert!(
            region
                .try_lock(std::time::Duration::ZERO)
                .unwrap()
                .is_none()
        );
        guard.write(4, &[4, 5, 6, 7, 8]).unwrap();
        let mut output = [0; 5];
        guard.read(4, &mut output).unwrap();
        assert_eq!(output, [4, 5, 6, 7, 8]);
        assert_eq!(region.atomic_load(4).unwrap(), 0x07060504);
        assert!(guard.write(8, &[1, 2]).is_err());
        guard.poison();
        assert!(region.poisoned());
        drop(guard);
        assert!(
            region
                .try_lock(std::time::Duration::ZERO)
                .unwrap()
                .is_some()
        );
    }
    #[test]
    fn blob_region_holds_codec_and_processor_refusal() {
        let budget = Arc::new(SharedBudget::default());
        let blob = Arc::new(SharedBlob::with_budget(&[7], budget.clone()).unwrap());
        let region = Arc::new(SharedRegion::with_budget(4, budget).unwrap());
        let ring = SharedRing::new(128).unwrap();
        assert!(
            ring.send(SharedValue::Array(vec![
                SharedValue::Blob(blob.clone()),
                SharedValue::Region(region.clone())
            ]))
            .unwrap()
        );
        assert!(ring.receive_for_processor().is_err());
        let Some(SharedValue::Array(value)) = ring.receive() else {
            panic!("missing handles")
        };
        assert!(matches!(&value[0],SharedValue::Blob(value)if Arc::ptr_eq(value,&blob)));
        assert!(matches!(&value[1],SharedValue::Region(value)if Arc::ptr_eq(value,&region)));
    }
}
