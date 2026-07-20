use crate::enums::event_type::EventType;
use crate::records::event::Event;
use crate::records::event::EventData;
use crate::records::thread_context::ThreadContext;

impl ThreadContext {
    pub fn event_leave_u32(&mut self, microsec: u32) {
        self.events.push(Event {
            r#type: EventType::Leave,
            token: 0,
            data: EventData { microsec },
        });

        if self.events.len() > Self::kEventFlushLimit {
            self.flush_events();
        }
    }
}
