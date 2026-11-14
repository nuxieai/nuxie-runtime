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
        // Both fit: each is at most its maximum less what is charged.
        state.bytes = state.bytes.saturating_add(bytes);
        state.objects = state.objects.saturating_add(objects);
        Ok(Charge {
            budget: self.clone(),
            bytes,
            objects,
        })
    }
    fn is_closed(&self) -> bool {
        lock(&self.state).closed
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
        // A charge is refunded once, so this never goes below zero.
        state.bytes = state.bytes.saturating_sub(self.bytes);
        state.objects = state.objects.saturating_sub(self.objects);
    }
}
/// The shared handles one ring record or one snapshot publication keeps
/// alive.
///
/// A ring or snapshot can hold the last handle of another ring or snapshot,
/// which holds the last handle of another, and so on: a script builds such a
/// chain with `s:publish(prev)` in a loop. Dropping the links one inside the
/// other would recurse once per link and overflow the stack, so the drop
/// below walks the chain with a list: before a link it owns outright drops,
/// its own holds are moved onto the list, and the link then drops with no
/// holds left. A link someone else still owns only loses one reference.
///
/// The values' storage is charged to the budget of the ring or snapshot that
/// keeps them, at its capacity, plus `extra` bytes for the entry that holds
/// the list (a ring's map entry); the record's own index words are in the
/// ring's bytes.
#[derive(Default)]
struct Holds {
    values: Vec<SharedValue>,
    _charge: Option<Charge>,
}
impl Holds {
    fn new(values: Vec<SharedValue>, budget: &Arc<SharedBudget>, extra: usize) -> Result<Self, String> {
        let bytes = values
            .capacity()
            .saturating_mul(std::mem::size_of::<SharedValue>())
            .saturating_add(extra);
        let charge = budget.charge(bytes, 0)?;
        Ok(Self {
            values,
            _charge: Some(charge),
        })
    }
}
/// A ring's map entry for one record's holds: its key and its value.
const RING_HOLDS_ENTRY_BYTES: usize =
    std::mem::size_of::<usize>().saturating_add(std::mem::size_of::<Holds>());
