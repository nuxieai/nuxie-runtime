use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    shapes::paint::dash_path::DashPath,
};

pub trait DashPathBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn offset_changed(&mut self) {}
    fn offset_is_percentage_changed(&mut self) {}
}

pub struct DashPathBase {
    pub base: ContainerComponent,
    offset: f32,
    offset_is_percentage: bool,
}

impl Default for DashPathBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            offset: 0.0,
            offset_is_percentage: false,
        }
    }
}

impl DashPathBase {
    pub const TYPE_KEY: u16 = 506;
    pub const OFFSET_PROPERTY_KEY: u16 = 690;
    pub const OFFSET_IS_PERCENTAGE_PROPERTY_KEY: u16 = 691;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn offset(&self) -> f32 {
        self.offset
    }
    pub fn set_offset(&mut self, value: f32, callbacks: &mut impl DashPathBaseCallbacks) {
        if self.offset == value {
            return;
        }
        self.offset = value;
        callbacks.offset_changed();
        callbacks.notify_property_changed(Self::OFFSET_PROPERTY_KEY);
    }
    pub fn offset_is_percentage(&self) -> bool {
        self.offset_is_percentage
    }
    pub fn set_offset_is_percentage(
        &mut self,
        value: bool,
        callbacks: &mut impl DashPathBaseCallbacks,
    ) {
        if self.offset_is_percentage == value {
            return;
        }
        self.offset_is_percentage = value;
        callbacks.offset_is_percentage_changed();
        callbacks.notify_property_changed(Self::OFFSET_IS_PERCENTAGE_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl DashPathBaseCallbacks) -> DashPath {
        let mut cloned = DashPath::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DashPathBaseCallbacks) {
        self.offset = object.offset;
        self.offset_is_percentage = object.offset_is_percentage;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DashPathBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::OFFSET_PROPERTY_KEY => {
                self.offset = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OFFSET_IS_PERCENTAGE_PROPERTY_KEY => {
                self.offset_is_percentage = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
