use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, drawable::Drawable, shapes::shape::Shape,
};

pub trait ShapeBaseCallbacks:
    crate::mechanical_port::source::generated::drawable_base::DrawableBaseCallbacks
{
    fn length_changed(&mut self) {}
    fn set_length(&mut self, value: f32);
    fn length(&mut self) -> f32;
}

pub struct ShapeBase {
    pub base: Drawable,
}

impl Default for ShapeBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
        }
    }
}

impl ShapeBase {
    pub const TYPE_KEY: u16 = 3;
    pub const LENGTH_PROPERTY_KEY: u16 = 781;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self, callbacks: &mut impl ShapeBaseCallbacks) -> Shape {
        let mut cloned = Shape::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ShapeBaseCallbacks) {
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ShapeBaseCallbacks,
    ) -> bool {
        self.base.deserialize(property_key, reader, callbacks)
    }
}

impl std::ops::Deref for ShapeBase {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ShapeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
