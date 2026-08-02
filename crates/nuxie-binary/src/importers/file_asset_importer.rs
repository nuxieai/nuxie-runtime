use super::*;

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "FileAssetContents" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("FileAssetContents is owned by FileAssetImporter"),
        );
    }
    if definition.is_a("FileAsset") {
        return Some(
            imports_successfully(object, definition, context)
                .expect("FileAsset is owned by FileAssetImporter"),
        );
    }
    None
}

pub(super) fn dispatch_update_context(
    definition: &'static Definition,
    context: &mut ImportContext,
    script_assets_create_importers: bool,
) {
    if file_asset_creates_importer(definition.name, script_assets_create_importers) {
        update_context(definition, context, script_assets_create_importers);
    }
}

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
