use crate::importers::{ImportContext, ImportStackKey};
use crate::*;

pub(crate) fn is_file_asset_referencer(definition: &'static Definition) -> bool {
    definition.is_a("TextStyle")
        || matches!(definition.name, "Image" | "AudioEvent")
        || definition_is_cpp_scripted_object(definition)
}

/// Pinned `FileAssetReferencer::registerReferencer`: the base relationship is
/// registered before each concrete referencer continues through its Super
/// import, so a missing BackboardImporter is immediately MissingObject.
pub(crate) fn register_referencer_succeeds(
    definition: &'static Definition,
    context: &ImportContext,
) -> bool {
    !is_file_asset_referencer(definition) || context.latest(ImportStackKey::Backboard)
}

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
        let asset = self
            .cpp_file_assets_for_backboard_owner(referencer)
            .into_iter()
            .nth(asset_index)?;
        if cpp_file_asset_matches_referencer(referencer, asset) {
            Some(asset)
        } else {
            None
        }
    }
}

fn cpp_file_asset_referencer_index(object: &RuntimeObject) -> Option<u64> {
    let definition = definition_by_type_key(object.type_key)?;
    if definition.is_a("TextStyle") {
        return object.uint_property("fontAssetId");
    }
    match definition.name {
        "Image" | "AudioEvent" => object.uint_property("assetId"),
        _ if definition_is_cpp_scripted_object(definition) => object.uint_property("scriptAssetId"),
        _ => None,
    }
}

fn cpp_file_asset_matches_referencer(referencer: &RuntimeObject, asset: &RuntimeObject) -> bool {
    let Some(definition) = definition_by_type_key(referencer.type_key) else {
        return false;
    };
    let Some(asset_definition) = definition_by_type_key(asset.type_key) else {
        return false;
    };
    if definition.is_a("TextStyle") {
        return asset_definition.is_a("FontAsset");
    }

    match definition.name {
        "Image" => asset_definition.is_a("ImageAsset"),
        "AudioEvent" => asset_definition.is_a("AudioAsset"),
        _ if definition_is_cpp_scripted_object(definition) => asset_definition.is_a("ScriptAsset"),
        _ => false,
    }
}
