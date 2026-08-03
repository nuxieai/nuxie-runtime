//! Blob-asset source compatibility owned by C++ `ContextValueAssetBlob`.

use crate::RuntimeBlobAssetValue;
use crate::data_bind_graph::RuntimeDataBindGraphValue;
use crate::view_model_cell::RuntimeViewModelCellValue;

pub(crate) fn graph_from_cell(value: &RuntimeBlobAssetValue) -> RuntimeDataBindGraphValue {
    RuntimeDataBindGraphValue::AssetBlob(value.clone())
}

pub(crate) fn cell_from_graph(value: &RuntimeBlobAssetValue) -> RuntimeViewModelCellValue {
    RuntimeViewModelCellValue::AssetBlob(value.clone())
}

pub(crate) fn matching(next: &RuntimeDataBindGraphValue) -> Option<RuntimeDataBindGraphValue> {
    match next {
        RuntimeDataBindGraphValue::AssetBlob(value) => Some(graph_from_cell(value)),
        RuntimeDataBindGraphValue::Integer(value) => Some(RuntimeDataBindGraphValue::AssetBlob(
            RuntimeBlobAssetValue::from_file_asset_index(*value),
        )),
        _ => None,
    }
}
