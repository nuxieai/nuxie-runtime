use crate::mechanical_port::source::{
    container_component::ContainerComponent, core::binary_reader::BinaryReader,
    text::text_modifier_range::TextModifierRange,
};

pub trait TextModifierRangeBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn modify_from_changed(&mut self) {}
    fn modify_to_changed(&mut self) {}
    fn strength_changed(&mut self) {}
    fn units_value_changed(&mut self) {}
    fn type_value_changed(&mut self) {}
    fn mode_value_changed(&mut self) {}
    fn clamp_changed(&mut self) {}
    fn falloff_from_changed(&mut self) {}
    fn falloff_to_changed(&mut self) {}
    fn offset_changed(&mut self) {}
    fn run_id_changed(&mut self) {}
}

pub struct TextModifierRangeBase {
    pub base: ContainerComponent,
    modify_from: f32,
    modify_to: f32,
    strength: f32,
    units_value: u32,
    type_value: u32,
    mode_value: u32,
    clamp: bool,
    falloff_from: f32,
    falloff_to: f32,
    offset: f32,
    run_id: u32,
}

impl Default for TextModifierRangeBase {
    fn default() -> Self {
        Self {
            base: ContainerComponent::default(),
            modify_from: 0.0,
            modify_to: 1.0,
            strength: 1.0,
            units_value: 0,
            type_value: 0,
            mode_value: 0,
            clamp: false,
            falloff_from: 0.0,
            falloff_to: 1.0,
            offset: 0.0,
            run_id: u32::MAX,
        }
    }
}

