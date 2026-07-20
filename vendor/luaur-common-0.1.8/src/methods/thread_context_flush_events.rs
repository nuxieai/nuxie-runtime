//! Source: `Common/include/Luau/TimeTrace.h:82-93` (hand-ported)
//! C++ `ThreadContext::flushEvents()`.
use crate::enums::event_type::EventType;
use crate::functions::create_token::create_token;
use crate::functions::flush_events::flush_events;
use crate::functions::get_clock_microseconds::get_clock_microseconds;
use crate::records::event::Event;
use crate::records::event::EventData;
use crate::records::thread_context::ThreadContext;
use core::sync::atomic::{AtomicU16, Ordering};

impl ThreadContext {
    pub fn flush_events(&mut self) {
        // `static uint16_t flushToken = createToken(*globalContext, "flushEvents", "TimeTrace");`
        static FLUSH_TOKEN: AtomicU16 = AtomicU16::new(0);
        static INITIALIZED: core::sync::atomic::AtomicBool =
            core::sync::atomic::AtomicBool::new(false);

        // `GlobalContext` guards its mutable state behind a Mutex, so the shared
        // Arc handle is enough for both `createToken` and `flushEvents`.
        let global_context = self.global_context.clone();

        if !INITIALIZED.load(Ordering::Relaxed) {
            let token = create_token(
                &global_context,
                c"flushEvents".as_ptr(),
                c"TimeTrace".as_ptr(),
            );
            FLUSH_TOKEN.store(token, Ordering::Relaxed);
            INITIALIZED.store(true, Ordering::Relaxed);
        }

        let flush_token = FLUSH_TOKEN.load(Ordering::Relaxed);

        // events.push_back({EventType::Enter, flushToken, {getClockMicroseconds()}});
        self.events.push(Event {
            r#type: EventType::Enter,
            token: flush_token,
            data: EventData {
                microsec: get_clock_microseconds(),
            },
        });

        // TimeTrace::flushEvents(*globalContext, threadId, events, data);
        flush_events(&global_context, self.thread_id, &self.events, &self.data);

        // events.clear(); data.clear();
        self.events.clear();
        self.data.clear();

        // events.push_back({EventType::Leave, 0, {getClockMicroseconds()}});
        self.events.push(Event {
            r#type: EventType::Leave,
            token: 0,
            data: EventData {
                microsec: get_clock_microseconds(),
            },
        });
    }
}
