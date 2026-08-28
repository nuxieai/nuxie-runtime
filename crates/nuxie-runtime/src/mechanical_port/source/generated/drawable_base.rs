use crate::mechanical_port::source::{core::binary_reader::BinaryReader, node::Node};

pub trait DrawableBaseCallbacks:
    crate::mechanical_port::source::generated::node_base::NodeBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn blend_mode_value_changed(&mut self) {}
    fn drawable_flags_changed(&mut self) {}
}

pub struct DrawableBase {
    pub base: Node,
    blend_mode_value: u32,
    drawable_flags: u32,
}

impl Default for DrawableBase {
    fn default() -> Self {
        Self {
            base: Node::default(),
            blend_mode_value: 3,
            drawable_flags: 0,
        }
    }
}

impl DrawableBase {
    pub const TYPE_KEY: u16 = 13;
    pub const BLEND_MODE_VALUE_PROPERTY_KEY: u16 = 23;
    pub const DRAWABLE_FLAGS_PROPERTY_KEY: u16 = 129;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn blend_mode_value(&self) -> u32 {
        self.blend_mode_value
    }
    pub fn set_blend_mode_value(&mut self, value: u32, callbacks: &mut impl DrawableBaseCallbacks) {
        if !self.set_blend_mode_value_value(value) {
            return;
        }
        callbacks.blend_mode_value_changed();
        callbacks.notify_property_changed(Self::BLEND_MODE_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_blend_mode_value_value(&mut self, value: u32) -> bool {
        if self.blend_mode_value == value {
            return false;
        }
        self.blend_mode_value = value;
        true
    }
    pub fn drawable_flags(&self) -> u32 {
        self.drawable_flags
    }
    pub fn set_drawable_flags(&mut self, value: u32, callbacks: &mut impl DrawableBaseCallbacks) {
        if !self.set_drawable_flags_value(value) {
            return;
        }
        callbacks.drawable_flags_changed();
        callbacks.notify_property_changed(Self::DRAWABLE_FLAGS_PROPERTY_KEY);
    }

    pub(crate) fn set_drawable_flags_value(&mut self, value: u32) -> bool {
        if self.drawable_flags == value {
            return false;
        }
        self.drawable_flags = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl DrawableBaseCallbacks) {
        self.blend_mode_value = object.blend_mode_value;
        self.drawable_flags = object.drawable_flags;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl DrawableBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::BLEND_MODE_VALUE_PROPERTY_KEY => {
                self.blend_mode_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::DRAWABLE_FLAGS_PROPERTY_KEY => {
                self.drawable_flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for DrawableBase {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for DrawableBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
