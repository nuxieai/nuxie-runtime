use crate::mechanical_port::source::{
    animation::state_machine_layer_component::StateMachineLayerComponent,
    core::binary_reader::BinaryReader,
};

pub trait LayerStateBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn flags_changed(&mut self) {}
}

pub struct LayerStateBase {
    pub base: StateMachineLayerComponent,
    flags: u32,
}

impl Default for LayerStateBase {
    fn default() -> Self {
        Self {
            base: StateMachineLayerComponent::default(),
            flags: 0,
        }
    }
}

impl LayerStateBase {
    pub const TYPE_KEY: u16 = 60;
    pub const FLAGS_PROPERTY_KEY: u16 = 536;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 66)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn flags(&self) -> u32 {
        self.flags
    }
    pub fn set_flags(&mut self, value: u32, callbacks: &mut impl LayerStateBaseCallbacks) {
        if self.flags == value {
            return;
        }
        self.flags = value;
        callbacks.flags_changed();
        callbacks.notify_property_changed(Self::FLAGS_PROPERTY_KEY);
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LayerStateBaseCallbacks) {
        self.flags = object.flags;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl LayerStateBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::FLAGS_PROPERTY_KEY => {
                self.flags = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
