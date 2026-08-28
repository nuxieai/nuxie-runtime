use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::data_bind::converters::data_converter_group_item_base::DataConverterGroupItemBase,
    status_code::StatusCode,
};
pub trait GroupItemImporter {
    fn add_group_item_referencer(&mut self, item: CoreHandle);
    fn add_item_to_group(&mut self, item: CoreHandle) -> bool;
    fn import_super(&mut self, item: &mut DataConverterGroupItem) -> StatusCode;
}
pub struct DataConverterGroupItem {
    pub base: DataConverterGroupItemBase,
    data_converter: Option<CoreHandle>,
    owns_converter: bool,
}
impl Default for DataConverterGroupItem {
    fn default() -> Self {
        Self {
            base: DataConverterGroupItemBase::default(),
            data_converter: None,
            owns_converter: false,
        }
    }
}
impl Drop for DataConverterGroupItem {
    fn drop(&mut self) {
        if self.owns_converter {
            // Converter clones are CoreArena occurrences. This field is a
            // retained identity, never authority to reconstruct a Box.
            self.data_converter = None;
        }
    }
}
impl DataConverterGroupItem {
    pub fn import(&mut self, importer: Option<&mut dyn GroupItemImporter>) -> StatusCode {
        let Some(importer) = importer else {
            return StatusCode::MissingObject;
        };
        let Some(item) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_group_item_referencer(item.clone());
        if !importer.add_item_to_group(item) {
            return StatusCode::MissingObject;
        }
        importer.import_super(self)
    }
    pub fn converter(&self) -> Option<CoreHandle> {
        self.data_converter.clone()
    }
    pub fn set_converter(&mut self, value: Option<CoreHandle>) {
        self.data_converter = value
    }
    pub fn set_owns_converter(&mut self, value: bool) {
        self.owns_converter = value
    }
    pub fn clone_definition(&self) -> Self {
        let mut cloned = Self::default();
        cloned.base.set_converter_id_value(self.base.converter_id());
        cloned
    }
    pub fn complete_clone(source: &CoreHandle, cloned: &CoreHandle) -> bool {
        let Some(converter) = source.with_downcast::<Self, _>(Self::converter) else {
            return false;
        };
        if let Some(converter) = converter {
            let Some(converter) = converter.clone_occurrence() else {
                return false;
            };
            return cloned
                .with_downcast_mut::<Self, _>(|cloned| {
                    cloned.data_converter = Some(converter);
                    cloned.owns_converter = true;
                })
                .is_some();
        }
        true
    }
}
