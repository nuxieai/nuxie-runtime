use crate::records::thread_context::ThreadContext;

impl ThreadContext {
    pub fn event_enter_u16(&mut self, token: u16) {
        self.event_enter_u16_u32(
            token,
            crate::functions::get_clock_microseconds::get_clock_microseconds(),
        );
    }
}
