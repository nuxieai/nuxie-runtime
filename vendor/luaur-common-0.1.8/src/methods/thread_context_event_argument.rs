use crate::enums::event_type::EventType;
use crate::records::event::Event;
use crate::records::event::EventData;
use crate::records::thread_context::ThreadContext;

impl ThreadContext {
    pub fn event_argument(
        &mut self,
        name: *const core::ffi::c_char,
        value: *const core::ffi::c_char,
    ) {
        let pos = self.data.len() as u32;
        unsafe {
            let mut p = name;
            while !p.is_null() && *p != 0 {
                self.data.push(*p);
                p = p.add(1);
            }
            self.data.push(0);
        }
        self.events.push(Event {
            r#type: EventType::ArgName,
            token: 0,
            data: EventData { microsec: pos },
        });

        let pos = self.data.len() as u32;
        unsafe {
            let mut p = value;
            while !p.is_null() && *p != 0 {
                self.data.push(*p);
                p = p.add(1);
            }
            self.data.push(0);
        }
        self.events.push(Event {
            r#type: EventType::ArgValue,
            token: 0,
            data: EventData { microsec: pos },
        });

        if self.events.len() > Self::kEventFlushLimit {
            self.flush_events();
        }
    }
}
