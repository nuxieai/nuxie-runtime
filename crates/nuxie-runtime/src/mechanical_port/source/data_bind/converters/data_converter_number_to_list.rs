use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::data_values::{
        data_type::DataType, data_value::DataValue, data_value_list::DataValueList,
        data_value_number::DataValueNumber,
    },
    file::RuntimeFileWeakHandle,
    generated::data_bind::converters::data_converter_number_to_list_base::{
        DataConverterNumberToListBase, DataConverterNumberToListBaseCallbacks,
    },
    viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem,
};
pub struct DataConverterNumberToList {
    pub base: DataConverterNumberToListBase,
    file: Option<RuntimeFileWeakHandle>,
    output: DataValueList,
    list_items: Vec<CoreHandle>,
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
    pub fn convert<'a>(&'a mut self, input: &'a dyn DataValue) -> Option<&'a dyn DataValue> {
        if input.as_any().is::<DataValueList>() {
            return Some(input);
        }
        if let Some(number) = input.as_any().downcast_ref::<DataValueNumber>() {
            self.output.clear();
            let count = number.value().floor() as i32;
            let count = count.max(0) as usize;
            let view_model_id = self.base.view_model_id() as usize;
            let populated = self.file.as_ref().is_some_and(|file| {
                file.with_file_mut(|file| {
                    let Some(view_model) = file.view_model(view_model_id) else {
                        return false;
                    };
                    while self.list_items.len() < count {
                        let Some(instance) =
                            file.create_default_view_model_instance(view_model.clone())
                        else {
                            break;
                        };
                        let Some(item) =
                            view_model.insert_sibling(ViewModelInstanceListItem::default())
                        else {
                            break;
                        };
                        item.with_downcast_mut::<ViewModelInstanceListItem, _>(|item| {
                            item.set_view_model_instance(Some(instance));
                        });
                        self.list_items.push(item);
                    }
                    while self.list_items.len() > count {
                        self.list_items.pop();
                    }
                    true
                })
                .unwrap_or(false)
            });
            if !populated {
                self.clear_items()
            }
            for item in &self.list_items {
                self.output.add_item(item.clone());
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
    pub fn set_file(&mut self, value: Option<RuntimeFileWeakHandle>) {
        self.file = value
    }
    pub fn file(&self) -> Option<RuntimeFileWeakHandle> {
        self.file.clone()
    }
    pub fn clone_converter(&self) -> Self {
        let mut cloned = Self::new(self.base.view_model_id());
        cloned.file = self.file.clone();
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

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterNumberToList
{
    fn convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        if let Some(value) = Self::convert(self, input) {
            output(value);
        }
    }

    fn reverse_convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(input);
    }

    fn output_type(&self) -> DataType {
        Self::output_type(self)
    }

    crate::data_converter_capability_lifecycle!(base.base);
}
