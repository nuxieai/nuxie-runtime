use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, node::Node, transform_component::TransformComponent,
};

pub trait NodeBaseCallbacks:
    crate::mechanical_port::source::generated::transform_component_base::TransformComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn x_changed(&mut self) {}
    fn y_changed(&mut self) {}
    fn computed_local_x_changed(&mut self) {}
    fn computed_local_y_changed(&mut self) {}
    fn computed_world_x_changed(&mut self) {}
    fn computed_world_y_changed(&mut self) {}
    fn computed_root_x_changed(&mut self) {}
    fn computed_root_y_changed(&mut self) {}
    fn computed_width_changed(&mut self) {}
    fn computed_height_changed(&mut self) {}
    fn set_computed_local_x(&mut self, value: f32);
    fn computed_local_x(&mut self) -> f32;
    fn set_computed_local_y(&mut self, value: f32);
    fn computed_local_y(&mut self) -> f32;
    fn set_computed_world_x(&mut self, value: f32);
    fn computed_world_x(&mut self) -> f32;
    fn set_computed_world_y(&mut self, value: f32);
    fn computed_world_y(&mut self) -> f32;
    fn set_computed_root_x(&mut self, value: f32);
    fn computed_root_x(&mut self) -> f32;
    fn set_computed_root_y(&mut self, value: f32);
    fn computed_root_y(&mut self) -> f32;
    fn set_computed_width(&mut self, value: f32);
    fn computed_width(&mut self) -> f32;
    fn set_computed_height(&mut self, value: f32);
    fn computed_height(&mut self) -> f32;
}

pub struct NodeBase {
    pub base: TransformComponent,
    x: f32,
    y: f32,
}

impl Default for NodeBase {
    fn default() -> Self {
        Self {
            base: TransformComponent::default(),
            x: 0.0,
            y: 0.0,
        }
    }
}

impl NodeBase {
    pub const TYPE_KEY: u16 = 2;
    pub const X_PROPERTY_KEY: u16 = 13;
    pub const X_ARTBOARD_PROPERTY_KEY: u16 = 9;
    pub const Y_PROPERTY_KEY: u16 = 14;
    pub const Y_ARTBOARD_PROPERTY_KEY: u16 = 10;
    pub const COMPUTED_LOCAL_X_PROPERTY_KEY: u16 = 806;
    pub const COMPUTED_LOCAL_Y_PROPERTY_KEY: u16 = 807;
    pub const COMPUTED_WORLD_X_PROPERTY_KEY: u16 = 808;
    pub const COMPUTED_WORLD_Y_PROPERTY_KEY: u16 = 809;
    pub const COMPUTED_ROOT_X_PROPERTY_KEY: u16 = 864;
    pub const COMPUTED_ROOT_Y_PROPERTY_KEY: u16 = 865;
    pub const COMPUTED_WIDTH_PROPERTY_KEY: u16 = 810;
    pub const COMPUTED_HEIGHT_PROPERTY_KEY: u16 = 811;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn x(&self) -> f32 {
        self.x
    }
    pub fn set_x(&mut self, value: f32, callbacks: &mut impl NodeBaseCallbacks) {
        if !self.set_x_value(value) {
            return;
        }
        callbacks.x_changed();
        callbacks.notify_property_changed(Self::X_PROPERTY_KEY);
    }

    pub(crate) fn set_x_value(&mut self, value: f32) -> bool {
        if self.x == value {
            return false;
        }
        self.x = value;
        true
    }
    pub fn y(&self) -> f32 {
        self.y
    }
    pub fn set_y(&mut self, value: f32, callbacks: &mut impl NodeBaseCallbacks) {
        if !self.set_y_value(value) {
            return;
        }
        callbacks.y_changed();
        callbacks.notify_property_changed(Self::Y_PROPERTY_KEY);
    }

    pub(crate) fn set_y_value(&mut self, value: f32) -> bool {
        if self.y == value {
            return false;
        }
        self.y = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl NodeBaseCallbacks) -> Node {
        let mut cloned = Node::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl NodeBaseCallbacks) {
        self.x = object.x;
        self.y = object.y;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl NodeBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::X_PROPERTY_KEY => {
                self.x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::Y_PROPERTY_KEY => {
                self.y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for NodeBase {
    type Target = TransformComponent;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NodeBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
