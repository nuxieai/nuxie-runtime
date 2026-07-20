//! Source: `Common/include/Luau/TimeTrace.h:74-80` (hand-ported)
//! C++ `~ThreadContext() { if (!events.empty()) flushEvents(); releaseThread(*globalContext, this); }`.
use crate::functions::release_thread::release_thread;
use crate::records::thread_context::ThreadContext;

impl ThreadContext {
    pub fn drop(&mut self) {
        if !self.events.is_empty() {
            self.flush_events();
        }

        // `releaseThread(*globalContext, this)` — the singleton guards its own
        // mutable state behind a Mutex, so a shared `&` suffices. Clone the Arc
        // first so we can also pass `self` as the thread pointer.
        let global_context = self.global_context.clone();
        release_thread(&global_context, self as *mut ThreadContext);
    }
}
