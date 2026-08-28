use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::viewmodel::viewmodel_instance_asset_base::ViewModelInstanceAssetBase,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStack},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct ViewModelInstanceAsset {
    pub base: ViewModelInstanceAssetBase,
    assets: Vec<CoreHandle>,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self, u32)>,
}

impl std::ops::Deref for ViewModelInstanceAsset {
    type Target = ViewModelInstanceAssetBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for ViewModelInstanceAsset {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
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

    pub fn add_asset(&mut self, asset: CoreHandle) {
        self.assets.push(asset);
    }

    pub fn assets(&self) -> &[CoreHandle] {
        &self.assets
    }

    #[cfg(feature = "tools")]
    pub fn changed_callback(&self) -> Option<fn(&mut Self, u32)> {
        self.changed_callback
    }

    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self, u32)>) {
        self.changed_callback = callback;
    }
}
