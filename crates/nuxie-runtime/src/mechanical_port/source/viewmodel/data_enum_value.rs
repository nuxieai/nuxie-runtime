use crate::mechanical_port::source::{
    generated::viewmodel::data_enum_value_base::DataEnumValueBase,
    importers::{enum_importer::EnumImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
use std::ptr::NonNull;

pub struct DataEnumValue {
    pub base: DataEnumValueBase,
}

impl DataEnumValue {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<EnumImporter>(
            crate::mechanical_port::source::generated::viewmodel::data_enum_custom_base::DataEnumCustomBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        importer.add_value(NonNull::from(&mut *self));
        self.base.import(import_stack)
    }
}
