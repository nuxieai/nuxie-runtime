use crate::mechanical_port::source::core::CoreHandle;

pub struct EventReport {
    event: CoreHandle,
    seconds_delay: f32,
}

impl EventReport {
    pub fn new(event: CoreHandle, seconds_delay: f32) -> Self {
        Self {
            event,
            seconds_delay,
        }
    }

    pub fn event(&self) -> CoreHandle {
        self.event.clone()
    }

    pub fn seconds_delay(&self) -> f32 {
        self.seconds_delay
    }
}
