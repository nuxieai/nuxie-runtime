use crate::mechanical_port::source::{
    bones::weight::Weight,
    component::Component,
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
};

pub trait WeightBaseCallbacks {
    fn values_changed(&mut self) {}
    fn indices_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct WeightBase {
    pub base: Component,
    values: u32,
    indices: u32,
}

impl Default for WeightBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            values: 255,
            indices: 1,
        }
    }
}

impl WeightBase {
    pub const TYPE_KEY: u16 = 45;
    pub const VALUES_PROPERTY_KEY: u16 = 102;
    pub const INDICES_PROPERTY_KEY: u16 = 103;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn values(&self) -> u32 {
        self.values
    }
    pub fn indices(&self) -> u32 {
        self.indices
    }

    pub fn set_values<C: WeightBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if self.values == value {
            return;
        }
        self.values = value;
        callbacks.values_changed();
        callbacks.notify_property_changed(Self::VALUES_PROPERTY_KEY);
    }

    pub fn set_indices<C: WeightBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if self.indices == value {
            return;
        }
        self.indices = value;
        callbacks.indices_changed();
        callbacks.notify_property_changed(Self::INDICES_PROPERTY_KEY);
    }

    pub fn clone_into<C: WeightBaseCallbacks>(&self, callbacks: &mut C) -> Weight {
        let mut cloned = Weight::default();
        cloned.base.copy(self, callbacks);
        cloned
    }

    pub fn copy<C: WeightBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.values = object.values;
        self.indices = object.indices;
        self.base.copy(&object.base, callbacks);
    }

    pub fn deserialize<C: WeightBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::VALUES_PROPERTY_KEY => {
                self.values = CoreUintType::deserialize(reader);
                true
            }
            Self::INDICES_PROPERTY_KEY => {
                self.indices = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}