impl TextModifierRangeBase {
    pub const TYPE_KEY: u16 = 158;
    pub const MODIFY_FROM_PROPERTY_KEY: u16 = 327;
    pub const MODIFY_TO_PROPERTY_KEY: u16 = 336;
    pub const STRENGTH_PROPERTY_KEY: u16 = 334;
    pub const UNITS_VALUE_PROPERTY_KEY: u16 = 316;
    pub const TYPE_VALUE_PROPERTY_KEY: u16 = 325;
    pub const MODE_VALUE_PROPERTY_KEY: u16 = 326;
    pub const CLAMP_PROPERTY_KEY: u16 = 333;
    pub const FALLOFF_FROM_PROPERTY_KEY: u16 = 317;
    pub const FALLOFF_TO_PROPERTY_KEY: u16 = 318;
    pub const OFFSET_PROPERTY_KEY: u16 = 319;
    pub const RUN_ID_PROPERTY_KEY: u16 = 378;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn modify_from(&self) -> f32 {
        self.modify_from
    }
    pub fn set_modify_from(
        &mut self,
        value: f32,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) {
        if self.modify_from == value {
            return;
        }
        self.modify_from = value;
        callbacks.modify_from_changed();
        callbacks.notify_property_changed(Self::MODIFY_FROM_PROPERTY_KEY);
    }
    pub fn modify_to(&self) -> f32 {
        self.modify_to
    }
    pub fn set_modify_to(
        &mut self,
        value: f32,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) {
        if self.modify_to == value {
            return;
        }
        self.modify_to = value;
        callbacks.modify_to_changed();
        callbacks.notify_property_changed(Self::MODIFY_TO_PROPERTY_KEY);
    }
    pub fn strength(&self) -> f32 {
        self.strength
    }
    pub fn set_strength(
        &mut self,
        value: f32,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) {
        if self.strength == value {
            return;
        }
        self.strength = value;
        callbacks.strength_changed();
        callbacks.notify_property_changed(Self::STRENGTH_PROPERTY_KEY);
    }
    pub fn units_value(&self) -> u32 {
        self.units_value
    }
    pub fn set_units_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) {
        if self.units_value == value {
            return;
        }
        self.units_value = value;
        callbacks.units_value_changed();
        callbacks.notify_property_changed(Self::UNITS_VALUE_PROPERTY_KEY);
    }
    pub fn type_value(&self) -> u32 {
        self.type_value
    }
    pub fn set_type_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) {
        if self.type_value == value {
            return;
        }
        self.type_value = value;
        callbacks.type_value_changed();
        callbacks.notify_property_changed(Self::TYPE_VALUE_PROPERTY_KEY);
    }
    pub fn mode_value(&self) -> u32 {
        self.mode_value
    }
    pub fn set_mode_value(
        &mut self,
        value: u32,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) {
        if self.mode_value == value {
            return;
        }
        self.mode_value = value;
        callbacks.mode_value_changed();
        callbacks.notify_property_changed(Self::MODE_VALUE_PROPERTY_KEY);
    }
    pub fn clamp(&self) -> bool {
        self.clamp
    }
    pub fn set_clamp(&mut self, value: bool, callbacks: &mut impl TextModifierRangeBaseCallbacks) {
        if self.clamp == value {
            return;
        }
        self.clamp = value;
        callbacks.clamp_changed();
        callbacks.notify_property_changed(Self::CLAMP_PROPERTY_KEY);
    }
    pub fn falloff_from(&self) -> f32 {
        self.falloff_from
    }
    pub fn set_falloff_from(
        &mut self,
        value: f32,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) {
        if self.falloff_from == value {
            return;
        }
        self.falloff_from = value;
        callbacks.falloff_from_changed();
        callbacks.notify_property_changed(Self::FALLOFF_FROM_PROPERTY_KEY);
    }
    pub fn falloff_to(&self) -> f32 {
        self.falloff_to
    }
    pub fn set_falloff_to(
        &mut self,
        value: f32,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) {
        if self.falloff_to == value {
            return;
        }
        self.falloff_to = value;
        callbacks.falloff_to_changed();
        callbacks.notify_property_changed(Self::FALLOFF_TO_PROPERTY_KEY);
    }
    pub fn offset(&self) -> f32 {
        self.offset
    }
    pub fn set_offset(&mut self, value: f32, callbacks: &mut impl TextModifierRangeBaseCallbacks) {
        if self.offset == value {
            return;
        }
        self.offset = value;
        callbacks.offset_changed();
        callbacks.notify_property_changed(Self::OFFSET_PROPERTY_KEY);
    }
    pub fn run_id(&self) -> u32 {
        self.run_id
    }
    pub fn set_run_id(&mut self, value: u32, callbacks: &mut impl TextModifierRangeBaseCallbacks) {
        if self.run_id == value {
            return;
        }
        self.run_id = value;
        callbacks.run_id_changed();
        callbacks.notify_property_changed(Self::RUN_ID_PROPERTY_KEY);
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) -> TextModifierRange {
        let mut cloned = TextModifierRange::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextModifierRangeBaseCallbacks) {
        self.modify_from = object.modify_from;
        self.modify_to = object.modify_to;
        self.strength = object.strength;
        self.units_value = object.units_value;
        self.type_value = object.type_value;
        self.mode_value = object.mode_value;
        self.clamp = object.clamp;
        self.falloff_from = object.falloff_from;
        self.falloff_to = object.falloff_to;
        self.offset = object.offset;
        self.run_id = object.run_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextModifierRangeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::MODIFY_FROM_PROPERTY_KEY => {
                self.modify_from = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MODIFY_TO_PROPERTY_KEY => {
                self.modify_to = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::STRENGTH_PROPERTY_KEY => {
                self.strength = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::UNITS_VALUE_PROPERTY_KEY => {
                self.units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::TYPE_VALUE_PROPERTY_KEY => {
                self.type_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::MODE_VALUE_PROPERTY_KEY => {
                self.mode_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::CLAMP_PROPERTY_KEY => {
                self.clamp = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::FALLOFF_FROM_PROPERTY_KEY => {
                self.falloff_from = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::FALLOFF_TO_PROPERTY_KEY => {
                self.falloff_to = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::OFFSET_PROPERTY_KEY => {
                self.offset = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::RUN_ID_PROPERTY_KEY => {
                self.run_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
