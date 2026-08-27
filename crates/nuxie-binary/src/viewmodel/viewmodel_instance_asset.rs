//! Direct Rust owner for pinned C++
//! `src/viewmodel/viewmodel_instance_asset.cpp`.

use crate::{
    RuntimeFile, RuntimeImportStatus, RuntimeObject, assets::cpp_file_assets_contains,
    definition_by_type_key,
};

impl RuntimeFile {
    /// The `FileAsset` handles copied from `BackboardImporter::assets()` when
    /// this value imported. The immutable Rust file owns those handles, so the
    /// equivalent snapshot is the imported, backboard-owned prefix preceding
    /// this value in file order.
    pub fn view_model_instance_asset_file_assets(&self, value_id: usize) -> Vec<&RuntimeObject> {
        let Some(value) = self.object(value_id) else {
            return Vec::new();
        };

        self.view_model_instance_asset_file_assets_for_object(value)
    }

    pub fn view_model_instance_asset_file_assets_for_object(
        &self,
        value: &RuntimeObject,
    ) -> Vec<&RuntimeObject> {
        self.cpp_view_model_instance_asset_file_assets(value)
    }

    pub fn resolved_file_asset_for_view_model_instance_asset(
        &self,
        value_id: usize,
    ) -> Option<&RuntimeObject> {
        let value = self.object(value_id)?;
        self.resolved_file_asset_for_view_model_instance_asset_object(value)
    }

    pub fn resolved_file_asset_for_view_model_instance_asset_object(
        &self,
        value: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let asset_index = usize::try_from(value.uint_property("propertyValue")?).ok()?;
        self.cpp_view_model_instance_asset_file_assets(value)
            .into_iter()
            .nth(asset_index)
    }

    fn cpp_view_model_instance_asset_file_assets<'a>(
        &'a self,
        value: &RuntimeObject,
    ) -> Vec<&'a RuntimeObject> {
        let Some(definition) = definition_by_type_key(value.type_key) else {
            return Vec::new();
        };
        if !definition.is_a("ViewModelInstanceAsset") {
            return Vec::new();
        }

        let Ok(value_index) = usize::try_from(value.id) else {
            return Vec::new();
        };
        if self.import_status(value_index) != Some(RuntimeImportStatus::Imported) {
            return Vec::new();
        }

        self.objects
            .iter()
            .take(value_index)
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }

                let object = object.as_ref()?;
                cpp_file_assets_contains(object).then_some(object)
            })
            .collect()
    }
}
