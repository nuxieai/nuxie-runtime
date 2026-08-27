use crate::mechanical_port::source::event::Event;

pub trait CallbackContext {
    fn report_event(&mut self, _event: &mut Event, _seconds_delay: f32) {}

    fn plays_audio(&self) -> bool {
        false
    }
}

pub struct CallbackData<'a> {
    context: Option<&'a mut dyn CallbackContext>,
    delay_seconds: f32,
}

impl<'a> CallbackData<'a> {
    pub fn new(context: Option<&'a mut dyn CallbackContext>, delay_seconds: f32) -> Self {
        Self {
            context,
            delay_seconds,
        }
    }

    pub fn context(&mut self) -> Option<&mut (dyn CallbackContext + 'a)> {
        self.context.as_deref_mut()
    }

    pub fn delay_seconds(&self) -> f32 {
        self.delay_seconds
    }
}
