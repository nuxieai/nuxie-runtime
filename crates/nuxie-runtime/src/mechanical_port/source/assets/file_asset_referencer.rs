use crate::mechanical_port::source::{
    core::CoreHandle, core_context::CoreContext, importers::import_stack::ImportStack,
    status_code::StatusCode,
};

#[derive(Default)]
pub struct FileAssetReferencer {
    file_asset: Option<CoreHandle>,
}

impl FileAssetReferencer {
    pub fn asset(&self) -> Option<CoreHandle> {
        self.file_asset
    }

    pub fn register_referencer(
        &mut self,
        this: CoreHandle,
        import_stack: &mut ImportStack,
    ) -> StatusCode {
        let Some(backboard_importer) = import_stack.latest_backboard_importer() else {
            return StatusCode::MissingObject;
        };
        backboard_importer.add_file_asset_referencer(this);
        StatusCode::Ok
    }

    pub fn set_asset(
        &mut self,
        this: CoreHandle,
        asset: Option<CoreHandle>,
        context: &mut CoreContext,
    ) {
        if let Some(previous) = self.file_asset {
            context
                .file_asset_mut(previous)
                .expect("a retained FileAsset handle must remain a FileAsset")
                .remove_file_asset_referencer(this);
        }
        self.file_asset = asset;
        if let Some(asset) = asset {
            context
                .file_asset_mut(asset)
                .expect("a supplied FileAsset handle must resolve as a FileAsset")
                .add_file_asset_referencer(this);
        }
    }

    pub fn detach(&mut self, this: CoreHandle, context: &mut CoreContext) {
        if let Some(asset) = self.file_asset {
            context
                .file_asset_mut(asset)
                .expect("a retained FileAsset handle must remain a FileAsset")
                .remove_file_asset_referencer(this);
        }
    }

    pub fn asset_updated(&mut self) {}
}
