#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StatusCode {
    Ok,
    MissingObject,
}
pub trait DataResolver {
    fn resolve_path(&self, id: u32) -> Vec<u32>;
}
pub trait PathFile {
    fn data_resolver(&self) -> Option<&dyn DataResolver>;
}
pub trait PathImporter {
    fn file(&mut self) -> Option<*mut dyn PathFile>;
    fn import_super(&mut self, path: &mut DataBindPath) -> StatusCode;
}
pub struct DataBindPath {
    pub base: DataBindPathBase,
    file: Option<*mut dyn PathFile>,
}
impl Default for DataBindPath {
    fn default() -> Self {
        Self {
            base: DataBindPathBase::default(),
            file: None,
        }
    }
}
impl DataBindPath {
    pub fn decode_path(&mut self, bytes: &[u8]) {
        let mut index = 0;
        while index < bytes.len() {
            let mut value = 0u32;
            let mut shift = 0;
            loop {
                let byte = bytes[index];
                index += 1;
                value |= ((byte & 127) as u32) << shift;
                if byte & 128 == 0 {
                    break;
                }
                shift += 7;
            }
            self.base.path_buffer.push(value);
        }
    }
    pub fn copy_path(&mut self, other: &Self) {
        self.base.path_buffer.clone_from(&other.base.path_buffer);
        self.base.resolved = other.base.resolved
    }
    pub fn import(&mut self, importer: Option<&mut dyn PathImporter>) -> StatusCode {
        let Some(importer) = importer else {
            return StatusCode::MissingObject;
        };
        self.file = importer.file();
        importer.import_super(self)
    }
    pub fn path(&mut self) -> &mut Vec<u32> {
        &mut self.base.path_buffer
    }
    pub fn is_relative(&self) -> bool {
        self.base.is_relative()
    }
    pub fn resolved_path(&mut self) -> &[u32] {
        if !self.base.resolved {
            let Some(file) = self.file else {
                return &self.base.path_buffer;
            };
            if self.base.path_buffer.len() == 1 {
                if let Some(resolver) = unsafe { (&*file).data_resolver() } {
                    self.base.path_buffer = resolver.resolve_path(self.base.path_buffer[0]);
                }
            }
            self.base.resolved = true;
        }
        &self.base.path_buffer
    }
    pub fn set_file(&mut self, file: Option<*mut dyn PathFile>) {
        self.file = file
    }
    pub fn file(&self) -> Option<*mut dyn PathFile> {
        self.file
    }
    pub fn set_resolved(&mut self, value: bool) {
        self.base.resolved = value
    }
}

impl DataBindPathBaseCallbacks for DataBindPath {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.base.base.notify_property_changed(property_key);
    }

    fn decode_path(&mut self, value: &[u8]) {
        Self::decode_path(self, value);
    }

    fn copy_path(&mut self, object: &DataBindPathBase) {
        self.base.path_buffer.clone_from(&object.path_buffer);
        self.base.resolved = object.resolved;
    }
}
use crate::mechanical_port::source::generated::data_bind::data_bind_path_base::{
    DataBindPathBase, DataBindPathBaseCallbacks,
};
