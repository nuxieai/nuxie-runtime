use crate::enums::event_type::EventType;
use crate::records::thread_context::ThreadContext;

impl ThreadContext {
    pub fn event_enter_u16_u32(&mut self, token: u16, microsec: u32) {
        self.events.push(crate::records::event::Event {
            r#type: EventType::Enter,
            token,
            data: crate::records::event::EventData { microsec },
        });
    }
}
