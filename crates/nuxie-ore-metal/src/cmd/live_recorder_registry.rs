//! renderer/cmd/live_recorder_registry.hpp at e949498e.
use std::collections::HashSet;
use std::sync::{Mutex, MutexGuard, OnceLock};

fn registry() -> &'static Mutex<HashSet<usize>> {
    static REGISTRY: OnceLock<Mutex<HashSet<usize>>> = OnceLock::new();
    REGISTRY.get_or_init(|| Mutex::new(HashSet::new()))
}
pub fn recorder_registry() -> MutexGuard<'static, HashSet<usize>> {
    registry()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}
pub fn register_recorder(identity: usize) {
    recorder_registry().insert(identity);
}
pub fn unregister_recorder(identity: usize) {
    recorder_registry().remove(&identity);
}
