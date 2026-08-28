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

impl std::ops::Deref for OpenUrlEvent {
    type Target = OpenUrlEventBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for OpenUrlEvent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
