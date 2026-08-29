use crate::mechanical_port::source::{
    bones::{cubic_weight::CubicWeight, weight::Weight},
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
    generated::bones::weight_base::WeightBaseCallbacks,
};

pub trait CubicWeightBaseCallbacks: WeightBaseCallbacks {
    fn in_values_changed(&mut self) {}
    fn in_indices_changed(&mut self) {}
    fn out_values_changed(&mut self) {}
    fn out_indices_changed(&mut self) {}
}

pub struct CubicWeightBase {
    pub base: Weight,
    in_values: u32,
    in_indices: u32,
    out_values: u32,
    out_indices: u32,
}

impl Default for CubicWeightBase {
    fn default() -> Self {
        Self {
            base: Weight::default(),
            in_values: 255,
            in_indices: 1,
            out_values: 255,
            out_indices: 1,
        }
    }
}

impl CubicWeightBase {
    pub const TYPE_KEY: u16 = 46;
    pub const IN_VALUES_PROPERTY_KEY: u16 = 110;
    pub const IN_INDICES_PROPERTY_KEY: u16 = 111;
    pub const OUT_VALUES_PROPERTY_KEY: u16 = 112;
    pub const OUT_INDICES_PROPERTY_KEY: u16 = 113;
    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 45 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn in_values(&self) -> u32 {
        self.in_values
    }
    pub fn in_indices(&self) -> u32 {
        self.in_indices
    }
    pub fn out_values(&self) -> u32 {
        self.out_values
    }
    pub fn out_indices(&self) -> u32 {
        self.out_indices
    }

    pub fn set_in_values<C: CubicWeightBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if !self.set_in_values_value(value) {
            return;
        }
        callbacks.in_values_changed();
        crate::mechanical_port::source::generated::bones::weight_base::WeightBaseCallbacks::notify_property_changed(callbacks, Self::IN_VALUES_PROPERTY_KEY);
    }

    pub(crate) fn set_in_values_value(&mut self, value: u32) -> bool {
        if self.in_values == value {
            return false;
        }
        self.in_values = value;
        true
    }
    pub fn set_in_indices<C: CubicWeightBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if !self.set_in_indices_value(value) {
            return;
        }
        callbacks.in_indices_changed();
        crate::mechanical_port::source::generated::bones::weight_base::WeightBaseCallbacks::notify_property_changed(callbacks, Self::IN_INDICES_PROPERTY_KEY);
    }

    pub(crate) fn set_in_indices_value(&mut self, value: u32) -> bool {
        if self.in_indices == value {
            return false;
        }
        self.in_indices = value;
        true
    }
    pub fn set_out_values<C: CubicWeightBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if !self.set_out_values_value(value) {
            return;
        }
        callbacks.out_values_changed();
        crate::mechanical_port::source::generated::bones::weight_base::WeightBaseCallbacks::notify_property_changed(callbacks, Self::OUT_VALUES_PROPERTY_KEY);
    }

    pub(crate) fn set_out_values_value(&mut self, value: u32) -> bool {
        if self.out_values == value {
            return false;
        }
        self.out_values = value;
        true
    }
    pub fn set_out_indices<C: CubicWeightBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if !self.set_out_indices_value(value) {
            return;
        }
        callbacks.out_indices_changed();
        crate::mechanical_port::source::generated::bones::weight_base::WeightBaseCallbacks::notify_property_changed(callbacks, Self::OUT_INDICES_PROPERTY_KEY);
    }

    pub(crate) fn set_out_indices_value(&mut self, value: u32) -> bool {
        if self.out_indices == value {
            return false;
        }
        self.out_indices = value;
        true
    }

    pub fn clone_into<C: CubicWeightBaseCallbacks>(&self, callbacks: &mut C) -> CubicWeight {
        let mut cloned = CubicWeight::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy<C: CubicWeightBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.in_values = object.in_values;
        self.in_indices = object.in_indices;
        self.out_values = object.out_values;
        self.out_indices = object.out_indices;
        self.base.base.copy(&object.base.base, callbacks);
    }
    pub fn deserialize<C: CubicWeightBaseCallbacks>(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut C,
    ) -> bool {
        match property_key {
            Self::IN_VALUES_PROPERTY_KEY => {
                self.in_values = CoreUintType::deserialize(reader);
                true
            }
            Self::IN_INDICES_PROPERTY_KEY => {
                self.in_indices = CoreUintType::deserialize(reader);
                true
            }
            Self::OUT_VALUES_PROPERTY_KEY => {
                self.out_values = CoreUintType::deserialize(reader);
                true
            }
            Self::OUT_INDICES_PROPERTY_KEY => {
                self.out_indices = CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for CubicWeightBase {
    type Target = Weight;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicWeightBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
