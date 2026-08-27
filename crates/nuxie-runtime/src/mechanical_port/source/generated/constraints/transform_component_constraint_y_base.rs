use crate::mechanical_port::source::{
    constraints::transform_component_constraint::TransformComponentConstraint,
    core::binary_reader::BinaryReader,
};

pub trait TransformComponentConstraintYBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn copy_factor_y_changed(&mut self) {}
    fn min_value_y_changed(&mut self) {}
    fn max_value_y_changed(&mut self) {}
    fn does_copy_y_changed(&mut self) {}
    fn min_y_changed(&mut self) {}
    fn max_y_changed(&mut self) {}
}

pub struct TransformComponentConstraintYBase {
    pub base: TransformComponentConstraint,
    copy_factor_y: f32,
    min_value_y: f32,
    max_value_y: f32,
    does_copy_y: bool,
    min_y: bool,
    max_y: bool,
}

impl Default for TransformComponentConstraintYBase {
    fn default() -> Self {
        Self {
            base: TransformComponentConstraint::default(),
            copy_factor_y: 1.0,
            min_value_y: 0.0,
            max_value_y: 0.0,
            does_copy_y: true,
            min_y: false,
            max_y: false,
        }
    }
}

impl TransformComponentConstraintYBase {
    pub const TYPE_KEY: u16 = 86;
    pub const COPY_FACTOR_Y_PROPERTY_KEY: u16 = 185;
    pub const MIN_VALUE_Y_PROPERTY_KEY: u16 = 186;
    pub const MAX_VALUE_Y_PROPERTY_KEY: u16 = 187;
    pub const DOES_COPY_Y_PROPERTY_KEY: u16 = 192;
    pub const MIN_Y_PROPERTY_KEY: u16 = 193;
    pub const MAX_Y_PROPERTY_KEY: u16 = 194;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 85 | 90 | 80 | 79 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn copy_factor_y(&self) -> f32 {
        self.copy_factor_y
    }
    pub fn set_copy_factor_y(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) {
        if self.copy_factor_y == value {
            return;
        }
        self.copy_factor_y = value;
        callbacks.copy_factor_y_changed();
        callbacks.notify_property_changed(Self::COPY_FACTOR_Y_PROPERTY_KEY);
    }
    pub fn min_value_y(&self) -> f32 {
        self.min_value_y
    }
    pub fn set_min_value_y(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) {
        if self.min_value_y == value {
            return;
        }
        self.min_value_y = value;
        callbacks.min_value_y_changed();
        callbacks.notify_property_changed(Self::MIN_VALUE_Y_PROPERTY_KEY);
    }
    pub fn max_value_y(&self) -> f32 {
        self.max_value_y
    }
    pub fn set_max_value_y(
        &mut self,
        value: f32,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) {
        if self.max_value_y == value {
            return;
        }
        self.max_value_y = value;
        callbacks.max_value_y_changed();
        callbacks.notify_property_changed(Self::MAX_VALUE_Y_PROPERTY_KEY);
    }
    pub fn does_copy_y(&self) -> bool {
        self.does_copy_y
    }
    pub fn set_does_copy_y(
        &mut self,
        value: bool,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) {
        if self.does_copy_y == value {
            return;
        }
        self.does_copy_y = value;
        callbacks.does_copy_y_changed();
        callbacks.notify_property_changed(Self::DOES_COPY_Y_PROPERTY_KEY);
    }
    pub fn min_y(&self) -> bool {
        self.min_y
    }
    pub fn set_min_y(
        &mut self,
        value: bool,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) {
        if self.min_y == value {
            return;
        }
        self.min_y = value;
        callbacks.min_y_changed();
        callbacks.notify_property_changed(Self::MIN_Y_PROPERTY_KEY);
    }
    pub fn max_y(&self) -> bool {
        self.max_y
    }
    pub fn set_max_y(
        &mut self,
        value: bool,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) {
        if self.max_y == value {
            return;
        }
        self.max_y = value;
        callbacks.max_y_changed();
        callbacks.notify_property_changed(Self::MAX_Y_PROPERTY_KEY);
    }
    pub fn copy(
        &mut self,
        object: &Self,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) {
        self.copy_factor_y = object.copy_factor_y;
        self.min_value_y = object.min_value_y;
        self.max_value_y = object.max_value_y;
        self.does_copy_y = object.does_copy_y;
        self.min_y = object.min_y;
        self.max_y = object.max_y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TransformComponentConstraintYBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::COPY_FACTOR_Y_PROPERTY_KEY => {
                self.copy_factor_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MIN_VALUE_Y_PROPERTY_KEY => {
                self.min_value_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MAX_VALUE_Y_PROPERTY_KEY => {
                self.max_value_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::DOES_COPY_Y_PROPERTY_KEY => {
                self.does_copy_y = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::MIN_Y_PROPERTY_KEY => {
                self.min_y = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::MAX_Y_PROPERTY_KEY => {
                self.max_y = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
