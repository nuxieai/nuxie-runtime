use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
};

pub trait WorldTransformComponentBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn opacity_changed(&mut self) {}
}

pub struct WorldTransformComponentBase {
    pub base: ContainerComponent,
    opacity: f32,
}

impl Default for WorldTransformComponentBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            opacity: 1.0,
        }
    }
}

impl WorldTransformComponentBase {
    pub const TYPE_KEY: u16 = 91;
    pub const OPACITY_PROPERTY_KEY: u16 = 18;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    pub fn set_opacity(
        &mut self,
        value: f32,
        callbacks: &mut impl WorldTransformComponentBaseCallbacks,
    ) {
        if !self.set_opacity_value(value) {
            return;
        }
        callbacks.opacity_changed();
        callbacks.notify_property_changed(Self::OPACITY_PROPERTY_KEY);
    }

    pub(crate) fn set_opacity_value(&mut self, value: f32) -> bool {
        if self.opacity == value {
            return false;
        }
        self.opacity = value;
        true
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl WorldTransformComponentBaseCallbacks,
    ) {
        self.opacity = object.opacity;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl WorldTransformComponentBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OPACITY_PROPERTY_KEY => {
                self.opacity = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for WorldTransformComponentBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for WorldTransformComponentBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
