use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle, generated::assets::blob_asset_base::BlobAssetBase,
};

pub struct BlobAsset {
    pub base: BlobAssetBase,
    bytes: Vec<u8>,
}

impl Default for BlobAsset {
    fn default() -> Self {
        Self {
            base: BlobAssetBase::default(),
            bytes: Vec::new(),
        }
    }
}

impl BlobAsset {
    pub fn decode(&mut self, data: &mut Vec<u8>, _factory: &RuntimeFactoryHandle) -> bool {
        self.bytes = std::mem::take(data);
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "blob"
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
