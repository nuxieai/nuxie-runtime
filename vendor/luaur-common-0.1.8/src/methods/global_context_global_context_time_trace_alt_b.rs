//! Source: `Common/src/TimeTrace.cpp:107` (hand-ported)
//! C++ `GlobalContext() = default;` (the private default ctor used only by
//! `getGlobalContext`).
use crate::records::global_context::{GlobalContext, GlobalContextState};
use std::sync::Mutex;

impl GlobalContext {
    pub fn new() -> Self {
        GlobalContext {
            state: Mutex::new(GlobalContextState::default()),
        }
    }
}

impl Default for GlobalContext {
    fn default() -> Self {
        Self::new()
    }
}
