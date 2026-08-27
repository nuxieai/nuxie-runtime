use crate::mechanical_port::source::{
    constraints::transform_space_constraint::TransformSpaceConstraint,
    core::binary_reader::BinaryReader,
};

pub trait TransformComponentConstraintBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn min_max_space_value_changed(&mut self) {}
    fn copy_factor_changed(&mut self) {}
    fn min_value_changed(&mut self) {}
    fn max_value_changed(&mut self) {}
    fn offset_changed(&mut self) {}
    fn does_copy_changed(&mut self) {}
    fn min_changed(&mut self) {}
    fn max_changed(&mut self) {}
}

pub struct TransformComponentConstraintBase {
    pub base: TransformSpaceConstraint,
    min_max_space_value: u32,
    copy_factor: f32,
    min_value: f32,
    max_value: f32,
    offset: bool,
    does_copy: bool,
    min: bool,
    max: bool,
}

impl Default for TransformComponentConstraintBase {
    fn default() -> Self {
        Self {
            base: TransformSpaceConstraint::default(),
            min_max_space_value: 0,
            copy_factor: 1.0,
            min_value: 0.0,
            max_value: 0.0,
            offset: false,
            does_copy: true,
            min: false,
            max: false,
        }
    }
}

impl TransformComponentConstraintBase {
    pub const TYPE_KEY: u16 = 85;
    pub const MIN_MAX_SPACE_VALUE_PROPERTY_KEY: u16 = 195;
    pub const COPY_FACTOR_PROPERTY_KEY: u16 = 182;
    pub const MIN_VALUE_PROPERTY_KEY: u16 = 183;
    pub const MAX_VALUE_PROPERTY_KEY: u16 = 184;
    pub const OFFSET_PROPERTY_KEY: u16 = 188;
    pub const DOES_COPY_PROPERTY_KEY: u16 = 189;
    pub const MIN_PROPERTY_KEY: u16 = 190;
    pub const MAX_PROPERTY_KEY: u16 = 191;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 90 | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn min_max_space_value(&self) -> u32 {
        self.min_max_space_value
    }
    pub fn set_min_max_space_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        if self.min_max_space_value == value {
            return;
        }
        self.min_max_space_value = value;
        callbacks.min_max_space_value_changed();
        callbacks.notify_property_changed(Self::MIN_MAX_SPACE_VALUE_PROPERTY_KEY);
    }
    pub fn copy_factor(&self) -> f32 {
        self.copy_factor
    }
    pub fn set_copy_factor(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        if self.copy_factor == value {
            return;
        }
        self.copy_factor = value;
        callbacks.copy_factor_changed();
        callbacks.notify_property_changed(Self::COPY_FACTOR_PROPERTY_KEY);
    }
    pub fn min_value(&self) -> f32 {
        self.min_value
    }
    pub fn set_min_value(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        if self.min_value == value {
            return;
        }
        self.min_value = value;
        callbacks.min_value_changed();
        callbacks.notify_property_changed(Self::MIN_VALUE_PROPERTY_KEY);
    }
    pub fn max_value(&self) -> f32 {
        self.max_value
    }
    pub fn set_max_value(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        if self.max_value == value {
            return;
        }
        self.max_value = value;
        callbacks.max_value_changed();
        callbacks.notify_property_changed(Self::MAX_VALUE_PROPERTY_KEY);
    }
    pub fn offset(&self) -> bool {
        self.offset
    }
    pub fn set_offset(
        &mut self,
        value: bool,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        if self.offset == value {
            return;
        }
        self.offset = value;
        callbacks.offset_changed();
        callbacks.notify_property_changed(Self::OFFSET_PROPERTY_KEY);
    }
    pub fn does_copy(&self) -> bool {
        self.does_copy
    }
    pub fn set_does_copy(
        &mut self,
        value: bool,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        if self.does_copy == value {
            return;
        }
        self.does_copy = value;
        callbacks.does_copy_changed();
        callbacks.notify_property_changed(Self::DOES_COPY_PROPERTY_KEY);
    }
    pub fn min(&self) -> bool {
        self.min
    }
    pub fn set_min(
        &mut self,
        value: bool,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        if self.min == value {
            return;
        }
        self.min = value;
        callbacks.min_changed();
        callbacks.notify_property_changed(Self::MIN_PROPERTY_KEY);
    }
    pub fn max(&self) -> bool {
        self.max
    }
    pub fn set_max(
        &mut self,
        value: bool,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        if self.max == value {
            return;
        }
        self.max = value;
        callbacks.max_changed();
        callbacks.notify_property_changed(Self::MAX_PROPERTY_KEY);
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) {
        self.min_max_space_value = object.min_max_space_value;
        self.copy_factor = object.copy_factor;
        self.min_value = object.min_value;
        self.max_value = object.max_value;
        self.offset = object.offset;
        self.does_copy = object.does_copy;
        self.min = object.min;
        self.max = object.max;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransformComponentConstraintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::MIN_MAX_SPACE_VALUE_PROPERTY_KEY => {
                self.min_max_space_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::COPY_FACTOR_PROPERTY_KEY => {
                self.copy_factor = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MIN_VALUE_PROPERTY_KEY => {
                self.min_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MAX_VALUE_PROPERTY_KEY => {
                self.max_value = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OFFSET_PROPERTY_KEY => {
                self.offset = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::DOES_COPY_PROPERTY_KEY => {
                self.does_copy = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::MIN_PROPERTY_KEY => {
                self.min = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::MAX_PROPERTY_KEY => {
                self.max = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
