//! Shared values for isolated Luau audio processors.
//!
//! The payload codec is a Luau adaptation: one-byte tags, little-endian u32
//! lengths/handles, f64 numbers, and recursively encoded arrays/objects. Ring
//! framing and shared-hold charges follow upstream (length + payload + holds).
//! No main-VM Lua value crosses a thread. Locks are acquired by try/spin, as in
//! upstream SharedRing's lock word; no callback waits on a sleeping mutex.
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, TryLockError, Weak};

pub const MAX_SHARED_BYTES: usize = 256 << 20;
pub const DEFAULT_SHARED_BUDGET: usize = 512 << 20;
pub const MAX_SHARED_OBJECTS: usize = 65_536;
pub const PROCESSOR_HOLDS_ERROR: &str = "a processor cannot pass shared objects through rings or snapshots; pass them in Audio.start's argument";

#[derive(Clone, Debug)]
pub enum SharedValue {
    Nil,
    Boolean(bool),
    Number(f64),
    Bytes(Vec<u8>),
    Array(Vec<SharedValue>),
    Object(BTreeMap<String, SharedValue>),
    Ring(Arc<SharedRing>),
    Snapshot(Arc<SharedSnapshot>),
}
impl SharedValue {
    pub fn contains_handles(&self) -> bool {
        match self {
            Self::Ring(_) | Self::Snapshot(_) => true,
            Self::Array(values) => values.iter().any(Self::contains_handles),
            Self::Object(values) => values.values().any(Self::contains_handles),
            _ => false,
        }
    }
    fn layout(&self) -> Result<(usize, usize), String> {
        let mut bytes = 0usize;
        let mut holds = 0usize;
        let mut pending = vec![self];
        while let Some(value) = pending.pop() {
            let size = match value {
                Self::Nil => 1,
                Self::Boolean(_) => 2,
                Self::Number(_) => 9,
                Self::Bytes(value) => 5usize
                    .checked_add(value.len())
                    .ok_or("shared value too large")?,
                Self::Array(values) => {
                    pending.extend(values);
                    5
                }
                Self::Object(values) => {
                    let mut size = 5usize;
                    for (key, value) in values {
                        size = size
                            .checked_add(4)
                            .and_then(|n| n.checked_add(key.len()))
                            .ok_or("shared value too large")?;
                        pending.push(value);
                    }
                    size
                }
                Self::Ring(_) | Self::Snapshot(_) => {
                    holds = holds.checked_add(1).ok_or("shared value too large")?;
                    5
                }
            };
            bytes = bytes.checked_add(size).ok_or("shared value too large")?;
            if bytes > MAX_SHARED_BYTES {
                return Err("shared value too large".into());
            }
        }
        let trailer = if holds == 0 {
            0
        } else {
            holds
                .checked_mul(4)
                .and_then(|n| n.checked_add(4))
                .ok_or("shared value too large")?
        };
        Ok((
            bytes.checked_add(trailer).ok_or("shared value too large")?,
            holds,
        ))
    }
}

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
    fn register(&self, entry: SharedWeak) -> Result<(), String> {
        let mut state = lock(&self.state);
        if state.closed {
            return Err("shared owner has been released".into());
        }
        state.entries.retain(|entry| match entry {
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
// Encoded bytes are the actual bounded transport storage. Shared-object holds
// are kept separately only for main-lane records; processors cannot take them.
struct EncodedValue {
    bytes: Vec<u8>,
    holds: Vec<SharedValue>,
}
fn encode(value: &SharedValue) -> Result<EncodedValue, String> {
    let (size, _) = value.layout()?;
    let mut encoded = EncodedValue {
        bytes: Vec::new(),
        holds: Vec::new(),
    };
    encoded
        .bytes
        .try_reserve_exact(size)
        .map_err(|_| "cannot encode shared value")?;
    fn word(out: &mut Vec<u8>, value: usize) -> Result<(), String> {
        let value = u32::try_from(value).map_err(|_| "shared value too large")?;
        out.extend_from_slice(&value.to_le_bytes());
        Ok(())
    }
    fn write(out: &mut EncodedValue, value: &SharedValue) -> Result<(), String> {
        match value {
            SharedValue::Nil => out.bytes.push(0),
            SharedValue::Boolean(value) => out.bytes.extend_from_slice(&[1, u8::from(*value)]),
            SharedValue::Number(value) => {
                out.bytes.push(2);
                out.bytes.extend_from_slice(&value.to_le_bytes());
            }
            SharedValue::Bytes(value) => {
                out.bytes.push(3);
                word(&mut out.bytes, value.len())?;
                out.bytes.extend_from_slice(value);
            }
            SharedValue::Array(values) => {
                out.bytes.push(4);
                word(&mut out.bytes, values.len())?;
                for value in values {
                    write(out, value)?;
                }
            }
            SharedValue::Object(values) => {
                out.bytes.push(5);
                word(&mut out.bytes, values.len())?;
                for (key, value) in values {
                    word(&mut out.bytes, key.len())?;
                    out.bytes.extend_from_slice(key.as_bytes());
                    write(out, value)?;
                }
            }
            SharedValue::Ring(_) | SharedValue::Snapshot(_) => {
                out.bytes.push(6);
                word(&mut out.bytes, out.holds.len())?;
                out.holds.push(value.clone());
            }
        }
        Ok(())
    }
    write(&mut encoded, value)?;
    Ok(encoded)
}
fn decode(bytes: &[u8], holds: &[SharedValue]) -> Result<SharedValue, String> {
    struct Reader<'a> {
        bytes: &'a [u8],
        at: usize,
        holds: &'a [SharedValue],
    }
    impl Reader<'_> {
        fn take(&mut self, size: usize) -> Result<&[u8], String> {
            let end = self.at.checked_add(size).ok_or("invalid shared value")?;
            let value = self.bytes.get(self.at..end).ok_or("invalid shared value")?;
            self.at = end;
            Ok(value)
        }
        fn word(&mut self) -> Result<usize, String> {
            Ok(u32::from_le_bytes(
                self.take(4)?
                    .try_into()
                    .map_err(|_| "invalid shared value")?,
            ) as usize)
        }
        fn value(&mut self) -> Result<SharedValue, String> {
            Ok(match self.take(1)?[0] {
                0 => SharedValue::Nil,
                1 => SharedValue::Boolean(self.take(1)?[0] != 0),
                2 => SharedValue::Number(f64::from_le_bytes(
                    self.take(8)?
                        .try_into()
                        .map_err(|_| "invalid shared value")?,
                )),
                3 => {
                    let size = self.word()?;
                    SharedValue::Bytes(self.take(size)?.to_vec())
                }
                4 => {
                    let size = self.word()?;
                    let mut values = Vec::new();
                    values
                        .try_reserve_exact(size)
                        .map_err(|_| "cannot decode shared array")?;
                    for _ in 0..size {
                        values.push(self.value()?);
                    }
                    SharedValue::Array(values)
                }
                5 => {
                    let size = self.word()?;
                    let mut values = BTreeMap::new();
                    for _ in 0..size {
                        let size = self.word()?;
                        let key = std::str::from_utf8(self.take(size)?)
                            .map_err(|_| "invalid shared key")?
                            .to_owned();
                        values.insert(key, self.value()?);
                    }
                    SharedValue::Object(values)
                }
                6 => {
                    let index = self.word()?;
                    self.holds
                        .get(index)
                        .ok_or("invalid shared handle")?
                        .clone()
                }
                _ => return Err("invalid shared value tag".into()),
            })
        }
    }
    let mut reader = Reader {
        bytes,
        at: 0,
        holds,
    };
    let value = reader.value()?;
    if reader.at != bytes.len() {
        return Err("invalid shared value suffix".into());
    }
    Ok(value)
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
