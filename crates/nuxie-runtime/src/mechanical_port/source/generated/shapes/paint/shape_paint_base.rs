use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
};

pub trait ShapePaintBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn is_visible_changed(&mut self) {}
    fn blend_mode_value_changed(&mut self) {}
    fn additive_amount_changed(&mut self) {}
}

pub struct ShapePaintBase {
    pub base: ContainerComponent,
    is_visible: bool,
    blend_mode_value: u8,
    additive_amount: u8,
}

impl Default for ShapePaintBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            is_visible: true,
            blend_mode_value: 127,
            additive_amount: 255,
        }
    }
}

impl ShapePaintBase {
    pub const TYPE_KEY: u16 = 21;
    pub const IS_VISIBLE_PROPERTY_KEY: u16 = 41;
    pub const BLEND_MODE_VALUE_PROPERTY_KEY: u16 = 747;
    pub const ADDITIVE_AMOUNT_PROPERTY_KEY: u16 = 452;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn is_visible(&self) -> bool {
        self.is_visible
    }
    pub fn set_is_visible(&mut self, value: bool, callbacks: &mut impl ShapePaintBaseCallbacks) {
        if !self.set_is_visible_value(value) {
            return;
        }
        callbacks.is_visible_changed();
        ShapePaintBaseCallbacks::notify_property_changed(callbacks, Self::IS_VISIBLE_PROPERTY_KEY);
    }

    pub(crate) fn set_is_visible_value(&mut self, value: bool) -> bool {
        if self.is_visible == value {
            return false;
        }
        self.is_visible = value;
        true
    }
    pub fn blend_mode_value(&self) -> u8 {
        self.blend_mode_value
    }
    pub fn set_blend_mode_value(
        &mut self,
        value: u8,
        callbacks: &mut impl ShapePaintBaseCallbacks,
    ) {
        if !self.set_blend_mode_value_value(value) {
            return;
        }
        callbacks.blend_mode_value_changed();
        ShapePaintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BLEND_MODE_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_blend_mode_value_value(&mut self, value: u8) -> bool {
        if self.blend_mode_value == value {
            return false;
        }
        self.blend_mode_value = value;
        true
    }
    pub fn additive_amount(&self) -> u8 {
        self.additive_amount
    }
    pub fn set_additive_amount(&mut self, value: u8, callbacks: &mut impl ShapePaintBaseCallbacks) {
        if !self.set_additive_amount_value(value) {
            return;
        }
        callbacks.additive_amount_changed();
        ShapePaintBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ADDITIVE_AMOUNT_PROPERTY_KEY,
        );
    }
    pub(crate) fn set_additive_amount_value(&mut self, value: u8) -> bool {
        if self.additive_amount == value {
            return false;
        }
        self.additive_amount = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl ShapePaintBaseCallbacks) {
        self.is_visible = object.is_visible;
        self.blend_mode_value = object.blend_mode_value;
        self.additive_amount = object.additive_amount;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl ShapePaintBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ADDITIVE_AMOUNT_PROPERTY_KEY => {
                self.additive_amount = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::IS_VISIBLE_PROPERTY_KEY => {
                self.is_visible = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::BLEND_MODE_VALUE_PROPERTY_KEY => {
                self.blend_mode_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for ShapePaintBase {
    type Target = ContainerComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ShapePaintBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
