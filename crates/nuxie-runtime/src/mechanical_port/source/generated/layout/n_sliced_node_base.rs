use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, layout::n_sliced_node::NSlicedNode, node::Node,
};

pub trait NSlicedNodeBaseCallbacks:
    crate::mechanical_port::source::generated::node_base::NodeBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn initial_width_changed(&mut self) {}
    fn initial_height_changed(&mut self) {}
    fn width_changed(&mut self) {}
    fn height_changed(&mut self) {}
}

pub struct NSlicedNodeBase {
    pub base: Node,
    initial_width: f32,
    initial_height: f32,
    width: f32,
    height: f32,
}

impl Default for NSlicedNodeBase {
    fn default() -> Self {
        Self {
            base: Node::default(),
            initial_width: 0.0,
            initial_height: 0.0,
            width: 0.0,
            height: 0.0,
        }
    }
}

impl NSlicedNodeBase {
    pub const TYPE_KEY: u16 = 508;
    pub const INITIAL_WIDTH_PROPERTY_KEY: u16 = 697;
    pub const INITIAL_HEIGHT_PROPERTY_KEY: u16 = 698;
    pub const WIDTH_PROPERTY_KEY: u16 = 699;
    pub const HEIGHT_PROPERTY_KEY: u16 = 700;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn initial_width(&self) -> f32 {
        self.initial_width
    }
    pub fn set_initial_width(&mut self, value: f32, callbacks: &mut impl NSlicedNodeBaseCallbacks) {
        if !self.set_initial_width_value(value) {
            return;
        }
        callbacks.initial_width_changed();
        callbacks.notify_property_changed(Self::INITIAL_WIDTH_PROPERTY_KEY);
    }

    pub(crate) fn set_initial_width_value(&mut self, value: f32) -> bool {
        if self.initial_width == value {
            return false;
        }
        self.initial_width = value;
        true
    }
    pub fn initial_height(&self) -> f32 {
        self.initial_height
    }
    pub fn set_initial_height(
        &mut self,
        value: f32,
        callbacks: &mut impl NSlicedNodeBaseCallbacks,
    ) {
        if !self.set_initial_height_value(value) {
            return;
        }
        callbacks.initial_height_changed();
        callbacks.notify_property_changed(Self::INITIAL_HEIGHT_PROPERTY_KEY);
    }

    pub(crate) fn set_initial_height_value(&mut self, value: f32) -> bool {
        if self.initial_height == value {
            return false;
        }
        self.initial_height = value;
        true
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn set_width(&mut self, value: f32, callbacks: &mut impl NSlicedNodeBaseCallbacks) {
        if !self.set_width_value(value) {
            return;
        }
        callbacks.width_changed();
        callbacks.notify_property_changed(Self::WIDTH_PROPERTY_KEY);
    }

    pub(crate) fn set_width_value(&mut self, value: f32) -> bool {
        if self.width == value {
            return false;
        }
        self.width = value;
        true
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn set_height(&mut self, value: f32, callbacks: &mut impl NSlicedNodeBaseCallbacks) {
        if !self.set_height_value(value) {
            return;
        }
        callbacks.height_changed();
        callbacks.notify_property_changed(Self::HEIGHT_PROPERTY_KEY);
    }

    pub(crate) fn set_height_value(&mut self, value: f32) -> bool {
        if self.height == value {
            return false;
        }
        self.height = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl NSlicedNodeBaseCallbacks) -> NSlicedNode {
        let mut cloned = NSlicedNode::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NSlicedNodeBaseCallbacks) {
        self.initial_width = object.initial_width;
        self.initial_height = object.initial_height;
        self.width = object.width;
        self.height = object.height;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NSlicedNodeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::INITIAL_WIDTH_PROPERTY_KEY => {
                self.initial_width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::INITIAL_HEIGHT_PROPERTY_KEY => {
                self.initial_height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::WIDTH_PROPERTY_KEY => {
                self.width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::HEIGHT_PROPERTY_KEY => {
                self.height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for NSlicedNodeBase {
    type Target = Node;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NSlicedNodeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
