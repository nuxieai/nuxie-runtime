//! Direct Rust owner for pinned C++
//! `src/data_bind/context/context_value_asset_image.cpp`.

use crate::RuntimeViewModelImage;
use crate::data_bind_graph::RuntimeDataBindGraphValue;
use nuxie_binary::RuntimeFile;

/// Safe-Rust equivalent of the integer id plus private `ImageAsset` retained
/// by `ViewModelInstanceAssetImage`, `DataValueAssetImage`, and
/// `BindablePropertyAsset`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeImageAssetValue {
    file_asset_index: u64,
    live_image: Option<RuntimeViewModelImage>,
}

impl RuntimeImageAssetValue {
    pub(crate) fn new(file_asset_index: u64, live_image: Option<RuntimeViewModelImage>) -> Self {
        Self {
            file_asset_index,
            live_image,
        }
    }

    pub(crate) fn file_asset_index(&self) -> u64 {
        self.file_asset_index
    }

    pub(crate) fn live_image(&self) -> Option<&RuntimeViewModelImage> {
        self.live_image.as_ref()
    }

    pub(crate) fn same_runtime_value(&self, other: &Self) -> bool {
        self.file_asset_index == other.file_asset_index
            && match (&self.live_image, &other.live_image) {
                (Some(current), Some(next)) => current.ptr_eq(next),
                (None, None) => true,
                _ => false,
            }
    }
}

/// Pinned `fileAsset`: accept only a valid file entry whose concrete type is
/// `ImageAsset`.
pub(crate) fn file_asset_global(file: &RuntimeFile, asset_index: u64) -> Option<u32> {
    let asset = file.file_asset(usize::try_from(asset_index).ok()?)?;
    (asset.type_name == "ImageAsset").then_some(asset.id)
}

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    crate::context_value_enum::integer_payload(next).map(RuntimeDataBindGraphValue::Asset)
}
