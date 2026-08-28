use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_interpolator::DataConverterInterpolator,
};

pub trait DataConverterInterpolatorBaseCallbacks: crate::mechanical_port::source::generated::data_bind::converters::data_converter_base::DataConverterBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn interpolation_type_changed(&mut self) {}
    fn interpolator_id_changed(&mut self) {}
    fn duration_changed(&mut self) {}
}

pub struct DataConverterInterpolatorBase {
    pub base: DataConverter,
    interpolation_type: u32,
    interpolator_id: u32,
    duration: f32,
}

impl Default for DataConverterInterpolatorBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            interpolation_type: 1,
            interpolator_id: u32::MAX,
            duration: 1.0,
        }
    }
}

impl DataConverterInterpolatorBase {
    pub const TYPE_KEY: u16 = 534;
    pub const INTERPOLATION_TYPE_PROPERTY_KEY: u16 = 757;
    pub const INTERPOLATOR_ID_PROPERTY_KEY: u16 = 758;
    pub const DURATION_PROPERTY_KEY: u16 = 756;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 488)
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
        callbacks: &mut impl DataConverterInterpolatorBaseCallbacks,
    ) {
        if !self.set_interpolation_type_value(value) {
            return;
        }
        callbacks.interpolation_type_changed();
        callbacks.notify_property_changed(Self::INTERPOLATION_TYPE_PROPERTY_KEY);
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
        callbacks: &mut impl DataConverterInterpolatorBaseCallbacks,
    ) {
        if !self.set_interpolator_id_value(value) {
            return;
        }
        callbacks.interpolator_id_changed();
        callbacks.notify_property_changed(Self::INTERPOLATOR_ID_PROPERTY_KEY);
    }

    pub(crate) fn set_interpolator_id_value(&mut self, value: u32) -> bool {
        if self.interpolator_id == value {
            return false;
        }
        self.interpolator_id = value;
        true
    }
    pub fn duration(&self) -> f32 {
        self.duration
    }
    pub fn set_duration(
        &mut self,
        value: f32,
        callbacks: &mut impl DataConverterInterpolatorBaseCallbacks,
    ) {
        if !self.set_duration_value(value) {
            return;
        }
        callbacks.duration_changed();
        callbacks.notify_property_changed(Self::DURATION_PROPERTY_KEY);
    }

    pub(crate) fn set_duration_value(&mut self, value: f32) -> bool {
        if self.duration == value {
            return false;
        }
        self.duration = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterInterpolatorBaseCallbacks,
    ) -> DataConverterInterpolator {
        let mut cloned = DataConverterInterpolator::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterInterpolatorBaseCallbacks,
    ) {
        self.interpolation_type = object.interpolation_type;
        self.interpolator_id = object.interpolator_id;
        self.duration = object.duration;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterInterpolatorBaseCallbacks,
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
            Self::DURATION_PROPERTY_KEY => {
                self.duration = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for DataConverterInterpolatorBase {
    type Target = DataConverter;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataConverterInterpolatorBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
