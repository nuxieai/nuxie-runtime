use super::data_converter::StatusCode;
use super::data_converter_group::GroupConverter;
pub trait GroupItemImporter {
    fn add_group_item_referencer(&mut self, item: *mut DataConverterGroupItem);
    fn add_item_to_group(&mut self, item: *mut DataConverterGroupItem) -> bool;
    fn import_super(&mut self, item: &mut DataConverterGroupItem) -> StatusCode;
}
pub struct DataConverterGroupItem {
    data_converter: Option<*mut dyn GroupConverter>,
    owns_converter: bool,
}
impl Default for DataConverterGroupItem {
    fn default() -> Self {
        Self {
            data_converter: None,
            owns_converter: false,
        }
    }
}
impl Drop for DataConverterGroupItem {
    fn drop(&mut self) {
        if self.owns_converter {
            if let Some(converter) = self.data_converter.take() {
                unsafe {
                    drop(Box::from_raw(converter));
                }
            }
        }
    }
}
impl DataConverterGroupItem {
    pub fn import(&mut self, importer: Option<&mut dyn GroupItemImporter>) -> StatusCode {
        let Some(importer) = importer else {
            return StatusCode::MissingObject;
        };
        importer.add_group_item_referencer(self as *mut Self);
        if !importer.add_item_to_group(self as *mut Self) {
            return StatusCode::MissingObject;
        }
        importer.import_super(self)
    }
    pub fn converter(&self) -> Option<*mut dyn GroupConverter> {
        self.data_converter
    }
    pub fn set_converter(&mut self, value: Option<*mut dyn GroupConverter>) {
        self.data_converter = value
    }
    pub fn set_owns_converter(&mut self, value: bool) {
        self.owns_converter = value
    }
    pub fn clone_item(&self) -> Self {
        if let Some(converter) = self.data_converter {
            let cloned = unsafe { (&*converter).clone_box() };
            Self {
                data_converter: Some(Box::into_raw(cloned)),
                owns_converter: true,
            }
        } else {
            Self::default()
        }
    }
}
