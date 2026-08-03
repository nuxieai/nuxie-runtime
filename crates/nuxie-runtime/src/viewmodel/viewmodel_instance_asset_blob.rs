// Direct Rust owner for pinned C++
// `src/viewmodel/viewmodel_instance_asset_blob.cpp`.

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
        // not copy the private live Blob asset.
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
