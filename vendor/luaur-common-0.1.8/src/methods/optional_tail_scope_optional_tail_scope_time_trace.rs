use crate::functions::get_clock_microseconds::get_clock_microseconds;
use crate::functions::get_thread_context::get_thread_context;
use crate::records::optional_tail_scope::OptionalTailScope;
use crate::FFlag::DebugLuauTimeTracing;

impl OptionalTailScope {
    pub fn optional_tail_scope(token: u16, threshold: u32) -> Self {
        let context = get_thread_context();
        let mut scope = Self {
            context: context as *mut _,
            token,
            threshold,
            microsec: 0,
            pos: 0,
        };

        if DebugLuauTimeTracing.get() {
            scope.pos = context.events.len() as u32;
            scope.microsec = get_clock_microseconds();
        }

        scope
    }
}
