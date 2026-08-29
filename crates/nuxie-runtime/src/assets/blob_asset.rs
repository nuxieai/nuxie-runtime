//! Direct owner for pinned `src/assets/blob_asset.cpp` and its primary header.

use nuxie_render_api::Factory as RenderFactory;
use std::sync::Arc;

pub const FILE_EXTENSION: &str = "blob";

/// One retained BlobAsset occurrence.
///
/// The shared identity is the Rust counterpart of `rcp<BlobAsset>` used by
/// scripting and data binding. The payload remains owned by this occurrence;
/// [`Self::bytes`] only exposes the same borrowed byte view as the C++ span.
#[derive(Debug)]
pub struct RuntimeBlobAsset {
    name: Arc<str>,
    bytes: Arc<[u8]>,
}

impl RuntimeBlobAsset {
    pub fn new(name: impl Into<String>, bytes: Arc<[u8]>) -> Self {
        Self {
            name: Arc::from(name.into()),
            bytes,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn from_decoded(name: impl Into<String>, data: Vec<u8>) -> Self {
        let mut asset = Self::new(name, Arc::from([]));
        asset.replace_owned_bytes(data);
        asset
    }

    /// Mechanical `BlobAsset::decode`: take ownership of the complete input,
    /// replace the retained bytes, ignore the factory, and always succeed.
    pub fn decode(&mut self, data: Vec<u8>, _factory: &mut dyn RenderFactory) -> bool {
        self.replace_owned_bytes(data);
        true
    }

    pub fn file_extension(&self) -> &'static str {
        FILE_EXTENSION
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn bytes_arc(&self) -> Arc<[u8]> {
        Arc::clone(&self.bytes)
    }

    pub(crate) fn bytes_arc_ptr_eq(&self, bytes: &Arc<[u8]>) -> bool {
        Arc::ptr_eq(&self.bytes, bytes)
    }

    fn replace_owned_bytes(&mut self, data: Vec<u8>) {
        self.bytes = Arc::from(data.into_boxed_slice());
    }
}
