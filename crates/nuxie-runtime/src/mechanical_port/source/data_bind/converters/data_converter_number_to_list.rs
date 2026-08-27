use crate::mechanical_port::source::{
    data_bind::data_values::{
        data_type::DataType,
        data_value::DataValue,
        data_value_list::{DataValueList, ViewModelInstanceListItem},
        data_value_number::DataValueNumber,
    },
    generated::data_bind::converters::data_converter_number_to_list_base::{
        DataConverterNumberToListBase, DataConverterNumberToListBaseCallbacks,
    },
};
use std::rc::Rc;
pub trait NumberToListItem: ViewModelInstanceListItem {
    fn set_view_model_instance(&self, value: Rc<dyn NumberToListInstance>);
    fn list_item(&self) -> Rc<dyn ViewModelInstanceListItem>;
}
pub trait NumberToListInstance {}
pub trait NumberToListFile {
    fn has_view_model(&self, id: u32) -> bool;
    fn make_item(&self) -> Rc<dyn NumberToListItem>;
    fn create_default_instance(&self, id: u32) -> Rc<dyn NumberToListInstance>;
}
pub struct DataConverterNumberToList {
    pub base: DataConverterNumberToListBase,
    file: Option<*mut dyn NumberToListFile>,
    output: DataValueList,
    list_items: Vec<Rc<dyn NumberToListItem>>,
}

impl Default for DataConverterNumberToList {
    fn default() -> Self {
        Self {
            base: DataConverterNumberToListBase::default(),
            file: None,
            output: DataValueList::default(),
            list_items: Vec::new(),
        }
    }
}

impl DataConverterNumberToList {
    pub fn new(view_model_id: u32) -> Self {
        let mut converter = Self::default();
        converter.base.set_view_model_id(
            view_model_id,
            &mut DataConverterNumberToListInitializationCallbacks,
        );
        converter
    }
    pub fn output_type(&self) -> DataType {
        DataType::List
    }
    pub fn convert<'a>(&'a mut self, input: &'a mut dyn DataValue) -> Option<&'a dyn DataValue> {
        if input.as_any().is::<DataValueList>() {
            return Some(input);
        }
        if let Some(number) = input.as_any().downcast_ref::<DataValueNumber>() {
            self.output.clear();
            let count = number.value().floor() as i32;
            let count = count.max(0) as usize;
            if let Some(file) = self
                .file
                .filter(|file| unsafe { (&**file).has_view_model(self.base.view_model_id()) })
            {
                let file = unsafe { &*file };
                while self.list_items.len() < count {
                    let item = file.make_item();
                    item.set_view_model_instance(
                        file.create_default_instance(self.base.view_model_id()),
                    );
                    self.list_items.push(item);
                }
                while self.list_items.len() > count {
                    self.list_items.pop();
                }
            } else {
                self.clear_items()
            }
            for item in &self.list_items {
                self.output.add_item(item.list_item());
            }
            return Some(&self.output);
        }
        None
    }
    fn clear_items(&mut self) {
        self.list_items.clear()
    }
    pub fn view_model_id_changed(&mut self) {
        self.clear_items();
        self.base.base.mark_converter_dirty()
    }
    pub fn set_file(&mut self, value: Option<*mut dyn NumberToListFile>) {
        self.file = value
    }
    pub fn file(&self) -> Option<*mut dyn NumberToListFile> {
        self.file
    }
    pub fn clone_converter(&self) -> Self {
        let mut cloned = Self::new(self.base.view_model_id());
        cloned.file = self.file;
        cloned
    }
}

impl DataConverterNumberToListBaseCallbacks for DataConverterNumberToList {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }

    fn view_model_id_changed(&mut self) {
        Self::view_model_id_changed(self);
    }
}

struct DataConverterNumberToListInitializationCallbacks;

impl DataConverterNumberToListBaseCallbacks for DataConverterNumberToListInitializationCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
