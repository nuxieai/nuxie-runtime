use crate::mechanical_port::source::{
    core::CoreHandle, generated::assets::file_asset_contents_base::FileAssetContentsBase,
    importers::import_stack::ImportStack, status_code::StatusCode,
};

pub struct FileAssetContents {
    pub base: FileAssetContentsBase,
    bytes: Vec<u8>,
    signature: Vec<u8>,
}

impl Default for FileAssetContents {
    fn default() -> Self {
        Self {
            base: FileAssetContentsBase::default(),
            bytes: Vec::new(),
            signature: Vec::new(),
        }
    }
}

impl FileAssetContents {
    pub fn import_handle(this: &CoreHandle, import_stack: &mut ImportStack) -> StatusCode {
        let Some(file_asset_importer) = import_stack.latest_file_asset_importer() else {
            return StatusCode::MissingObject;
        };
        // TextAssetImporter reads this occurrence's bytes in the virtual call.
        // Retain its identity, but do not borrow it until that callback returns.
        file_asset_importer.on_file_asset_contents(this.clone());
        this.with_downcast_mut::<Self, _>(|contents| contents.base.import(import_stack))
            .expect("FileAssetContents remains live after importer ownership transfer")
    }

    pub fn decode_bytes(&mut self, value: &[u8]) {
        self.bytes = value.to_vec();
    }

    pub fn copy_bytes(&mut self, _object: &FileAssetContentsBase) {
        panic!("FileAssetContents::copyBytes must never be called");
    }

    pub fn bytes(&mut self) -> &mut Vec<u8> {
        &mut self.bytes
    }

    pub fn decode_signature(&mut self, value: &[u8]) {
        self.signature = value.to_vec();
    }

    pub fn copy_signature(&mut self, _object: &FileAssetContentsBase) {
        panic!("FileAssetContents::copySignature must never be called");
    }

    pub fn signature(&mut self) -> &mut Vec<u8> {
        &mut self.signature
    }
}
