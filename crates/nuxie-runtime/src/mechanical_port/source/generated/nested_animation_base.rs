use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
};

pub trait NestedAnimationBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn animation_id_changed(&mut self) {}
}

pub struct NestedAnimationBase {
    pub base: ContainerComponent,
    animation_id: u32,
}

impl Default for NestedAnimationBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            animation_id: u32::MAX,
        }
    }
}

impl NestedAnimationBase {
    pub const TYPE_KEY: u16 = 93;
    pub const ANIMATION_ID_PROPERTY_KEY: u16 = 198;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn animation_id(&self) -> u32 {
        self.animation_id
    }
    pub fn set_animation_id(
        &mut self,
        value: u32,
        callbacks: &mut impl NestedAnimationBaseCallbacks,
    ) {
        if !self.set_animation_id_value(value) {
            return;
        }
        callbacks.animation_id_changed();
        NestedAnimationBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ANIMATION_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_animation_id_value(&mut self, value: u32) -> bool {
        if self.animation_id == value {
            return false;
        }
        self.animation_id = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NestedAnimationBaseCallbacks) {
        self.animation_id = object.animation_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NestedAnimationBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ANIMATION_ID_PROPERTY_KEY => {
                self.animation_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for NestedAnimationBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NestedAnimationBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
