use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, event::Event, open_url_event::OpenUrlEvent,
};

pub trait OpenUrlEventBaseCallbacks:
    crate::mechanical_port::source::generated::event_base::EventBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn url_changed(&mut self) {}
    fn target_value_changed(&mut self) {}
}

pub struct OpenUrlEventBase {
    pub base: Event,
    url: String,
    target_value: u32,
}

impl Default for OpenUrlEventBase {
    fn default() -> Self {
        Self {
            base: Event::default(),
            url: "".to_owned(),
            target_value: 0,
        }
    }
}

impl OpenUrlEventBase {
    pub const TYPE_KEY: u16 = 131;
    pub const URL_PROPERTY_KEY: u16 = 248;
    pub const TARGET_VALUE_PROPERTY_KEY: u16 = 249;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 128 | 548 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn url(&self) -> &str {
        &self.url
    }
    pub fn set_url(&mut self, value: String, callbacks: &mut impl OpenUrlEventBaseCallbacks) {
        if !self.set_url_value(value) {
            return;
        }
        callbacks.url_changed();
        OpenUrlEventBaseCallbacks::notify_property_changed(callbacks, Self::URL_PROPERTY_KEY);
    }

    pub(crate) fn set_url_value(&mut self, value: String) -> bool {
        if self.url == value {
            return false;
        }
        self.url = value;
        true
    }
    pub fn target_value(&self) -> u32 {
        self.target_value
    }
    pub fn set_target_value(&mut self, value: u32, callbacks: &mut impl OpenUrlEventBaseCallbacks) {
        if !self.set_target_value_value(value) {
            return;
        }
        callbacks.target_value_changed();
        OpenUrlEventBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TARGET_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_target_value_value(&mut self, value: u32) -> bool {
        if self.target_value == value {
            return false;
        }
        self.target_value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl OpenUrlEventBaseCallbacks) -> OpenUrlEvent {
        let mut cloned = OpenUrlEvent::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl OpenUrlEventBaseCallbacks) {
        self.url.clone_from(&object.url);
        self.target_value = object.target_value;
        self.base.base.copy(&object.base.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl OpenUrlEventBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::URL_PROPERTY_KEY => {
                self.url = crate::mechanical_port::source::core::field_types::core_string_type::CoreStringType::deserialize(reader);
                true
            }
            Self::TARGET_VALUE_PROPERTY_KEY => {
                self.target_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for OpenUrlEventBase {
    type Target = Event;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for OpenUrlEventBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
