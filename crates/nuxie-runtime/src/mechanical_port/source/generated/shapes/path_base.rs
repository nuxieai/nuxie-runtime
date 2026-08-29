use crate::mechanical_port::source::{core::binary_reader::BinaryReader, node::Node};

pub trait PathBaseCallbacks:
    crate::mechanical_port::source::generated::node_base::NodeBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn path_flags_changed(&mut self) {}
    fn is_hole_changed(&mut self) {}
}

pub struct PathBase {
    pub base: Node,
    path_flags: u32,
    is_hole: bool,
}

impl Default for PathBase {
    fn default() -> Self {
        Self {
            base: Node::default(),
            path_flags: 0,
            is_hole: false,
        }
    }
}

impl PathBase {
    pub const TYPE_KEY: u16 = 12;
    pub const PATH_FLAGS_PROPERTY_KEY: u16 = 128;
    pub const IS_HOLE_PROPERTY_KEY: u16 = 770;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn path_flags(&self) -> u32 {
        self.path_flags
    }
    pub fn set_path_flags(&mut self, value: u32, callbacks: &mut impl PathBaseCallbacks) {
        if !self.set_path_flags_value(value) {
            return;
        }
        callbacks.path_flags_changed();
        PathBaseCallbacks::notify_property_changed(callbacks, Self::PATH_FLAGS_PROPERTY_KEY);
    }

    pub(crate) fn set_path_flags_value(&mut self, value: u32) -> bool {
        if self.path_flags == value {
            return false;
        }
        self.path_flags = value;
        true
    }
    pub fn is_hole(&self) -> bool {
        self.is_hole
    }
    pub fn set_is_hole(&mut self, value: bool, callbacks: &mut impl PathBaseCallbacks) {
        if !self.set_is_hole_value(value) {
            return;
        }
        callbacks.is_hole_changed();
        PathBaseCallbacks::notify_property_changed(callbacks, Self::IS_HOLE_PROPERTY_KEY);
    }

    pub(crate) fn set_is_hole_value(&mut self, value: bool) -> bool {
        if self.is_hole == value {
            return false;
        }
        self.is_hole = value;
        true
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl PathBaseCallbacks) {
        self.path_flags = object.path_flags;
        self.is_hole = object.is_hole;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl PathBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::PATH_FLAGS_PROPERTY_KEY => {
                self.path_flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::IS_HOLE_PROPERTY_KEY => {
                self.is_hole = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for PathBase {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for PathBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
