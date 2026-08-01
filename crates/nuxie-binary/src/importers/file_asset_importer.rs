use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "FileAssetContents" {
        return Some(context.latest(ImportStackKey::FileAsset));
    }
    definition
        .is_a("FileAsset")
        .then(|| definition.name == "ManifestAsset" || context.latest(ImportStackKey::Backboard))
}

pub(super) fn update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
    script_assets_create_importers: bool,
) {
    if file_asset_creates_importer(definition.name, script_assets_create_importers) {
        context.make_latest(ImportStackKey::FileAsset);
    }
}
