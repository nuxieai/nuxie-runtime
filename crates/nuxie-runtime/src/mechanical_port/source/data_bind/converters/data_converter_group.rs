use super::data_converter_group_item::DataConverterGroupItem;
use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue,
};
pub trait GroupConverter {
    fn convert(&mut self, value: Box<dyn DataValue>, data_bind: *mut ()) -> Box<dyn DataValue>;
    fn reverse_convert(
        &mut self,
        value: Box<dyn DataValue>,
        data_bind: *mut (),
    ) -> Box<dyn DataValue>;
    fn output_type(&self) -> DataType;
    fn clone_box(&self) -> Box<dyn GroupConverter>;
    fn bind_from_context(&mut self, context: *mut (), data_bind: *mut ());
    fn unbind(&mut self);
    fn update(&mut self);
    fn reset(&mut self);
    fn advance(&mut self, elapsed: f32) -> bool;
}
#[derive(Default)]
pub struct DataConverterGroup {
    items: Vec<Box<DataConverterGroupItem>>,
}
impl DataConverterGroup {
    pub fn add_item(&mut self, item: Box<DataConverterGroupItem>) {
        self.items.push(item)
    }
    pub fn convert(
        &mut self,
        mut value: Box<dyn DataValue>,
        data_bind: *mut (),
    ) -> Box<dyn DataValue> {
        for item in &mut self.items {
            if let Some(converter) = item.converter() {
                value = unsafe { (&mut *converter).convert(value, data_bind) };
            }
        }
        value
    }
    pub fn reverse_convert(
        &mut self,
        mut value: Box<dyn DataValue>,
        data_bind: *mut (),
    ) -> Box<dyn DataValue> {
        for item in self.items.iter_mut().rev() {
            if let Some(converter) = item.converter() {
                value = unsafe { (&mut *converter).reverse_convert(value, data_bind) };
            }
        }
        value
    }
    pub fn output_type(&self, super_output: DataType) -> DataType {
        for item in self.items.iter().rev() {
            if let Some(converter) = item.converter() {
                let output = unsafe { (&*converter).output_type() };
                if output != DataType::Input {
                    return output;
                }
            }
        }
        super_output
    }
    pub fn items(&self) -> &[Box<DataConverterGroupItem>] {
        &self.items
    }
    pub fn clone_group(&self) -> Self {
        let mut cloned = Self::default();
        for item in &self.items {
            if item.converter().is_some() {
                cloned.add_item(Box::new(item.clone_item()));
            }
        }
        cloned
    }
    pub fn bind_from_context(&mut self, context: *mut (), data_bind: *mut ()) {
        for item in &mut self.items {
            if let Some(converter) = item.converter() {
                unsafe {
                    (&mut *converter).bind_from_context(context, data_bind);
                }
            }
        }
    }
    pub fn unbind(&mut self) {
        for item in &mut self.items {
            if let Some(converter) = item.converter() {
                unsafe {
                    (&mut *converter).unbind();
                }
            }
        }
    }
    pub fn update(&mut self) {
        for item in &mut self.items {
            if let Some(converter) = item.converter() {
                unsafe {
                    (&mut *converter).update();
                }
            }
        }
    }
    pub fn reset(&mut self) {
        for item in &mut self.items {
            if let Some(converter) = item.converter() {
                unsafe {
                    (&mut *converter).reset();
                }
            }
        }
    }
    pub fn advance(&mut self, elapsed: f32) -> bool {
        let mut did_update = false;
        for item in &mut self.items {
            if let Some(converter) = item.converter() {
                if unsafe { (&mut *converter).advance(elapsed) } {
                    did_update = true;
                }
            }
        }
        did_update
    }
}
