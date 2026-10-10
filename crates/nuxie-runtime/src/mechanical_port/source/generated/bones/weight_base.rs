use std::rc::Rc;

use crate::mechanical_port::source::{
    bones::weight::Weight,
    shapes::vertex_cells::WeightCells,
    component::Component,
    core::{binary_reader::BinaryReader, field_types::core_uint_type::CoreUintType},
};

pub trait WeightBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn values_changed(&mut self) {}
    fn indices_changed(&mut self) {}
    fn notify_property_changed(&mut self, property_key: u16);
}

pub struct WeightBase {
    pub base: Component,
    // values and indices, shared with the weighted vertex's cells.
    cells: Rc<WeightCells>,
}

impl Default for WeightBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            cells: Rc::default(),
        }
    }
}

impl WeightBase {
    pub const TYPE_KEY: u16 = 45;
    pub const VALUES_PROPERTY_KEY: u16 = 102;
    pub const INDICES_PROPERTY_KEY: u16 = 103;

    pub fn is_type_of(type_key: u16) -> bool {
        Self::TYPE_KEY == type_key
            || crate::mechanical_port::source::generated::core_type_tree::has_ancestor(
                Self::TYPE_KEY,
                type_key,
            )
    }

    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub(crate) fn cells(&self) -> &Rc<WeightCells> {
        &self.cells
    }
    pub fn values(&self) -> u32 {
        self.cells.values()
    }
    pub fn indices(&self) -> u32 {
        self.cells.indices()
    }

    pub fn set_values<C: WeightBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if !self.set_values_value(value) {
            return;
        }
        callbacks.values_changed();
        WeightBaseCallbacks::notify_property_changed(callbacks, Self::VALUES_PROPERTY_KEY);
    }

    pub(crate) fn set_values_value(&mut self, value: u32) -> bool {
        if self.cells.values() == value {
            return false;
        }
        self.cells.set_values(value);
        true
    }

    pub fn set_indices<C: WeightBaseCallbacks>(&mut self, value: u32, callbacks: &mut C) {
        if !self.set_indices_value(value) {
            return;
        }
        callbacks.indices_changed();
        WeightBaseCallbacks::notify_property_changed(callbacks, Self::INDICES_PROPERTY_KEY);
    }

    pub(crate) fn set_indices_value(&mut self, value: u32) -> bool {
        if self.cells.indices() == value {
            return false;
        }
        self.cells.set_indices(value);
        true
    }

    pub fn clone_into<C: WeightBaseCallbacks>(&self, callbacks: &mut C) -> Weight {
        let mut cloned = Weight::default();
        cloned.base.copy(self, callbacks);
        cloned
    }

    pub fn copy<C: WeightBaseCallbacks>(&mut self, object: &Self, callbacks: &mut C) {
        self.cells.set_values(object.values());
        self.cells.set_indices(object.indices());
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
                self.cells.set_values(CoreUintType::deserialize(reader));
                true
            }
            Self::INDICES_PROPERTY_KEY => {
                self.cells.set_indices(CoreUintType::deserialize(reader));
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for WeightBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for WeightBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
