//! Node: `cxx:Record:Luau.Common:Common/src/TimeTrace.cpp:92:global_context`
//! Source: `Common/src/TimeTrace.cpp:92-110` (hand-ported)
//!
//! C++ `struct GlobalContext` holds a `std::mutex` guarding the mutable trace
//! state (`threads`, `nextThreadId`, `tokens`, `traceFile`). The Rust port keeps
//! that state behind a `std::sync::Mutex` so the singleton is safely shareable
//! through `Arc`, exactly as the C++ `std::scoped_lock(context.mutex)` does.

extern crate alloc;

use crate::records::thread_context::ThreadContext;
use crate::records::token::Token;
use alloc::vec::Vec;
use std::sync::Mutex;

/// `std::vector<ThreadContext*>`. The raw pointers reference thread-owned
/// `ThreadContext`s exactly as the C++ vector does; wrapped so `Send` holds
/// for the `Mutex`-guarded state.
#[derive(Debug, Default)]
pub(crate) struct ThreadPtr(pub(crate) *mut ThreadContext);

// Safety: mirrors the C++ `GlobalContext`, where the same `ThreadContext*`
// pointers are shared across threads under the protection of `context.mutex`.
unsafe impl Send for ThreadPtr {}

#[derive(Debug, Default)]
pub(crate) struct GlobalContextState {
    pub(crate) threads: Vec<ThreadPtr>,
    pub(crate) next_thread_id: u32,
    pub(crate) tokens: Vec<Token>,
    pub(crate) trace_file: Option<std::fs::File>,
}

#[derive(Debug)]
pub struct GlobalContext {
    pub(crate) state: Mutex<GlobalContextState>,
}
