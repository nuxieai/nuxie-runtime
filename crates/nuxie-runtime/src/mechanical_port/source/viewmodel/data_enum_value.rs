use crate::mechanical_port::source::{
    generated::viewmodel::data_enum_value_base::{DataEnumValueBase, DataEnumValueBaseCallbacks},
    importers::{enum_importer::EnumImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
#[derive(Default)]
pub struct DataEnumValue {
    pub base: DataEnumValueBase,
}

impl DataEnumValue {
    pub fn set_value(&mut self, value: String) {
        let mut callbacks = DataEnumValueCallbacks::default();
        self.base.set_value(value, &mut callbacks);
        if let Some(property_key) = callbacks.property_key {
            self.base.base.notify_property_changed(property_key);
        }
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<EnumImporter>(
            crate::mechanical_port::source::generated::viewmodel::data_enum_custom_base::DataEnumCustomBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        let Some(value) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_value(value);
        self.base.import(import_stack)
    }
}

#[derive(Default)]
struct DataEnumValueCallbacks {
    property_key: Option<u16>,
}

impl DataEnumValueBaseCallbacks for DataEnumValueCallbacks {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.property_key = Some(property_key);
    }
}
