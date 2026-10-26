//! Shared values for isolated Luau audio processors.
//!
//! The payload codec is a Luau adaptation: one-byte tags, little-endian u32
//! lengths/handles, f64 numbers, and recursively encoded arrays/objects. Ring
//! framing and shared-hold charges follow upstream (length + payload + holds).
//! No main-VM Lua value crosses a thread. Locks are acquired by try/spin, as in
//! upstream SharedRing's lock word; no callback waits on a sleeping mutex.
use std::collections::{BTreeMap, VecDeque};
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
    Snapshot(Weak<Mutex<SnapshotState>>),
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
                        let records = {
                            let mut state = lock(&value);
                            state.closed = true;
                            state.used = 0;
                            std::mem::take(&mut state.records)
                        };
                        drop(records);
                    }
                }
                SharedWeak::Snapshot(value) => {
                    if let Some(value) = value.upgrade() {
                        let old = {
                            let mut state = lock(&value);
                            state.closed = true;
                            state.latest.take()
                        };
                        drop(old);
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
struct Record {
    value: SharedValue,
    bytes: usize,
    holds: usize,
}
struct RingState {
    records: VecDeque<Record>,
    used: usize,
    closed: bool,
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
        // Even nil needs a one-byte tag and four-byte record length. Reserving
        // all possible record slots here keeps queue growth off the callback.
        let mut records = VecDeque::new();
        records
            .try_reserve_exact(capacity / 5)
            .map_err(|_| "cannot allocate shared ring")?;
        let state = Arc::new(Mutex::new(RingState {
            records,
            used: 0,
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
        let (payload, holds) = value.layout()?;
        let bytes = payload.checked_add(4).ok_or("shared value too large")?;
        if bytes > self.capacity {
            return Ok(false);
        }
        let mut state = lock(&self.state);
        if state.closed || bytes > self.capacity - state.used {
            return Ok(false);
        }
        state.records.push_back(Record {
            value,
            bytes,
            holds,
        });
        state.used += bytes;
        Ok(true)
    }
    pub fn send_from_processor(&self, value: SharedValue) -> Result<bool, String> {
        if value.contains_handles() {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        self.send(value)
    }
    pub fn receive(&self) -> Option<SharedValue> {
        let mut state = lock(&self.state);
        let record = state.records.pop_front()?;
        state.used -= record.bytes;
        Some(record.value)
    }
    pub fn receive_for_processor(&self) -> Result<Option<SharedValue>, String> {
        let mut state = lock(&self.state);
        if state
            .records
            .front()
            .is_some_and(|record| record.holds != 0)
        {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        let Some(record) = state.records.pop_front() else {
            return Ok(None);
        };
        state.used -= record.bytes;
        Ok(Some(record.value))
    }
}
struct SnapshotState {
    latest: Option<Arc<Record>>,
    sequence: u64,
    closed: bool,
}
pub struct SharedSnapshot {
    capacity: usize,
    state: Arc<Mutex<SnapshotState>>,
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
        let state = Arc::new(Mutex::new(SnapshotState {
            latest: None,
            sequence: 0,
            closed: false,
        }));
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
        lock(&self.state).sequence
    }
    pub fn publish(&self, value: SharedValue) -> Result<bool, String> {
        let (bytes, holds) = value.layout()?;
        if bytes > self.capacity {
            return Ok(false);
        }
        let record = Arc::new(Record {
            value,
            bytes,
            holds,
        });
        let previous = {
            let mut state = lock(&self.state);
            if state.closed {
                return Ok(false);
            }
            state.sequence = state.sequence.wrapping_add(1);
            state.latest.replace(record)
        };
        // Replaced handles may own transport objects; destroy outside the lock.
        drop(previous);
        Ok(true)
    }
    pub fn publish_from_processor(&self, value: SharedValue) -> Result<bool, String> {
        if value.contains_handles() {
            return Err(PROCESSOR_HOLDS_ERROR.into());
        }
        let (bytes, _) = value.layout()?;
        if bytes > self.capacity {
            return Ok(false);
        }
        let record = Arc::new(Record {
            value,
            bytes,
            holds: 0,
        });
        let previous = {
            let mut state = lock(&self.state);
            if state.closed {
                return Ok(false);
            }
            if state.latest.as_ref().is_some_and(|value| value.holds != 0) {
                return Err(PROCESSOR_HOLDS_ERROR.into());
            }
            state.sequence = state.sequence.wrapping_add(1);
            state.latest.replace(record)
        };
        drop(previous);
        Ok(true)
    }
    pub fn latest(&self) -> Option<(u64, SharedValue)> {
        let (sequence, record) = {
            let state = lock(&self.state);
            (state.sequence, state.latest.as_ref()?.clone())
        };
        Some((sequence, record.value.clone()))
    }
    pub fn latest_for_processor(&self) -> Result<Option<(u64, SharedValue)>, String> {
        let (sequence, record) = {
            let state = lock(&self.state);
            let Some(record) = state.latest.as_ref() else {
                return Ok(None);
            };
            if record.holds != 0 {
                return Err(PROCESSOR_HOLDS_ERROR.into());
            }
            (state.sequence, record.clone())
        };
        Ok(Some((sequence, record.value.clone())))
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
        assert!(ring
            .send(SharedValue::Array(vec![SharedValue::Snapshot(shared)]))
            .unwrap());
        let used = ring.used();
        assert!(ring
            .receive_for_processor()
            .unwrap_err()
            .contains("Audio.start"));
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
        assert!(snapshot
            .publish_from_processor(SharedValue::Number(1.0))
            .is_err());
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
                        assert!(ring
                            .send(SharedValue::Number((producer * 1000 + n) as f64))
                            .unwrap());
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
}
