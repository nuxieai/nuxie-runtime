use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::{
        backboard_base::BackboardBase,
        data_bind::converters::{
            data_converter_group_base::DataConverterGroupBase,
            data_converter_group_item_base::DataConverterGroupItemBase,
        },
    },
    importers::{
        backboard_importer::BackboardImporter,
        data_converter_group_importer::DataConverterGroupImporter, import_stack::ImportStack,
    },
    status_code::StatusCode,
};
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
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(backboard) = stack.latest::<BackboardImporter>(BackboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(item) = self.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        backboard.add_data_converter_group_item_referencer(item.clone());
        let Some(group) =
            stack.latest::<DataConverterGroupImporter>(DataConverterGroupBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        group
            .group()
            .with_downcast_mut::<super::data_converter_group::DataConverterGroup, _>(|group| {
                group.add_item(item)
            })
            .expect("DataConverterGroupImporter retains the actual group owner");
        self.base.base.import(stack)
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
