//! Source: `Common/src/TimeTrace.cpp:134-140` (hand-ported)
//! C++:
//! ```cpp
//! void releaseThread(GlobalContext& context, ThreadContext* threadContext)
//! {
//!     std::scoped_lock lock(context.mutex);
//!     if (auto it = std::find(context.threads.begin(), context.threads.end(), threadContext); it != context.threads.end())
//!         context.threads.erase(it);
//! }
//! ```
use crate::records::global_context::GlobalContext;
use crate::records::thread_context::ThreadContext;

pub fn release_thread(context: &GlobalContext, thread_context: *mut ThreadContext) {
    let mut state = context
        .state
        .lock()
        .expect("TimeTrace GlobalContext mutex poisoned");

    if let Some(pos) = state.threads.iter().position(|t| t.0 == thread_context) {
        state.threads.remove(pos);
    }
}
