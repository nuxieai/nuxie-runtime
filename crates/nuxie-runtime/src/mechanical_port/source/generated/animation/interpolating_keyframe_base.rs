use crate::mechanical_port::source::{
    animation::keyframe::KeyFrame, core::binary_reader::BinaryReader,
};

pub trait InterpolatingKeyFrameBaseCallbacks:
    crate::mechanical_port::source::generated::animation::keyframe_base::KeyFrameBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn interpolation_type_changed(&mut self) {}
    fn interpolator_id_changed(&mut self) {}
}

pub struct InterpolatingKeyFrameBase {
    pub base: KeyFrame,
    interpolation_type: u32,
    interpolator_id: u32,
}

impl Default for InterpolatingKeyFrameBase {
    fn default() -> Self {
        Self {
            base: KeyFrame::default(),
            interpolation_type: 0,
            interpolator_id: u32::MAX,
        }
    }
}

impl InterpolatingKeyFrameBase {
    pub const TYPE_KEY: u16 = 170;
    pub const INTERPOLATION_TYPE_PROPERTY_KEY: u16 = 68;
    pub const INTERPOLATOR_ID_PROPERTY_KEY: u16 = 69;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 29)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn interpolation_type(&self) -> u32 {
        self.interpolation_type
    }
    pub fn set_interpolation_type(
        &mut self,
        value: u32,
        callbacks: &mut impl InterpolatingKeyFrameBaseCallbacks,
    ) {
        if !self.set_interpolation_type_value(value) {
            return;
        }
        callbacks.interpolation_type_changed();
        InterpolatingKeyFrameBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INTERPOLATION_TYPE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_interpolation_type_value(&mut self, value: u32) -> bool {
        if self.interpolation_type == value {
            return false;
        }
        self.interpolation_type = value;
        true
    }
    pub fn interpolator_id(&self) -> u32 {
        self.interpolator_id
    }
    pub fn set_interpolator_id(
        &mut self,
        value: u32,
        callbacks: &mut impl InterpolatingKeyFrameBaseCallbacks,
    ) {
        if !self.set_interpolator_id_value(value) {
            return;
        }
        callbacks.interpolator_id_changed();
        InterpolatingKeyFrameBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INTERPOLATOR_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_interpolator_id_value(&mut self, value: u32) -> bool {
        if self.interpolator_id == value {
            return false;
        }
        self.interpolator_id = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl InterpolatingKeyFrameBaseCallbacks) {
        self.interpolation_type = object.interpolation_type;
        self.interpolator_id = object.interpolator_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl InterpolatingKeyFrameBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INTERPOLATION_TYPE_PROPERTY_KEY => {
                self.interpolation_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::INTERPOLATOR_ID_PROPERTY_KEY => {
                self.interpolator_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for InterpolatingKeyFrameBase {
    type Target = KeyFrame;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for InterpolatingKeyFrameBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
