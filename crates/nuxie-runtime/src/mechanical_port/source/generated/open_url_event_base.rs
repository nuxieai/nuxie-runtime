use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, event::Event, open_url_event::OpenUrlEvent,
};

pub trait OpenUrlEventBaseCallbacks {
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
        if self.url == value {
            return;
        }
        self.url = value;
        callbacks.url_changed();
        callbacks.notify_property_changed(Self::URL_PROPERTY_KEY);
    }
    pub fn target_value(&self) -> u32 {
        self.target_value
    }
    pub fn set_target_value(&mut self, value: u32, callbacks: &mut impl OpenUrlEventBaseCallbacks) {
        if self.target_value == value {
            return;
        }
        self.target_value = value;
        callbacks.target_value_changed();
        callbacks.notify_property_changed(Self::TARGET_VALUE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl OpenUrlEventBaseCallbacks) -> OpenUrlEvent {
        let mut cloned = OpenUrlEvent::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl OpenUrlEventBaseCallbacks) {
        self.url.clone_from(&object.url);
        self.target_value = object.target_value;
        self.base.copy(&object.base, callbacks);
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
