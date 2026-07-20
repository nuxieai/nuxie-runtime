#[cfg(feature = "luau_enable_time_trace")]
use crate::functions::create_thread::create_thread;
use crate::functions::get_global_context::get_global_context;
use crate::records::thread_context::ThreadContext;

impl ThreadContext {
    pub fn thread_context() -> Self {
        #[cfg(not(feature = "luau_enable_time_trace"))]
        {
            ThreadContext {
                global_context: get_global_context(),
                thread_id: 0,
                events: Vec::new(),
                data: Vec::new(),
            }
        }

        #[cfg(feature = "luau_enable_time_trace")]
        {
            // C++: `ThreadContext() : globalContext(getGlobalContext()) { threadId = createThread(*globalContext, this); }`
            let global_context = get_global_context();
            let mut result = ThreadContext {
                global_context: global_context.clone(),
                thread_id: 0,
                events: Vec::new(),
                data: Vec::new(),
            };

            // `createThread(*globalContext, this)` — the singleton context guards
            // its own mutable state behind a Mutex, so a shared `&` suffices.
            result.thread_id = create_thread(&global_context, &mut result as *mut ThreadContext);

            result
        }
    }
}