impl Drop for Holds {
    fn drop(&mut self) {
        let mut pending = std::mem::take(&mut self.values);
        while let Some(value) = pending.pop() {
            take_holds(value, &mut pending);
        }
    }
}
/// Moves the holds of `value`, where this was its last owner, onto `pending`,
/// then drops `value`.
fn take_holds(value: SharedValue, pending: &mut Vec<SharedValue>) {
    match value {
        SharedValue::Array(values) => pending.extend(values),
        SharedValue::Object(values) => pending.extend(values.into_values()),
        SharedValue::Ring(ring) => {
            let Some(ring) = Arc::into_inner(ring) else {
                return;
            };
            let Some(state) = Arc::into_inner(ring.state) else {
                return;
            };
            let mut state = state
                .into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for holds in state.holds.values_mut() {
                pending.append(&mut holds.values);
            }
        }
        SharedValue::Snapshot(snapshot) => {
            let Some(snapshot) = Arc::into_inner(snapshot) else {
                return;
            };
            let Some(state) = Arc::into_inner(snapshot.state) else {
                return;
            };
            let mut writer = state
                .writer
                .into_inner()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for holds in &mut writer.holds {
                pending.append(&mut holds.values);
            }
        }
        SharedValue::Nil
        | SharedValue::Boolean(_)
        | SharedValue::Number(_)
        | SharedValue::Bytes(_)
        | SharedValue::Blob(_)
        | SharedValue::Region(_) => {}
    }
}
/// Bytes after a record's payload that carries shared holds: one index word
/// per hold and the count word.
fn layout_trailer_bytes(holds: usize) -> usize {
    holds.saturating_mul(4).saturating_add(4)
}
fn allocate_bytes(capacity: usize) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(capacity)
        .map_err(|_| "cannot allocate shared storage")?;
    bytes.resize(capacity, 0);
    Ok(bytes)
}
/// Ring storage. A position is always below the capacity; every copy walks
/// the storage from a position around the end, so no byte index is computed.
struct RingBytes(Box<[u8]>);
impl RingBytes {
    fn capacity(&self) -> usize {
        self.0.len()
    }
    /// The storage from `at` to the end, then from the start to `at`.
    fn from(&self, at: usize) -> impl Iterator<Item = &u8> {
        let at = at.min(self.capacity());
        let (front, back) = self.0.split_at_checked(at).unwrap_or_default();
        back.iter().chain(front)
    }
    fn from_mut(&mut self, at: usize) -> impl Iterator<Item = &mut u8> {
        let at = at.min(self.capacity());
        let (front, back) = self.0.split_at_mut_checked(at).unwrap_or_default();
        back.iter_mut().chain(front)
    }
    fn write(&mut self, at: usize, bytes: &[u8]) {
        for (slot, byte) in self.from_mut(at).zip(bytes) {
            *slot = *byte;
        }
    }
    fn read(&self, at: usize, out: &mut [u8]) {
        for (byte, slot) in out.iter_mut().zip(self.from(at)) {
            *byte = *slot;
        }
    }
    fn word(&self, at: usize) -> u32 {
        let mut bytes = [0; 4];
        self.read(at, &mut bytes);
        u32::from_le_bytes(bytes)
    }
    /// The position `count` bytes after `at`, wrapped; `count` is at most the
    /// capacity for every record this module frames.
    fn advance(&self, at: usize, count: usize) -> usize {
        let to_end = self.capacity().saturating_sub(at);
        match count.checked_sub(to_end) {
            Some(past) => past.checked_rem(self.capacity()).unwrap_or(0),
            None => at.saturating_add(count),
        }
    }
}
struct RingState {
    bytes: RingBytes,
    head: usize,
    used: usize,
    // Keys are record offsets, which cannot collide while the records are live.
    // This map is touched only for messages containing main-lane shared handles.
    holds: BTreeMap<usize, Holds>,
    closed: bool,
}
impl RingState {
    /// Appends one record made of `parts` if it fits; returns where it starts.
    fn push(&mut self, parts: &[&[u8]]) -> Option<usize> {
        let size = parts
            .iter()
            .try_fold(0usize, |size, part| size.checked_add(part.len()))?;
        let free = self.bytes.capacity().checked_sub(self.used)?;
        if self.closed || size > free {
            return None;
        }
        let start = self.bytes.advance(self.head, self.used);
        let mut at = start;
        for part in parts {
            self.bytes.write(at, part);
            at = self.bytes.advance(at, part.len());
        }
        self.used = self.used.checked_add(size)?;
        Some(start)
    }
    /// Removes the record of `size` bytes, its word included, at the head.
    fn pop(&mut self, size: usize) {
        self.head = self.bytes.advance(self.head, size);
        self.used = self.used.saturating_sub(size);
    }
}
pub struct SharedRing {
    capacity: usize,
    state: Arc<Mutex<RingState>>,
    charge: Charge,
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
            bytes: RingBytes(allocate_bytes(capacity)?.into_boxed_slice()),
            head: 0,
            used: 0,
            holds: BTreeMap::new(),
            closed: false,
        }));
        budget.register(SharedWeak::Ring(Arc::downgrade(&state)))?;
        Ok(Self {
            capacity,
            state,
            charge,
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
        // A record fits in a ring of at most `MAX_SHARED_BYTES`, so its length
        // never reaches the hold flag in bit 31.
        let Some(word) = u32::try_from(bytes.len())
            .ok()
            .filter(|word| *word & (1 << 31) == 0)
        else {
            return Ok(false);
        };
        Ok(lock(&self.state)
            .push(&[&word.to_le_bytes(), bytes])
            .is_some())
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
        let word = state.bytes.word(state.head);
        if word & (1 << 31) != 0 {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        // Bit 31 is clear, so the size and size + 1 fit in a u32.
        let size = word as usize;
        let reported = size.saturating_add(1);
        let Some(out) = out.get_mut(..size) else {
            return Ok(reported);
        };
        let payload = state.bytes.advance(state.head, 4);
        state.bytes.read(payload, out);
        state.pop(size.saturating_add(4));
        Ok(reported)
    }
    pub fn send(&self, value: SharedValue) -> Result<bool, String> {
        let (payload, _) = value.layout()?;
        let size = payload.checked_add(4).ok_or("shared value too large")?;
        if size > self.capacity {
            return Ok(false);
        }
        let encoded = encode(&value)?;
        // The ring holds at most `MAX_SHARED_BYTES`, so the payload length
        // and every hold index fit below bit 31.
        let length = u32::try_from(payload).map_err(|_| "shared value too large")?;
        let word = if encoded.holds.is_empty() {
            length
        } else {
            length | (1 << 31)
        };
        let mut trailer = Vec::new();
        if !encoded.holds.is_empty() {
            let count = u32::try_from(encoded.holds.len()).map_err(|_| "shared value too large")?;
            trailer
                .try_reserve_exact(layout_trailer_bytes(encoded.holds.len()))
                .map_err(|_| "cannot encode shared value")?;
            for index in 0..count {
                trailer.extend_from_slice(&index.to_le_bytes());
            }
            trailer.extend_from_slice(&count.to_le_bytes());
        }
        // Charged before the ring lock, and refunded (outside it) if the
        // record does not fit.
        let holds = if encoded.holds.is_empty() {
            None
        } else {
            match Holds::new(encoded.holds, &self.charge.budget, RING_HOLDS_ENTRY_BYTES) {
                Ok(holds) => Some(holds),
                // A released owner's ring refuses the record, as before.
                Err(_) if self.charge.budget.is_closed() => return Ok(false),
                Err(error) => return Err(error),
            }
        };
        let mut state = lock(&self.state);
        let Some(tail) = state.push(&[&word.to_le_bytes(), &encoded.bytes, &trailer]) else {
            drop(state);
            return Ok(false);
        };
        if let Some(holds) = holds {
            state.holds.insert(tail, holds);
        }
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
            let word = state.bytes.word(head);
            let payload = (word & !(1 << 31)) as usize;
            let has_holds = word & (1 << 31) != 0;
            if processor && has_holds {
                return Err(PROCESSOR_HOLDS_ERROR.into());
            }
            let start = state.bytes.advance(head, 4);
            // The hold count is the payload's last word.
            let size = if has_holds {
                let count_at = state
                    .bytes
                    .advance(start, payload.checked_sub(4).ok_or("invalid shared value")?);
                let hold_count = state.bytes.word(count_at) as usize;
                payload
                    .checked_sub(layout_trailer_bytes(hold_count))
                    .ok_or("invalid shared value")?
            } else {
                payload
            };
            if encoded.capacity() < size {
                drop(state);
                encoded
                    .try_reserve_exact(size)
                    .map_err(|_| "cannot read shared ring")?;
                continue;
            }
            encoded.clear();
            encoded.extend(state.bytes.from(start).take(size));
            let holds = state.holds.remove(&head).unwrap_or_default();
            state.pop(payload.saturating_add(4));
            drop(state);
            return decode(&encoded, &holds.values).map(Some);
        }
    }
    pub fn try_receive(&self) -> Result<Option<SharedValue>, String> {
        self.read(false)
    }
    #[cfg(test)]
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
/// One of a snapshot's two copies. `SnapshotState::published` stores it as
/// its number (1 or 2), zero before the first publication.
#[derive(Clone, Copy, PartialEq)]
enum Slot {
    First,
    Second,
}
impl Slot {
    fn published(value: usize) -> Option<Self> {
        match value {
            0 => None,
            1 => Some(Self::First),
            _ => Some(Self::Second),
        }
    }
    fn number(self) -> usize {
        match self {
            Self::First => 1,
            Self::Second => 2,
        }
    }
    fn of<T>(self, pair: &[T; 2]) -> &T {
        let [first, second] = pair;
        match self {
            Self::First => first,
            Self::Second => second,
        }
    }
    fn of_mut<T>(self, pair: &mut [T; 2]) -> &mut T {
        let [first, second] = pair;
        match self {
            Self::First => first,
            Self::Second => second,
        }
    }
}
impl SnapshotCopy {
    /// The first `size` bytes; the copy holds `capacity` bytes and `size` is
    /// clamped to it by the reader.
    fn load(&self, size: usize) -> impl Iterator<Item = u8> + '_ {
        self.bytes
            .iter()
            .take(size)
            .map(|value| value.load(Ordering::SeqCst))
    }
}
struct SnapshotWriter {
    sequence: u64,
    holds: [Holds; 2],
    closed: bool,
}
struct SnapshotState {
    copies: [SnapshotCopy; 2],
    // Zero until the first publication; otherwise `Slot::number`.
    published: AtomicUsize,
    writer: Mutex<SnapshotWriter>,
}
impl SnapshotState {
    fn published(&self) -> Option<Slot> {
        Slot::published(self.published.load(Ordering::SeqCst))
    }
}
pub struct SharedSnapshot {
    capacity: usize,
    state: Arc<SnapshotState>,
    charge: Charge,
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
                holds: [Holds::default(), Holds::default()],
                closed: false,
            }),
        });
        budget.register(SharedWeak::Snapshot(Arc::downgrade(&state)))?;
        Ok(Self {
            capacity,
            state,
            charge,
        })
    }
    pub fn capacity(&self) -> usize {
        self.capacity
    }
    pub fn seq(&self) -> u64 {
        loop {
            let Some(published) = self.state.published() else {
                return 0;
            };
            let copy = published.of(&self.state.copies);
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
        let has_holds = !encoded.holds.is_empty();
        // Charged before the writer lock; a refusal below refunds it after.
        let holds = if has_holds {
            match Holds::new(encoded.holds, &self.charge.budget, 0) {
                Ok(holds) => holds,
                // A released owner's snapshot refuses the value, as before.
                Err(_) if self.charge.budget.is_closed() => return Ok(false),
                Err(error) => return Err(error),
            }
        } else {
            Holds::default()
        };
        let mut writer = lock(&self.state.writer);
        if writer.closed {
            drop(writer);
            return Ok(false);
        }
        let published = self.state.published();
        if processor && published.is_some_and(|slot| !slot.of(&writer.holds).values.is_empty()) {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        let index = if published == Some(Slot::First) {
            Slot::Second
        } else {
            Slot::First
        };
        let copy = index.of(&self.state.copies);
        copy.version.fetch_add(1, Ordering::SeqCst);
        for (target, byte) in copy.bytes.iter().zip(&encoded.bytes) {
            target.store(*byte, Ordering::SeqCst);
        }
        copy.size.store(encoded.bytes.len(), Ordering::SeqCst);
        copy.has_holds.store(has_holds, Ordering::SeqCst);
        writer.sequence = writer.sequence.wrapping_add(1);
        copy.sequence.store(writer.sequence, Ordering::SeqCst);
        let old = std::mem::replace(index.of_mut(&mut writer.holds), holds);
        copy.version.fetch_add(1, Ordering::SeqCst);
        self.state.published.store(index.number(), Ordering::SeqCst);
        // The prior publication is no longer live. Keep neither duplicate holds
        // nor their destruction on the publisher lock.
        let previous = published
            .map(|slot| std::mem::take(slot.of_mut(&mut writer.holds)))
            .unwrap_or_default();
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
            let Some(published) = self.state.published() else {
                return Ok(None);
            };
            let copy = published.of(&self.state.copies);
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
                let Some(published) = self.state.published() else {
                    return Ok(None);
                };
                let copy = published.of(&self.state.copies);
                let sequence = copy.sequence.load(Ordering::SeqCst);
                let size = copy.size.load(Ordering::SeqCst).min(self.capacity);
                bytes.clear();
                bytes
                    .try_reserve_exact(size)
                    .map_err(|_| "cannot read shared snapshot")?;
                bytes.extend(copy.load(size));
                let holds = published.of(&writer.holds).values.clone();
                drop(writer);
                return decode(&bytes, &holds).map(|value| Some((sequence, value)));
            }
            bytes.clear();
            bytes
                .try_reserve_exact(size)
                .map_err(|_| "cannot read shared snapshot")?;
            bytes.extend(copy.load(size));
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
    #[cfg(test)]
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
    fn nested_arrays(depth: usize) -> Vec<u8> {
        let mut bytes = Vec::new();
        for _ in 0..depth {
            bytes.extend_from_slice(&[4, 1, 0, 0, 0]);
        }
        bytes.push(0);
        bytes
    }
    #[test]
    fn decode_refuses_nesting_past_the_shared_depth_limit() {
        assert!(decode(&nested_arrays(MAX_SHARED_DEPTH), &[]).is_ok());
        let error = decode(&nested_arrays(MAX_SHARED_DEPTH + 1), &[]).unwrap_err();
        assert!(error.contains("nesting exceeds 64 tables"), "{error}");
    }
    #[test]
    fn decode_refuses_more_values_than_the_shared_node_limit() {
        // An array of MAX_SHARED_NODES nils is MAX_SHARED_NODES + 1 values.
        let mut bytes = vec![4];
        bytes.extend_from_slice(&u32::try_from(MAX_SHARED_NODES).unwrap().to_le_bytes());
        bytes.resize(5 + MAX_SHARED_NODES, 0);
        let error = decode(&bytes, &[]).unwrap_err();
        assert!(error.contains("exceeds 4194304 values"), "{error}");
        bytes[1..5].copy_from_slice(&u32::try_from(MAX_SHARED_NODES - 1).unwrap().to_le_bytes());
        bytes.pop();
        assert!(decode(&bytes, &[]).is_ok());
    }
    #[test]
    fn decode_refuses_container_counts_the_input_cannot_hold() {
        for tag in [4u8, 5] {
            let mut bytes = vec![tag];
            bytes.extend_from_slice(&(1u32 << 20).to_le_bytes());
            bytes.extend_from_slice(&[0; 64]);
            assert_eq!(decode(&bytes, &[]).unwrap_err(), "invalid shared value");
        }
    }
    #[test]
    fn decoding_a_raw_record_fails_that_message_only() {
        // Raw PCM records travel in rings too; reading one as a shared value
        // must fail the message, never the thread.
        let ring = SharedRing::new(1 << 12).unwrap();
        assert!(ring.write_record(&nested_arrays(MAX_SHARED_DEPTH + 1)).unwrap());
        assert!(ring.send(SharedValue::Number(1.0)).unwrap());
        assert!(ring.try_receive().is_err());
        assert!(matches!(ring.try_receive(), Ok(Some(SharedValue::Number(1.0)))));
    }
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
        // The ring's bytes and the one hold its record keeps fill the budget.
        let hold = std::mem::size_of::<SharedValue>() + RING_HOLDS_ENTRY_BYTES;
        let budget = Arc::new(SharedBudget::new(64 + hold));
        let ring = Arc::new(SharedRing::with_budget(64, budget.clone()).unwrap());
        let weak = Arc::downgrade(&ring);
        assert!(ring.send(SharedValue::Ring(ring.clone())).unwrap());
        assert!(SharedSnapshot::with_budget(1, budget.clone()).is_err());
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
        assert_eq!(lock(&ring.state).bytes.0.len(), 1024);
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
    /// Runs `f` on a thread with a 256 KiB stack: a drop that recursed once
    /// per link of a 20,000-link chain overflows it and aborts the process.
    fn on_small_stack(f: impl FnOnce() + Send + 'static) {
        std::thread::Builder::new()
            .stack_size(256 << 10)
            .spawn(f)
            .unwrap()
            .join()
            .unwrap();
    }
    const CHAIN_LINKS: usize = 20_000;
    #[test]
    fn dropping_a_long_snapshot_hold_chain_does_not_recurse() {
        // Each snapshot publishes the previous one, as `s:publish(prev)` in a
        // script loop does; the head owns the whole chain.
        let mut head = Arc::new(SharedSnapshot::new(16).unwrap());
        for _ in 0..CHAIN_LINKS {
            let next = Arc::new(SharedSnapshot::new(16).unwrap());
            assert!(next.publish(SharedValue::Snapshot(head)).unwrap());
            head = next;
        }
        on_small_stack(move || drop(head));
    }
    #[test]
    fn dropping_a_long_ring_hold_chain_does_not_recurse() {
        let mut head = Arc::new(SharedRing::new(64).unwrap());
        for _ in 0..CHAIN_LINKS {
            let next = Arc::new(SharedRing::new(64).unwrap());
            // Rings and snapshots alternate inside containers, too.
            let value = SharedValue::Array(vec![SharedValue::Object(BTreeMap::from([(
                "next".to_owned(),
                SharedValue::Ring(head),
            )]))]);
            assert!(next.send(value).unwrap());
            head = next;
        }
        on_small_stack(move || drop(head));
    }
    #[test]
    fn closing_a_budget_with_a_long_hold_chain_does_not_recurse() {
        // Registered head first, so `close` reaches the head while every
        // other link is owned only by its predecessor's holds.
        let budget = Arc::new(SharedBudget::default());
        let links: Vec<_> = (0..CHAIN_LINKS)
            .map(|_| Arc::new(SharedSnapshot::with_budget(16, budget.clone()).unwrap()))
            .collect();
        for pair in links.windows(2) {
            assert!(pair[0].publish(SharedValue::Snapshot(pair[1].clone())).unwrap());
        }
        let mut links = links.into_iter();
        let head = links.next().unwrap();
        drop(links);
        let weak = Arc::downgrade(&head);
        on_small_stack(move || {
            budget.close();
            drop(head);
        });
        assert!(weak.upgrade().is_none());
    }
    #[test]
    fn ring_and_snapshot_holds_are_charged_at_their_size() {
        // They were charged nothing past the ring's bytes (9 per handle).
        let budget = Arc::new(SharedBudget::default());
        let ring = SharedRing::with_budget(4096, budget.clone()).unwrap();
        let snapshot = SharedSnapshot::with_budget(4096, budget.clone()).unwrap();
        let blob = Arc::new(SharedBlob::with_budget(&[1, 2, 3], budget.clone()).unwrap());
        let handles = || SharedValue::Array((0..100).map(|_| SharedValue::Blob(blob.clone())).collect());
        let bytes = || lock(&budget.state).bytes;
        let held = 100 * std::mem::size_of::<SharedValue>();
        let base = bytes();
        assert!(ring.send(handles()).unwrap());
        assert!(bytes() - base >= held + RING_HOLDS_ENTRY_BYTES, "{}", bytes() - base);
        assert!(ring.receive().is_some());
        assert_eq!(bytes(), base);
        assert!(snapshot.publish(handles()).unwrap());
        assert!(bytes() - base >= held, "{}", bytes() - base);
        // The next publication retires the previous holds.
        assert!(snapshot.publish(SharedValue::Nil).unwrap());
        assert_eq!(bytes(), base);
    }
    #[test]
    fn holds_past_the_shared_budget_are_refused() {
        let budget = Arc::new(SharedBudget::new(4096 + 2048 + 3));
        let ring = SharedRing::with_budget(4096, budget.clone()).unwrap();
        let snapshot = SharedSnapshot::with_budget(1024, budget.clone()).unwrap();
        let blob = Arc::new(SharedBlob::with_budget(&[1, 2, 3], budget.clone()).unwrap());
        let handles = SharedValue::Array((0..100).map(|_| SharedValue::Blob(blob.clone())).collect());
        assert_eq!(ring.send(handles.clone()).unwrap_err(), "shared memory budget exceeded");
        assert_eq!(snapshot.publish(handles).unwrap_err(), "shared memory budget exceeded");
        assert_eq!(ring.used(), 0);
        assert!(snapshot.latest().is_none());
    }
    #[test]
    fn a_dropped_chain_keeps_the_holds_of_links_still_owned_elsewhere() {
        let first = Arc::new(SharedSnapshot::new(16).unwrap());
        assert!(first.publish(SharedValue::Number(3.0)).unwrap());
        let second = Arc::new(SharedSnapshot::new(16).unwrap());
        assert!(second.publish(SharedValue::Snapshot(first)).unwrap());
        let third = Arc::new(SharedSnapshot::new(16).unwrap());
        assert!(third.publish(SharedValue::Snapshot(second.clone())).unwrap());
        drop(third);
        let Some((_, SharedValue::Snapshot(first))) = second.latest() else {
            panic!("the second snapshot lost its publication");
        };
        assert!(matches!(first.latest(), Some((1, SharedValue::Number(3.0)))));
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
        let mut copy = Vec::new();
        copy.try_reserve_exact(bytes.len())
            .map_err(|_| "cannot allocate shared storage")?;
        copy.extend_from_slice(bytes);
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
        let source = if self.size() == 0 {
            &[]
        } else {
            self.bytes.get(offset..).unwrap_or_default()
        };
        let mut count = 0usize;
        for (byte, value) in out.iter_mut().zip(source) {
            *byte = *value;
            count = count.saturating_add(1);
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
        self.words
            .get(offset / 4)
            .ok_or_else(|| "invalid shared region atomic offset".into())
    }
    /// The bytes from `offset` on, each with the word that holds it and its
    /// byte position in that word (little-endian).
    fn bytes_from(&self, offset: usize) -> impl Iterator<Item = (&AtomicU32, usize)> {
        self.words
            .get(offset / 4..)
            .unwrap_or_default()
            .iter()
            .flat_map(|word| (0..4).map(move |lane| (word, lane)))
            .skip(offset % 4)
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
        for (byte, (word, lane)) in out.iter_mut().zip(self.0.bytes_from(offset)) {
            let bytes = word.load(Ordering::SeqCst).to_le_bytes();
            *byte = bytes.get(lane).copied().unwrap_or_default();
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
        for (byte, (word, lane)) in input.iter().zip(self.0.bytes_from(offset)) {
            // The update always returns `Some`, so it cannot fail.
            let _ = word.fetch_update(Ordering::SeqCst, Ordering::SeqCst, |old| {
                let mut bytes = old.to_le_bytes();
                if let Some(slot) = bytes.get_mut(lane) {
                    *slot = *byte;
                }
                Some(u32::from_le_bytes(bytes))
            });
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
