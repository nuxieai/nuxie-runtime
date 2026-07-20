//! Source: `Common/src/TimeTrace.cpp:253-261` (hand-ported)
//! C++:
//! ```cpp
//! ThreadContext& getThreadContext()
//! {
//!     if (auto provider = threadContextProvider())
//!         return provider();
//!     thread_local ThreadContext context;
//!     return context;
//! }
//! ```
use crate::functions::thread_context_provider::thread_context_provider;
use crate::records::thread_context::ThreadContext;

thread_local! {
    // `thread_local ThreadContext context;` — boxed so its address is stable for
    // the life of the thread, allowing a `&'static mut` to be handed out exactly
    // as the C++ returns a reference to the thread-local object.
    static CONTEXT: core::cell::UnsafeCell<alloc::boxed::Box<ThreadContext>> =
        core::cell::UnsafeCell::new(alloc::boxed::Box::new(ThreadContext::thread_context()));
}

pub fn get_thread_context() -> &'static mut ThreadContext {
    // Check the custom provider, which might implement a custom TLS.
    let provider = *thread_context_provider();
    let provided = provider();
    if !provided.is_null() {
        return unsafe { &mut *provided };
    }

    CONTEXT.with(|cell| {
        // Safety: the boxed `ThreadContext` lives for the duration of the thread;
        // its heap address is stable, so a reference promoted to `'static` is sound
        // for single-threaded TimeTrace use (the C++ object has the same property).
        let boxed: &mut alloc::boxed::Box<ThreadContext> = unsafe { &mut *cell.get() };
        unsafe { &mut *(boxed.as_mut() as *mut ThreadContext) }
    })
}
