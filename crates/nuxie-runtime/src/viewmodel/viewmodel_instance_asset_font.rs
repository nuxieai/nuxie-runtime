// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_asset_font.cpp`.
// Font asset serialized/live payload identity and pinned clone asymmetry.

/// One retained C++ `ViewModelInstanceAssetFont`: the cell owns both the
/// serialized index and private live Font payload. Retained consumers clone
/// this cell handle, eliminating the old index/stamp plus payload snapshot.
#[derive(Debug)]
pub(super) struct RuntimeOwnedViewModelFontAsset {
    property_index: usize,
    pub(super) cell: RuntimeViewModelCell,
}

impl RuntimeOwnedViewModelFontAsset {
    pub(super) fn new(property_index: usize, value: RuntimeFontAssetValue) -> Self {
        Self {
            property_index,
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::AssetFont(value)),
        }
    }

    pub(super) fn value(&self) -> RuntimeFontAssetValue {
        match self.cell.value() {
            RuntimeViewModelCellValue::AssetFont(value) => value,
            _ => unreachable!("font slot must retain an AssetFont cell"),
        }
    }

    pub(super) fn set_file_asset_index(&mut self, file_asset_index: u64) -> bool {
        self.cell.set_font_asset_index(file_asset_index)
    }

    pub(super) fn set_live_font_bytes(&mut self, font_bytes: Option<Arc<[u8]>>) -> bool {
        self.cell.set_live_font_bytes(font_bytes)
    }

    pub(super) fn apply_data_bind_value(&mut self, value: &RuntimeFontAssetValue) -> bool {
        self.cell.apply_font_asset_data_bind_value(value)
    }
}

impl Clone for RuntimeOwnedViewModelFontAsset {
    fn clone(&self) -> Self {
        // Pinned C++ `ViewModelInstanceAssetFont::clone` copies serialized
        // base fields into a fresh object but does not copy its private
        // `m_fontAsset` (`viewmodel_instance_asset_font.cpp:78-86`).
        Self::new(
            self.property_index,
            RuntimeFontAssetValue::from_file_asset_index(self.value().file_asset_index()),
        )
    }
}

fn runtime_owned_view_model_font_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelFontAsset> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyAssetFont").then(|| {
                        RuntimeOwnedViewModelFontAsset::new(
                            property_index,
                            RuntimeFontAssetValue::default(),
                        )
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_font_assets_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelFontAsset> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let file_asset_index = file.view_model_instance_font_asset_index_for_object(source)?;
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            Some(RuntimeOwnedViewModelFontAsset::new(
                property_index,
                RuntimeFontAssetValue::from_file_asset_index(file_asset_index),
            ))
        })
        .collect()
}

fn runtime_owned_view_model_imported_font_assets(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelFontAsset>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_font_assets_for_instance(
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
