use crate::mechanical_port::source::{
    core::field_types::core_callback_type::CallbackData,
    custom_property_group::CustomPropertyGroup, event::Event,
};

pub trait EventBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn trigger(&mut self, value: &mut CallbackData<'_>);
}

pub struct EventBase {
    pub base: CustomPropertyGroup,
}
impl Default for EventBase {
    fn default() -> Self {
        Self {
            base: CustomPropertyGroup::default(),
        }
    }
}
impl EventBase {
    pub const TYPE_KEY: u16 = 128;
    pub const TRIGGER_PROPERTY_KEY: u16 = 395;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 548 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy(&mut self, object: &Self) {
        self.base.base.copy(&object.base.base);
    }
    pub fn clone_into(&self) -> Event {
        let mut cloned = Event::default();
        cloned.base.copy(self);
        cloned
    }
}

impl std::ops::Deref for EventBase {
    type Target = CustomPropertyGroup;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for EventBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
