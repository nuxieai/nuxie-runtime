use crate::*;

impl RuntimeFile {
    pub fn resolved_file_asset_for_object(&self, object_id: usize) -> Option<&RuntimeObject> {
        let object = self.object(object_id)?;
        self.resolved_file_asset_for_referencer(object)
    }

    pub fn resolved_file_asset_for_referencer(
        &self,
        referencer: &RuntimeObject,
    ) -> Option<&RuntimeObject> {
        let object_id = usize::try_from(referencer.id).ok()?;
        if self.import_status(object_id) != Some(RuntimeImportStatus::Imported) {
            return None;
        }

        let asset_index = usize::try_from(cpp_file_asset_referencer_index(referencer)?).ok()?;
        let asset = self.file_asset(asset_index)?;
        if cpp_file_asset_matches_referencer(referencer, asset) {
            Some(asset)
        } else {
            None
        }
    }
}

fn cpp_file_asset_referencer_index(object: &RuntimeObject) -> Option<u64> {
    let definition = definition_by_type_key(object.type_key)?;
    match definition.name {
        "Image" | "AudioEvent" => object.uint_property("assetId"),
        "TextStyle" => object.uint_property("fontAssetId"),
        _ if definition_is_cpp_scripted_object(definition) => object.uint_property("scriptAssetId"),
        _ => None,
    }
}

fn cpp_file_asset_matches_referencer(referencer: &RuntimeObject, asset: &RuntimeObject) -> bool {
    let Some(definition) = definition_by_type_key(referencer.type_key) else {
        return false;
    };

    match definition.name {
        "Image" => asset.type_name == "ImageAsset",
        "AudioEvent" => asset.type_name == "AudioAsset",
        "TextStyle" => asset.type_name == "FontAsset",
        _ if definition_is_cpp_scripted_object(definition) => asset.type_name == "ScriptAsset",
        _ => false,
    }
}
