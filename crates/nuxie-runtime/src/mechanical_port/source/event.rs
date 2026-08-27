use crate::mechanical_port::source::core::field_types::core_callback_type::CallbackData;
use crate::mechanical_port::source::generated::event_base::EventBase;

pub struct Event {
    pub base: EventBase,
}

impl Event {
    pub fn trigger(&mut self, value: &mut CallbackData<'_>) {
        let delay_seconds = value.delay_seconds();
        value
            .context()
            .expect("Event::trigger requires CallbackData context")
            .report_event(self, delay_seconds);
    }
}
