use crate::mechanical_port::source::{
    bones::{bone::Bone, skeletal_component::SkeletalComponent},
    core::{binary_reader::BinaryReader, field_types::core_double_type::CoreDoubleType},
};

pub trait BoneBaseCallbacks:
    crate::mechanical_port::source::generated::transform_component_base::TransformComponentBaseCallbacks
{
    fn length_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct BoneBase {
    pub base: SkeletalComponent,
    length: f32,
}

impl Default for BoneBase {
    fn default() -> Self {
        Self {
            base: SkeletalComponent::default(),
            length: 0.0,
        }
    }
}

impl BoneBase {
    pub const TYPE_KEY: u16 = 40;
    pub const LENGTH_PROPERTY_KEY: u16 = 89;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 39 | 38 | 91 | 11 | 10)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }

    pub fn length(&self) -> f32 {
        self.length
    }

    pub fn set_length<C: BoneBaseCallbacks>(&mut self, value: f32, callbacks: &mut C) {
        if !self.set_length_value(value) {
            return;
        }
        callbacks.length_changed();
        BoneBaseCallbacks::notify_property_changed(callbacks, Self::LENGTH_PROPERTY_KEY);
    }

    pub(crate) fn set_length_value(&mut self, value: f32) -> bool {
        if self.length == value {
            return false;
        }
        self.length = value;
        true
    }

    pub fn clone_into<C: BoneBaseCallbacks>(&self, callbacks: &mut C) -> Bone {
        let mut cloned = Bone::default();
        cloned.base.copy(self, callbacks);
        cloned
    }

    pub fn copy<C: BoneBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.length = object.length;
        self.base.base.base.copy(&object.base.base.base, callbacks);
    }

    pub fn deserialize<C: BoneBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::LENGTH_PROPERTY_KEY => {
                self.length = CoreDoubleType::deserialize(reader);
                true
            }
            _ => self
                .base
                .base
                .base
                .deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for BoneBase {
    type Target = SkeletalComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for BoneBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
