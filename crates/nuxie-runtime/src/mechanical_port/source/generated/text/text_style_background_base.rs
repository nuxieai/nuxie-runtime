use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    text::text_style_background::TextStyleBackground,
};

pub trait TextStyleBackgroundBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn corner_radius_changed(&mut self) {}
}

#[derive(Default)]
pub struct TextStyleBackgroundBase {
    pub base: ContainerComponent,
    corner_radius: f32,
}

impl TextStyleBackgroundBase {
    pub const TYPE_KEY: u16 = 1069;
    pub const CORNER_RADIUS_PROPERTY_KEY: u16 = 1071;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn corner_radius(&self) -> f32 {
        self.corner_radius
    }
    pub fn set_corner_radius(
        &mut self,
        value: f32,
        callbacks: &mut impl TextStyleBackgroundBaseCallbacks,
    ) {
        if !self.set_corner_radius_value(value) {
            return;
        }
        callbacks.corner_radius_changed();
        TextStyleBackgroundBaseCallbacks::notify_property_changed(
            callbacks,
            Self::CORNER_RADIUS_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_corner_radius_value(&mut self, value: f32) -> bool {
        if self.corner_radius == value {
            return false;
        }
        self.corner_radius = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TextStyleBackgroundBaseCallbacks,
    ) -> TextStyleBackground {
        let mut cloned = TextStyleBackground::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextStyleBackgroundBaseCallbacks) {
        self.corner_radius = object.corner_radius;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextStyleBackgroundBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::CORNER_RADIUS_PROPERTY_KEY => {
                self.corner_radius = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TextStyleBackgroundBase {
    type Target = ContainerComponent;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextStyleBackgroundBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
