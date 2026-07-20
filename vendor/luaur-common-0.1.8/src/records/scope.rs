use crate::records::thread_context::ThreadContext;

#[allow(non_camel_case_types)]
#[derive(Debug)]
pub struct Scope {
    pub(crate) context: *mut ThreadContext,
}

impl Drop for Scope {
    fn drop(&mut self) {
        #[cfg(feature = "luau_enable_time_trace")]
        {
            if crate::FFlag::DebugLuauTimeTracing.get() {
                unsafe {
                    (*self.context).event_leave();
                }
            }
        }
    }
}
