use crate::mechanical_port::source::{
    assets::file_asset::FileAsset,
    generated::viewmodel::viewmodel_instance_asset_base::ViewModelInstanceAssetBase,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    refcnt::RiveRc,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct ViewModelInstanceAsset {
    pub base: ViewModelInstanceAssetBase,
    assets: Vec<RiveRc<FileAsset>>,
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self, u32)>,
}

impl ViewModelInstanceAsset {
    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<BackboardImporter>(
            crate::mechanical_port::source::backboard::Backboard::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        for asset in importer.assets() {
            self.assets.push(asset.clone());
        }
        self.base.import(import_stack)
    }

    pub fn add_asset(&mut self, asset: RiveRc<FileAsset>) {
        self.assets.push(asset);
    }

    pub fn assets(&self) -> &[RiveRc<FileAsset>] {
        &self.assets
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, u32)>) {
        self.changed_callback = callback;
    }
}
