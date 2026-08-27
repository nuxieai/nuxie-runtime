use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    text::text_modifier_group::TextModifierGroup,
};

pub trait TextModifierGroupBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn modifier_flags_changed(&mut self) {}
    fn origin_x_changed(&mut self) {}
    fn origin_y_changed(&mut self) {}
    fn opacity_changed(&mut self) {}
    fn x_changed(&mut self) {}
    fn y_changed(&mut self) {}
    fn rotation_changed(&mut self) {}
    fn scale_x_changed(&mut self) {}
    fn scale_y_changed(&mut self) {}
}

pub struct TextModifierGroupBase {
    pub base: ContainerComponent,
    modifier_flags: u32,
    origin_x: f32,
    origin_y: f32,
    opacity: f32,
    x: f32,
    y: f32,
    rotation: f32,
    scale_x: f32,
    scale_y: f32,
}

impl Default for TextModifierGroupBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            modifier_flags: 0,
            origin_x: 0.0,
            origin_y: 0.0,
            opacity: 1.0,
            x: 0.0,
            y: 0.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }
    }
}

impl TextModifierGroupBase {
    pub const TYPE_KEY: u16 = 159;
    pub const MODIFIER_FLAGS_PROPERTY_KEY: u16 = 335;
    pub const ORIGIN_X_PROPERTY_KEY: u16 = 328;
    pub const ORIGIN_Y_PROPERTY_KEY: u16 = 329;
    pub const OPACITY_PROPERTY_KEY: u16 = 324;
    pub const X_PROPERTY_KEY: u16 = 322;
    pub const Y_PROPERTY_KEY: u16 = 323;
    pub const ROTATION_PROPERTY_KEY: u16 = 332;
    pub const SCALE_X_PROPERTY_KEY: u16 = 330;
    pub const SCALE_Y_PROPERTY_KEY: u16 = 331;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn modifier_flags(&self) -> u32 {
        self.modifier_flags
    }
    pub fn set_modifier_flags(
        &mut self,
        value: u32,
        callbacks: &mut impl TextModifierGroupBaseCallbacks,
    ) {
        if self.modifier_flags == value {
            return;
        }
        self.modifier_flags = value;
        callbacks.modifier_flags_changed();
        callbacks.notify_property_changed(Self::MODIFIER_FLAGS_PROPERTY_KEY);
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn set_origin_x(
        &mut self,
        value: f32,
        callbacks: &mut impl TextModifierGroupBaseCallbacks,
    ) {
        if self.origin_x == value {
            return;
        }
        self.origin_x = value;
        callbacks.origin_x_changed();
        callbacks.notify_property_changed(Self::ORIGIN_X_PROPERTY_KEY);
    }
    pub fn origin_y(&self) -> f32 {
        self.origin_y
    }
    pub fn set_origin_y(
        &mut self,
        value: f32,
        callbacks: &mut impl TextModifierGroupBaseCallbacks,
    ) {
        if self.origin_y == value {
            return;
        }
        self.origin_y = value;
        callbacks.origin_y_changed();
        callbacks.notify_property_changed(Self::ORIGIN_Y_PROPERTY_KEY);
    }
    pub fn opacity(&self) -> f32 {
        self.opacity
    }
    pub fn set_opacity(&mut self, value: f32, callbacks: &mut impl TextModifierGroupBaseCallbacks) {
        if self.opacity == value {
            return;
        }
        self.opacity = value;
        callbacks.opacity_changed();
        callbacks.notify_property_changed(Self::OPACITY_PROPERTY_KEY);
    }
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn set_x(&mut self, value: f32, callbacks: &mut impl TextModifierGroupBaseCallbacks) {
        if self.x == value {
            return;
        }
        self.x = value;
        callbacks.x_changed();
        callbacks.notify_property_changed(Self::X_PROPERTY_KEY);
    }
    pub fn y(&self) -> f32 {
        self.y
    }
    pub fn set_y(&mut self, value: f32, callbacks: &mut impl TextModifierGroupBaseCallbacks) {
        if self.y == value {
            return;
        }
        self.y = value;
        callbacks.y_changed();
        callbacks.notify_property_changed(Self::Y_PROPERTY_KEY);
    }
    pub fn rotation(&self) -> f32 {
        self.rotation
    }
    pub fn set_rotation(
        &mut self,
        value: f32,
        callbacks: &mut impl TextModifierGroupBaseCallbacks,
    ) {
        if self.rotation == value {
            return;
        }
        self.rotation = value;
        callbacks.rotation_changed();
        callbacks.notify_property_changed(Self::ROTATION_PROPERTY_KEY);
    }
    pub fn scale_x(&self) -> f32 {
        self.scale_x
    }
    pub fn set_scale_x(&mut self, value: f32, callbacks: &mut impl TextModifierGroupBaseCallbacks) {
        if self.scale_x == value {
            return;
        }
        self.scale_x = value;
        callbacks.scale_x_changed();
        callbacks.notify_property_changed(Self::SCALE_X_PROPERTY_KEY);
    }
    pub fn scale_y(&self) -> f32 {
        self.scale_y
    }
    pub fn set_scale_y(&mut self, value: f32, callbacks: &mut impl TextModifierGroupBaseCallbacks) {
        if self.scale_y == value {
            return;
        }
        self.scale_y = value;
        callbacks.scale_y_changed();
        callbacks.notify_property_changed(Self::SCALE_Y_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TextModifierGroupBaseCallbacks,
    ) -> TextModifierGroup {
        let mut cloned = TextModifierGroup::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextModifierGroupBaseCallbacks) {
        self.modifier_flags = object.modifier_flags;
        self.origin_x = object.origin_x;
        self.origin_y = object.origin_y;
        self.opacity = object.opacity;
        self.x = object.x;
        self.y = object.y;
        self.rotation = object.rotation;
        self.scale_x = object.scale_x;
        self.scale_y = object.scale_y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextModifierGroupBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::MODIFIER_FLAGS_PROPERTY_KEY => {
                self.modifier_flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::ORIGIN_X_PROPERTY_KEY => {
                self.origin_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_Y_PROPERTY_KEY => {
                self.origin_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OPACITY_PROPERTY_KEY => {
                self.opacity = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::X_PROPERTY_KEY => {
                self.x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::Y_PROPERTY_KEY => {
                self.y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ROTATION_PROPERTY_KEY => {
                self.rotation = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::SCALE_X_PROPERTY_KEY => {
                self.scale_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::SCALE_Y_PROPERTY_KEY => {
                self.scale_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
