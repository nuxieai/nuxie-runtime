// Direct Rust owner for pinned C++
// `src/viewmodel/viewmodel_instance_asset_blob.cpp`.

/// Pinned `ViewModelInstanceAssetBlob::propertyValue(uint32_t)` plus its
/// `propertyValueChanged()` override. `RuntimeViewModelCell::set_value`
/// performs the retained `Bindings` dirt cascade and `onValueChanged`
/// delegation after the serialized field changes.
pub(crate) fn view_model_instance_asset_blob_property_value(
    cell: &RuntimeViewModelCell,
    file_asset_index: u64,
) -> bool {
    let RuntimeViewModelCellValue::AssetBlob(mut value) = cell.value() else {
        debug_assert!(false, "blob propertyValue on non-blob cell");
        return false;
    };
    if !value.set_file_asset_index(file_asset_index) {
        return false;
    }
    cell.set_value(RuntimeViewModelCellValue::AssetBlob(value))
}

/// Pinned `ViewModelInstanceAssetBlob::value(BlobAsset*)` under the approved
/// owned-byte asset representation. An `Arc<[u8]>` is the retained identity
/// available at this host boundary.
pub(crate) fn view_model_instance_asset_blob_value_bytes(
    cell: &RuntimeViewModelCell,
    bytes: Option<Arc<[u8]>>,
) -> bool {
    let RuntimeViewModelCellValue::AssetBlob(mut value) = cell.value() else {
        debug_assert!(false, "blob value on non-blob cell");
        return false;
    };
    let same_live_blob = match (value.live_blob_bytes_arc(), bytes.as_ref()) {
        (Some(current), Some(next)) => Arc::ptr_eq(&current, next),
        (None, None) => true,
        _ => false,
    };
    let index_was_not_sentinel =
        value.file_asset_index() != RuntimeBlobAssetValue::MISSING_FILE_ASSET_INDEX;
    if !value.set_live_blob_bytes(bytes) {
        return false;
    }
    let changed = cell.set_value(RuntimeViewModelCellValue::AssetBlob(value));

    // For a different Blob, pinned C++ first writes propertyValue(-1), whose
    // callback publishes once when the index was not already the sentinel,
    // and then publishes explicitly after assigning m_blobAsset.
    if !same_live_blob && index_was_not_sentinel {
        cell.notify_bindings_value_changed();
    }
    changed
}

/// The same pinned `value(BlobAsset*)` path for consumers which retain the
/// decoded blob object, preserving Blob pointer identity exactly.
pub(crate) fn view_model_instance_asset_blob_value(
    cell: &RuntimeViewModelCell,
    asset: Option<Arc<RuntimeBlobAsset>>,
) -> bool {
    let RuntimeViewModelCellValue::AssetBlob(mut value) = cell.value() else {
        debug_assert!(false, "blob value on non-blob cell");
        return false;
    };
    let same_live_blob = match (value.live_blob_asset(), asset.as_ref()) {
        (Some(current), Some(next)) => Arc::ptr_eq(current, next),
        (None, None) => true,
        _ => false,
    };
    let index_was_not_sentinel =
        value.file_asset_index() != RuntimeBlobAssetValue::MISSING_FILE_ASSET_INDEX;
    if !value.set_live_blob_asset(asset) {
        return false;
    }
    let changed = cell.set_value(RuntimeViewModelCellValue::AssetBlob(value));
    if !same_live_blob && index_was_not_sentinel {
        cell.notify_bindings_value_changed();
    }
    changed
}

/// Pinned `ViewModelInstanceAssetBlob::applyValue(DataValueInteger*)`: prefer
/// the retained live Blob and return; otherwise apply null through `value()`
/// before falling through to the serialized integer value.
pub(crate) fn view_model_instance_asset_blob_apply_value(
    cell: &RuntimeViewModelCell,
    value: &RuntimeBlobAssetValue,
) -> bool {
    if let Some(asset) = value.live_blob_asset() {
        return view_model_instance_asset_blob_value(cell, Some(Arc::clone(asset)));
    }
    let mut changed = view_model_instance_asset_blob_value_bytes(cell, None);
    changed |= view_model_instance_asset_blob_property_value(cell, value.file_asset_index());
    changed
}

#[derive(Debug)]
pub(super) struct RuntimeOwnedViewModelBlobAsset {
    property_index: usize,
    pub(super) cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelBlobAsset {
    pub(super) fn new(property_index: usize, value: RuntimeBlobAssetValue) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::AssetBlob(value)),
        }
    }

    pub(super) fn value(&self) -> RuntimeBlobAssetValue {
        match self.cell.value() {
            RuntimeViewModelCellValue::AssetBlob(value) => value,
            _ => unreachable!("blob slot must retain an AssetBlob cell"),
        }
    }

    pub(super) fn set_file_asset_index(&mut self, file_asset_index: u64) -> bool {
        self.cell.set_blob_asset_index(file_asset_index)
    }

    pub(super) fn set_live_bytes(&mut self, bytes: Option<Arc<[u8]>>) -> bool {
        self.cell.set_live_blob_bytes(bytes)
    }

    pub(super) fn apply_data_bind_value(&mut self, value: &RuntimeBlobAssetValue) -> bool {
        self.cell.apply_blob_asset_data_bind_value(value)
    }
}

impl Clone for RuntimeOwnedViewModelBlobAsset {
    fn clone(&self) -> Self {
        // Upstream clone copies serialized base fields and intentionally does
        // not copy the private live Blob asset. Its inherited `assets()` list
        // is the file-owned asset registry under the approved arena/owned-byte
        // adaptation, so it remains retained by the surrounding RuntimeFile
        // instead of being duplicated on each value.
        Self::new(
            self.property_index,
            RuntimeBlobAssetValue::from_file_asset_index(self.value().file_asset_index()),
        )
    }
}

fn runtime_owned_view_model_blob_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelBlobAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyAssetBlob").then(|| {
                        RuntimeOwnedViewModelBlobAsset::new(
                            property_index,
                            RuntimeBlobAssetValue::default(),
                        )
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_blob_assets_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelBlobAsset> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            if source.type_name != "ViewModelInstanceAssetBlob" {
                return None;
            }
            let file_asset_index = file.view_model_instance_blob_asset_index_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelBlobAsset::new(
                property_index,
                RuntimeBlobAssetValue::from_file_asset_index(file_asset_index),
            ))
        })
        .collect()
}

fn runtime_owned_view_model_imported_blob_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelBlobAsset>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_blob_assets_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}
