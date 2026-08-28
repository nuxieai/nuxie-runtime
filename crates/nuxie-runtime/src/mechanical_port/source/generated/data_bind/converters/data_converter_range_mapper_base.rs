use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, data_bind::converters::data_converter::DataConverter,
    data_bind::converters::data_converter_range_mapper::DataConverterRangeMapper,
};

pub trait DataConverterRangeMapperBaseCallbacks: crate::mechanical_port::source::generated::data_bind::converters::data_converter_base::DataConverterBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn interpolation_type_changed(&mut self) {}
    fn interpolator_id_changed(&mut self) {}
    fn flags_changed(&mut self) {}
    fn min_input_changed(&mut self) {}
    fn max_input_changed(&mut self) {}
    fn min_output_changed(&mut self) {}
    fn max_output_changed(&mut self) {}
}

pub struct DataConverterRangeMapperBase {
    pub base: DataConverter,
    interpolation_type: u32,
    interpolator_id: u32,
    flags: u32,
    min_input: f32,
    max_input: f32,
    min_output: f32,
    max_output: f32,
}

impl Default for DataConverterRangeMapperBase {
    fn default() -> Self {
        Self {
            base: DataConverter::default(),
            interpolation_type: 1,
            interpolator_id: u32::MAX,
            flags: 0,
            min_input: 1.0,
            max_input: 1.0,
            min_output: 1.0,
            max_output: 1.0,
        }
    }
}

impl DataConverterRangeMapperBase {
    pub const TYPE_KEY: u16 = 519;
    pub const INTERPOLATION_TYPE_PROPERTY_KEY: u16 = 713;
    pub const INTERPOLATOR_ID_PROPERTY_KEY: u16 = 714;
    pub const FLAGS_PROPERTY_KEY: u16 = 715;
    pub const MIN_INPUT_PROPERTY_KEY: u16 = 716;
    pub const MAX_INPUT_PROPERTY_KEY: u16 = 717;
    pub const MIN_OUTPUT_PROPERTY_KEY: u16 = 718;
    pub const MAX_OUTPUT_PROPERTY_KEY: u16 = 719;

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
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
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
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
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
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn set_flags(
        &mut self,
        value: u32,
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
    ) {
        if !self.set_flags_value(value) {
            return;
        }
        callbacks.flags_changed();
        callbacks.notify_property_changed(Self::FLAGS_PROPERTY_KEY);
    }

    pub(crate) fn set_flags_value(&mut self, value: u32) -> bool {
        if self.flags == value {
            return false;
        }
        self.flags = value;
        true
    }
    pub fn min_input(&self) -> f32 {
        self.min_input
    }
    pub fn set_min_input(
        &mut self,
        value: f32,
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
    ) {
        if !self.set_min_input_value(value) {
            return;
        }
        callbacks.min_input_changed();
        callbacks.notify_property_changed(Self::MIN_INPUT_PROPERTY_KEY);
    }

    pub(crate) fn set_min_input_value(&mut self, value: f32) -> bool {
        if self.min_input == value {
            return false;
        }
        self.min_input = value;
        true
    }
    pub fn max_input(&self) -> f32 {
        self.max_input
    }
    pub fn set_max_input(
        &mut self,
        value: f32,
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
    ) {
        if !self.set_max_input_value(value) {
            return;
        }
        callbacks.max_input_changed();
        callbacks.notify_property_changed(Self::MAX_INPUT_PROPERTY_KEY);
    }

    pub(crate) fn set_max_input_value(&mut self, value: f32) -> bool {
        if self.max_input == value {
            return false;
        }
        self.max_input = value;
        true
    }
    pub fn min_output(&self) -> f32 {
        self.min_output
    }
    pub fn set_min_output(
        &mut self,
        value: f32,
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
    ) {
        if !self.set_min_output_value(value) {
            return;
        }
        callbacks.min_output_changed();
        callbacks.notify_property_changed(Self::MIN_OUTPUT_PROPERTY_KEY);
    }

    pub(crate) fn set_min_output_value(&mut self, value: f32) -> bool {
        if self.min_output == value {
            return false;
        }
        self.min_output = value;
        true
    }
    pub fn max_output(&self) -> f32 {
        self.max_output
    }
    pub fn set_max_output(
        &mut self,
        value: f32,
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
    ) {
        if !self.set_max_output_value(value) {
            return;
        }
        callbacks.max_output_changed();
        callbacks.notify_property_changed(Self::MAX_OUTPUT_PROPERTY_KEY);
    }

    pub(crate) fn set_max_output_value(&mut self, value: f32) -> bool {
        if self.max_output == value {
            return false;
        }
        self.max_output = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
    ) -> DataConverterRangeMapper {
        let mut cloned = DataConverterRangeMapper::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
    ) {
        self.interpolation_type = object.interpolation_type;
        self.interpolator_id = object.interpolator_id;
        self.flags = object.flags;
        self.min_input = object.min_input;
        self.max_input = object.max_input;
        self.min_output = object.min_output;
        self.max_output = object.max_output;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DataConverterRangeMapperBaseCallbacks,
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
            Self::FLAGS_PROPERTY_KEY => {
                self.flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::MIN_INPUT_PROPERTY_KEY => {
                self.min_input = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MAX_INPUT_PROPERTY_KEY => {
                self.max_input = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MIN_OUTPUT_PROPERTY_KEY => {
                self.min_output = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MAX_OUTPUT_PROPERTY_KEY => {
                self.max_output = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for DataConverterRangeMapperBase {
    type Target = DataConverter;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DataConverterRangeMapperBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
