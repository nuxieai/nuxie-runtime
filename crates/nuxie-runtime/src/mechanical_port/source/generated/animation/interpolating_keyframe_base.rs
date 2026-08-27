use crate::mechanical_port::source::{core::binary_reader::BinaryReader, key_frame::KeyFrame};

pub trait InterpolatingKeyFrameBaseCallbacks {
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
        if self.interpolation_type == value {
            return;
        }
        self.interpolation_type = value;
        callbacks.interpolation_type_changed();
        callbacks.notify_property_changed(Self::INTERPOLATION_TYPE_PROPERTY_KEY);
    }
    pub fn interpolator_id(&self) -> u32 {
        self.interpolator_id
    }
    pub fn set_interpolator_id(
        &mut self,
        value: u32,
        callbacks: &mut impl InterpolatingKeyFrameBaseCallbacks,
    ) {
        if self.interpolator_id == value {
            return;
        }
        self.interpolator_id = value;
        callbacks.interpolator_id_changed();
        callbacks.notify_property_changed(Self::INTERPOLATOR_ID_PROPERTY_KEY);
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
