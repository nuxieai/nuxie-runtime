use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader, shapes::paint::dash::Dash,
};

pub trait DashBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn length_changed(&mut self) {}
    fn length_is_percentage_changed(&mut self) {}
}

pub struct DashBase {
    pub base: Component,
    length: f32,
    length_is_percentage: bool,
}

impl Default for DashBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            length: 0.0,
            length_is_percentage: false,
        }
    }
}

impl DashBase {
    pub const TYPE_KEY: u16 = 507;
    pub const LENGTH_PROPERTY_KEY: u16 = 692;
    pub const LENGTH_IS_PERCENTAGE_PROPERTY_KEY: u16 = 693;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn length(&self) -> f32 {
        self.length
    }
    pub fn set_length(&mut self, value: f32, callbacks: &mut impl DashBaseCallbacks) {
        if self.length == value {
            return;
        }
        self.length = value;
        callbacks.length_changed();
        callbacks.notify_property_changed(Self::LENGTH_PROPERTY_KEY);
    }
    pub fn length_is_percentage(&self) -> bool {
        self.length_is_percentage
    }
    pub fn set_length_is_percentage(
        &mut self,
        value: bool,
        callbacks: &mut impl DashBaseCallbacks,
    ) {
        if self.length_is_percentage == value {
            return;
        }
        self.length_is_percentage = value;
        callbacks.length_is_percentage_changed();
        callbacks.notify_property_changed(Self::LENGTH_IS_PERCENTAGE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl DashBaseCallbacks) -> Dash {
        let mut cloned = Dash::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DashBaseCallbacks) {
        self.length = object.length;
        self.length_is_percentage = object.length_is_percentage;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DashBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::LENGTH_PROPERTY_KEY => {
                self.length = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::LENGTH_IS_PERCENTAGE_PROPERTY_KEY => {
                self.length_is_percentage = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
