use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue,
};
pub const DEPENDENTS: u32 = 1;
pub const BINDINGS: u32 = 2;
pub const BINDINGS_TARGET: u32 = 4;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusCode {
    Ok,
    MissingObject,
}
pub trait DataBindNode {
    fn clone_bind(&self) -> Box<dyn DataBindNode>;
    fn set_target_converter(&mut self, converter: *mut DataConverter);
    fn copy_file_from(&mut self, source: &dyn DataBindNode);
    fn bind_from_context(&mut self, context: *mut ());
    fn unbind(&mut self);
    fn update(&mut self, _force: bool);
}
pub trait ParentDataBind {
    fn target_origin(&self) -> bool;
    fn add_dirt(&mut self, dirt: u32, recurse: bool);
}
pub trait ConverterImporter {
    fn add_data_converter(&mut self, converter: *mut DataConverter);
    fn import_super(&mut self, converter: &mut DataConverter) -> StatusCode;
}
pub struct DataConverter {
    parent_data_bind: Option<*mut dyn ParentDataBind>,
    data_binds: Vec<Box<dyn DataBindNode>>,
    dirty_data_binds: Vec<*mut dyn DataBindNode>,
}
impl Default for DataConverter {
    fn default() -> Self {
        Self {
            parent_data_bind: None,
            data_binds: Vec::new(),
            dirty_data_binds: Vec::new(),
        }
    }
}
impl DataConverter {
    pub fn convert<'a>(&mut self, value: &'a dyn DataValue) -> &'a dyn DataValue {
        value
    }
    pub fn reverse_convert<'a>(&mut self, value: &'a dyn DataValue) -> &'a dyn DataValue {
        value
    }
    pub fn output_type(&self) -> DataType {
        DataType::None
    }
    pub fn import(&mut self, importer: Option<&mut dyn ConverterImporter>) -> StatusCode {
        let Some(importer) = importer else {
            return StatusCode::MissingObject;
        };
        importer.add_data_converter(self as *mut Self);
        importer.import_super(self)
    }
    pub fn bind_from_context(&mut self, data_context: *mut (), data_bind: *mut dyn ParentDataBind) {
        self.parent_data_bind = Some(data_bind);
        for bind in &mut self.data_binds {
            bind.bind_from_context(data_context);
        }
    }
    pub fn unbind(&mut self) {
        for bind in &mut self.data_binds {
            bind.unbind();
        }
    }
    pub fn mark_converter_dirty(&mut self) {
        if let Some(parent) = self.parent_data_bind {
            unsafe {
                let parent = &mut *parent;
                parent.add_dirt(
                    DEPENDENTS
                        | if parent.target_origin() {
                            BINDINGS_TARGET
                        } else {
                            BINDINGS
                        },
                    false,
                );
            }
        }
    }
    pub fn add_dirty_data_bind(&mut self, data_bind: *mut dyn DataBindNode) {
        self.mark_converter_dirty();
        self.dirty_data_binds.push(data_bind)
    }
    pub fn update(&mut self) {
        for bind in &mut self.data_binds {
            bind.update(false);
        }
    }
    pub fn copy(&mut self, object: &Self) {
        for data_bind in &object.data_binds {
            let mut clone = data_bind.clone_bind();
            clone.set_target_converter(self as *mut Self);
            clone.copy_file_from(data_bind.as_ref());
            self.data_binds.push(clone);
        }
    }
    pub fn advance(&mut self, _elapsed_time: f32) -> bool {
        false
    }
    pub fn reset(&mut self) {}
}
