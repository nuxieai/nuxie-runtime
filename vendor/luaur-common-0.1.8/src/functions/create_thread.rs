//! Source: `Common/src/TimeTrace.cpp:125-132` (hand-ported)
//! C++:
//! ```cpp
//! uint32_t createThread(GlobalContext& context, ThreadContext* threadContext)
//! {
//!     std::scoped_lock lock(context.mutex);
//!     context.threads.push_back(threadContext);
//!     return ++context.nextThreadId;
//! }
//! ```
use crate::records::global_context::{GlobalContext, ThreadPtr};
use crate::records::thread_context::ThreadContext;

pub fn create_thread(context: &GlobalContext, thread_context: *mut ThreadContext) -> u32 {
    let mut state = context
        .state
        .lock()
        .expect("TimeTrace GlobalContext mutex poisoned");

    state.threads.push(ThreadPtr(thread_context));

    state.next_thread_id += 1;
    state.next_thread_id
}
