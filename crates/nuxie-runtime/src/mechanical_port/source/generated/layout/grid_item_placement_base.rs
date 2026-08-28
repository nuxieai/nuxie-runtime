use crate::mechanical_port::source::{
    component::Component, core::binary_reader::BinaryReader,
    layout::grid_item_placement::GridItemPlacement,
};

pub trait GridItemPlacementBaseCallbacks:
    crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn grid_column_changed(&mut self) {}
    fn grid_row_changed(&mut self) {}
    fn grid_column_span_changed(&mut self) {}
    fn grid_row_span_changed(&mut self) {}
}

pub struct GridItemPlacementBase {
    pub base: Component,
    grid_column: i16,
    grid_row: i16,
    grid_column_span: u16,
    grid_row_span: u16,
}

impl Default for GridItemPlacementBase {
    fn default() -> Self {
        Self {
            base: Component::default(),
            grid_column: 0,
            grid_row: 0,
            grid_column_span: 1,
            grid_row_span: 1,
        }
    }
}

impl GridItemPlacementBase {
    pub const TYPE_KEY: u16 = 1068;
    pub const GRID_COLUMN_PROPERTY_KEY: u16 = 1047;
    pub const GRID_ROW_PROPERTY_KEY: u16 = 1048;
    pub const GRID_COLUMN_SPAN_PROPERTY_KEY: u16 = 1049;
    pub const GRID_ROW_SPAN_PROPERTY_KEY: u16 = 1050;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn grid_column(&self) -> i16 {
        self.grid_column
    }
    pub fn set_grid_column(
        &mut self,
        value: i16,
        callbacks: &mut impl GridItemPlacementBaseCallbacks,
    ) {
        if !self.set_grid_column_value(value) {
            return;
        }
        callbacks.grid_column_changed();
        callbacks.notify_property_changed(Self::GRID_COLUMN_PROPERTY_KEY);
    }

    pub(crate) fn set_grid_column_value(&mut self, value: i16) -> bool {
        if self.grid_column == value {
            return false;
        }
        self.grid_column = value;
        true
    }
    pub fn grid_row(&self) -> i16 {
        self.grid_row
    }
    pub fn set_grid_row(
        &mut self,
        value: i16,
        callbacks: &mut impl GridItemPlacementBaseCallbacks,
    ) {
        if !self.set_grid_row_value(value) {
            return;
        }
        callbacks.grid_row_changed();
        callbacks.notify_property_changed(Self::GRID_ROW_PROPERTY_KEY);
    }

    pub(crate) fn set_grid_row_value(&mut self, value: i16) -> bool {
        if self.grid_row == value {
            return false;
        }
        self.grid_row = value;
        true
    }
    pub fn grid_column_span(&self) -> u16 {
        self.grid_column_span
    }
    pub fn set_grid_column_span(
        &mut self,
        value: u16,
        callbacks: &mut impl GridItemPlacementBaseCallbacks,
    ) {
        if !self.set_grid_column_span_value(value) {
            return;
        }
        callbacks.grid_column_span_changed();
        callbacks.notify_property_changed(Self::GRID_COLUMN_SPAN_PROPERTY_KEY);
    }

    pub(crate) fn set_grid_column_span_value(&mut self, value: u16) -> bool {
        if self.grid_column_span == value {
            return false;
        }
        self.grid_column_span = value;
        true
    }
    pub fn grid_row_span(&self) -> u16 {
        self.grid_row_span
    }
    pub fn set_grid_row_span(
        &mut self,
        value: u16,
        callbacks: &mut impl GridItemPlacementBaseCallbacks,
    ) {
        if !self.set_grid_row_span_value(value) {
            return;
        }
        callbacks.grid_row_span_changed();
        callbacks.notify_property_changed(Self::GRID_ROW_SPAN_PROPERTY_KEY);
    }

    pub(crate) fn set_grid_row_span_value(&mut self, value: u16) -> bool {
        if self.grid_row_span == value {
            return false;
        }
        self.grid_row_span = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl GridItemPlacementBaseCallbacks,
    ) -> GridItemPlacement {
        let mut cloned = GridItemPlacement::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl GridItemPlacementBaseCallbacks) {
        self.grid_column = object.grid_column;
        self.grid_row = object.grid_row;
        self.grid_column_span = object.grid_column_span;
        self.grid_row_span = object.grid_row_span;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl GridItemPlacementBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::GRID_COLUMN_PROPERTY_KEY => {
                self.grid_column = crate::mechanical_port::source::core::field_types::core_int_type::CoreIntType::deserialize(reader);
                true
            }
            Self::GRID_ROW_PROPERTY_KEY => {
                self.grid_row = crate::mechanical_port::source::core::field_types::core_int_type::CoreIntType::deserialize(reader);
                true
            }
            Self::GRID_COLUMN_SPAN_PROPERTY_KEY => {
                self.grid_column_span = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::GRID_ROW_SPAN_PROPERTY_KEY => {
                self.grid_row_span = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for GridItemPlacementBase {
    type Target = Component;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for GridItemPlacementBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
