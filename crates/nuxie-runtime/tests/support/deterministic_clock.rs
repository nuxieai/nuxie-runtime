//! Upstream tests/include/deterministic_clock.hpp at ed330356.
use nuxie_runtime::File;
use std::{
    cell::Cell,
    sync::{Mutex, MutexGuard},
};

static CLOCK_LOCK: Mutex<()> = Mutex::new(());

pub struct DeterministicClock {
    previous: bool,
    time: Cell<f32>,
    _lock: MutexGuard<'static, ()>,
}
impl DeterministicClock {
    pub fn new() -> Self {
        let lock = CLOCK_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        let previous = File::deterministic_mode();
        File::set_deterministic_mode(true);
        Self {
            previous,
            time: Cell::new(0.0),
            _lock: lock,
        }
    }
    pub fn next(&self) -> f32 {
        let time = self.time.get() + 1.0 / 60.0;
        self.time.set(time);
        time
    }
}
impl Drop for DeterministicClock {
    fn drop(&mut self) {
        File::set_deterministic_mode(self.previous);
    }
}
