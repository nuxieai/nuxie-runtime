pub trait PathImporter {
    fn file(&mut self) -> Option<RuntimeFileWeakHandle>;
    fn import_super(&mut self, path: &mut DataBindPath) -> StatusCode;
}
pub struct DataBindPath {
    pub base: DataBindPathBase,
    file: RuntimeFileWeakHandle,
}
impl Default for DataBindPath {
    fn default() -> Self {
        Self {
            base: DataBindPathBase::default(),
            file: RuntimeFileWeakHandle::default(),
        }
    }
}
impl DataBindPath {
    pub(crate) fn clone_core(&self) -> Self {
        let mut cloned = Self::default();
        cloned.copy_path(self);
        cloned.base.set_is_relative_value(self.base.is_relative());
        cloned
    }

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
        let Some(file) = importer.file() else {
            return StatusCode::MissingObject;
        };
        self.file = file;
        importer.import_super(self)
    }
    pub fn import_stack(
        &mut self,
        stack: &mut crate::mechanical_port::source::importers::import_stack::ImportStack,
    ) -> StatusCode {
        use crate::mechanical_port::source::{
            generated::backboard_base::BackboardBase,
            importers::backboard_importer::BackboardImporter,
        };
        let Some(importer) = stack.latest::<BackboardImporter>(BackboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        let Some(file) = importer.file() else {
            return StatusCode::MissingObject;
        };
        self.file = file;
        self.base.base.import(stack)
    }
    pub fn path(&self) -> &[u32] {
        &self.base.path_buffer
    }
    pub fn is_relative(&self) -> bool {
        self.base.is_relative()
    }
    pub fn resolved_path(&mut self) -> &[u32] {
        if !self.base.resolved {
            if self.file.upgrade().is_none() {
                return &self.base.path_buffer;
            }
            if self.base.path_buffer.len() == 1 {
                let path_id = self.base.path_buffer[0];
                let resolved = self
                    .file
                    .with_file(|file| {
                        file.manifest()?
                            .with_downcast::<ManifestAsset, _>(|resolver| {
                                resolver.resolve_path(path_id as i32).to_vec()
                            })
                    })
                    .flatten();
                if let Some(resolved) = resolved {
                    self.base.path_buffer = resolved;
                }
            }
            self.base.resolved = true;
        }
        &self.base.path_buffer
    }
    pub fn set_file(&mut self, file: RuntimeFileWeakHandle) {
        self.file = file
    }
    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.file.clone()
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
use crate::mechanical_port::source::{
    assets::manifest_asset::ManifestAsset,
    file::RuntimeFileWeakHandle,
    generated::data_bind::data_bind_path_base::{DataBindPathBase, DataBindPathBaseCallbacks},
    status_code::StatusCode,
};
