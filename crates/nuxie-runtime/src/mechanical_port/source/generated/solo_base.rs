use crate::mechanical_port::source::{core::binary_reader::BinaryReader, node::Node, solo::Solo};

pub trait SoloBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn active_component_id_changed(&mut self) {}
}

pub struct SoloBase {
    pub base: Node,
    active_component_id: u32,
}

impl Default for SoloBase {
    fn default() -> Self {
        Self {
            base: Node::default(),
            active_component_id: 0,
        }
    }
}

impl SoloBase {
    pub const TYPE_KEY: u16 = 147;
    pub const ACTIVE_COMPONENT_ID_PROPERTY_KEY: u16 = 296;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn active_component_id(&self) -> u32 {
        self.active_component_id
    }
    pub fn set_active_component_id(&mut self, value: u32, callbacks: &mut impl SoloBaseCallbacks) {
        if self.active_component_id == value {
            return;
        }
        self.active_component_id = value;
        callbacks.active_component_id_changed();
        callbacks.notify_property_changed(Self::ACTIVE_COMPONENT_ID_PROPERTY_KEY);
    }
    pub fn clone_into(&self, callbacks: &mut impl SoloBaseCallbacks) -> Solo {
        let mut cloned = Solo::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl SoloBaseCallbacks) {
        self.active_component_id = object.active_component_id;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl SoloBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ACTIVE_COMPONENT_ID_PROPERTY_KEY => {
                self.active_component_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
