use crate::mechanical_port::source::{
    audio_event::AudioEvent, core::binary_reader::BinaryReader, event::Event,
};

pub trait AudioEventBaseCallbacks:
    crate::mechanical_port::source::generated::event_base::EventBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn asset_id_changed(&mut self) {}
}

pub struct AudioEventBase {
    pub base: Event,
    asset_id: u32,
}

impl Default for AudioEventBase {
    fn default() -> Self {
        Self {
            base: Event::default(),
            asset_id: u32::MAX,
        }
    }
}

impl AudioEventBase {
    pub const TYPE_KEY: u16 = 407;
    pub const ASSET_ID_PROPERTY_KEY: u16 = 408;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 128 | 548 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn asset_id(&self) -> u32 {
        self.asset_id
    }
    pub fn set_asset_id(&mut self, value: u32, callbacks: &mut impl AudioEventBaseCallbacks) {
        if !self.set_asset_id_value(value) {
            return;
        }
        callbacks.asset_id_changed();
        AudioEventBaseCallbacks::notify_property_changed(callbacks, Self::ASSET_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_asset_id_value(&mut self, value: u32) -> bool {
        if self.asset_id == value {
            return false;
        }
        self.asset_id = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl AudioEventBaseCallbacks) -> AudioEvent {
        let mut cloned = AudioEvent::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, _callbacks: &mut impl AudioEventBaseCallbacks) {
        self.asset_id = object.asset_id;
        self.base.base.copy(&object.base.base);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl AudioEventBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ASSET_ID_PROPERTY_KEY => {
                self.asset_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for AudioEventBase {
    type Target = Event;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for AudioEventBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
