use crate::mechanical_port::source::event::Event;

pub struct EventReport {
    event: *mut Event,
    seconds_delay: f32,
}

impl EventReport {
    pub fn new(event: *mut Event, seconds_delay: f32) -> Self {
        Self {
            event,
            seconds_delay,
        }
    }

    pub fn event(&self) -> *mut Event {
        self.event
    }

    pub fn seconds_delay(&self) -> f32 {
        self.seconds_delay
    }
}
