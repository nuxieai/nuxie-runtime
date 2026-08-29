use crate::mechanical_port::source::{
    factory::RuntimeFactoryHandle, generated::assets::blob_asset_base::BlobAssetBase,
};
use std::{cell::OnceCell, sync::Arc};

pub struct BlobAsset {
    pub base: BlobAssetBase,
    bytes: Vec<u8>,
    script_asset: OnceCell<Arc<crate::RuntimeBlobAsset>>,
}

impl Default for BlobAsset {
    fn default() -> Self {
        Self {
            base: BlobAssetBase::default(),
            bytes: Vec::new(),
            script_asset: OnceCell::new(),
        }
    }
}

impl BlobAsset {
    pub fn decode(&mut self, data: &mut Vec<u8>, _factory: &RuntimeFactoryHandle) -> bool {
        self.bytes = std::mem::take(data);
        self.script_asset.take();
        true
    }

    pub fn file_extension(&self) -> &'static str {
        "blob"
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Retain the same decoded asset identity across Lua property wrappers.
    pub fn script_asset(&self) -> Arc<crate::RuntimeBlobAsset> {
        self.script_asset
            .get_or_init(|| {
                Arc::new(crate::RuntimeBlobAsset::from_decoded(
                    self.base.name(),
                    self.bytes.clone(),
                ))
            })
            .clone()
    }
}
