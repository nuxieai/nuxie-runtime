use crate::mechanical_port::source::core::field_types::core_callback_type::CallbackData;
use crate::mechanical_port::source::generated::event_base::{EventBase, EventBaseCallbacks};

#[derive(Default)]
pub struct Event {
    pub base: EventBase,
}

impl EventBaseCallbacks for Event {
    fn trigger(&mut self, value: &mut CallbackData<'_>) {
        Event::trigger(self, value);
    }
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

impl std::ops::Deref for Event {
    type Target = EventBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Event {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
