use crate::mechanical_port::source::generated::open_url_event_base::{
    OpenUrlEventBase, OpenUrlEventBaseCallbacks,
};

#[derive(Default)]
pub struct OpenUrlEvent {
    pub base: OpenUrlEventBase,
}

impl OpenUrlEventBaseCallbacks for OpenUrlEvent {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}
