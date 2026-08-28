use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    shapes::paint::feather::Feather,
};

pub trait FeatherBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn space_value_changed(&mut self) {}
    fn strength_changed(&mut self) {}
    fn offset_x_changed(&mut self) {}
    fn offset_y_changed(&mut self) {}
    fn inner_changed(&mut self) {}
}

pub struct FeatherBase {
    pub base: ContainerComponent,
    space_value: u32,
    strength: f32,
    offset_x: f32,
    offset_y: f32,
    inner: bool,
}

impl Default for FeatherBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            space_value: 0,
            strength: 12.0,
            offset_x: 0.0,
            offset_y: 0.0,
            inner: false,
        }
    }
}

impl FeatherBase {
    pub const TYPE_KEY: u16 = 533;
    pub const SPACE_VALUE_PROPERTY_KEY: u16 = 748;
    pub const STRENGTH_PROPERTY_KEY: u16 = 749;
    pub const OFFSET_X_PROPERTY_KEY: u16 = 750;
    pub const OFFSET_Y_PROPERTY_KEY: u16 = 751;
    pub const INNER_PROPERTY_KEY: u16 = 752;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn space_value(&self) -> u32 {
        self.space_value
    }
    pub fn set_space_value(&mut self, value: u32, callbacks: &mut impl FeatherBaseCallbacks) {
        if !self.set_space_value_value(value) {
            return;
        }
        callbacks.space_value_changed();
        callbacks.notify_property_changed(Self::SPACE_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_space_value_value(&mut self, value: u32) -> bool {
        if self.space_value == value {
            return false;
        }
        self.space_value = value;
        true
    }
    pub fn strength(&self) -> f32 {
        self.strength
    }
    pub fn set_strength(&mut self, value: f32, callbacks: &mut impl FeatherBaseCallbacks) {
        if !self.set_strength_value(value) {
            return;
        }
        callbacks.strength_changed();
        callbacks.notify_property_changed(Self::STRENGTH_PROPERTY_KEY);
    }

    pub(crate) fn set_strength_value(&mut self, value: f32) -> bool {
        if self.strength == value {
            return false;
        }
        self.strength = value;
        true
    }
    pub fn offset_x(&self) -> f32 {
        self.offset_x
    }
    pub fn set_offset_x(&mut self, value: f32, callbacks: &mut impl FeatherBaseCallbacks) {
        if !self.set_offset_x_value(value) {
            return;
        }
        callbacks.offset_x_changed();
        callbacks.notify_property_changed(Self::OFFSET_X_PROPERTY_KEY);
    }

    pub(crate) fn set_offset_x_value(&mut self, value: f32) -> bool {
        if self.offset_x == value {
            return false;
        }
        self.offset_x = value;
        true
    }
    pub fn offset_y(&self) -> f32 {
        self.offset_y
    }
    pub fn set_offset_y(&mut self, value: f32, callbacks: &mut impl FeatherBaseCallbacks) {
        if !self.set_offset_y_value(value) {
            return;
        }
        callbacks.offset_y_changed();
        callbacks.notify_property_changed(Self::OFFSET_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_offset_y_value(&mut self, value: f32) -> bool {
        if self.offset_y == value {
            return false;
        }
        self.offset_y = value;
        true
    }
    pub fn inner(&self) -> bool {
        self.inner
    }
    pub fn set_inner(&mut self, value: bool, callbacks: &mut impl FeatherBaseCallbacks) {
        if !self.set_inner_value(value) {
            return;
        }
        callbacks.inner_changed();
        callbacks.notify_property_changed(Self::INNER_PROPERTY_KEY);
    }

    pub(crate) fn set_inner_value(&mut self, value: bool) -> bool {
        if self.inner == value {
            return false;
        }
        self.inner = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl FeatherBaseCallbacks) -> Feather {
        let mut cloned = Feather::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl FeatherBaseCallbacks) {
        self.space_value = object.space_value;
        self.strength = object.strength;
        self.offset_x = object.offset_x;
        self.offset_y = object.offset_y;
        self.inner = object.inner;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl FeatherBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::SPACE_VALUE_PROPERTY_KEY => {
                self.space_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::STRENGTH_PROPERTY_KEY => {
                self.strength = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OFFSET_X_PROPERTY_KEY => {
                self.offset_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OFFSET_Y_PROPERTY_KEY => {
                self.offset_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::INNER_PROPERTY_KEY => {
                self.inner = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for FeatherBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for FeatherBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
