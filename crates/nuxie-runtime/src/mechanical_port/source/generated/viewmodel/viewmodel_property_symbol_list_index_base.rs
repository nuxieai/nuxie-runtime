use crate::mechanical_port::source::viewmodel::viewmodel_property_symbol_list_index::ViewModelPropertySymbolListIndex;

use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, view_model_property_symbol::ViewModelPropertySymbol,
};

pub struct ViewModelPropertySymbolListIndexBase {
    pub base: ViewModelPropertySymbol,
}

impl Default for ViewModelPropertySymbolListIndexBase {
    fn default() -> Self {
        Self {
            base: ViewModelPropertySymbol::default(),
        }
    }
}

impl ViewModelPropertySymbolListIndexBase {
    pub const TYPE_KEY: u16 = 564;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 563 | 430 | 429)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn clone_into(&self) -> ViewModelPropertySymbolListIndex {
        let mut cloned = ViewModelPropertySymbolListIndex::default();
        cloned.base.copy(self);
        cloned
    }
}
