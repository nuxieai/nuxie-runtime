use crate::mechanical_port::source::{
    core::CoreHandle, importers::import_stack::ImportStack, status_code::StatusCode,
};

#[derive(Default)]
pub struct FileAssetReferencer {
    file_asset: Option<CoreHandle>,
    referencer: Option<CoreHandle>,
}

impl FileAssetReferencer {
    pub fn asset(&self) -> Option<CoreHandle> {
        self.file_asset.clone()
    }

    pub fn referencer(&self) -> Option<CoreHandle> {
        self.referencer.clone()
    }

    pub fn register_referencer(
        &mut self,
        this: CoreHandle,
        import_stack: &mut ImportStack,
    ) -> StatusCode {
        let Some(backboard_importer) = import_stack.latest_backboard_importer() else {
            return StatusCode::MissingObject;
        };
        self.referencer = Some(this.clone());
        backboard_importer.add_file_asset_referencer(this);
        StatusCode::Ok
    }

    pub fn attach(&mut self, this: CoreHandle) {
        if self.referencer.as_ref() == Some(&this) {
            return;
        }
        self.referencer = Some(this.clone());
        if let Some(asset) = self.file_asset.as_ref() {
            asset
                .with_mut(|asset| {
                    asset
                        .as_file_asset_mut()
                        .expect("a retained asset remains FileAsset-derived")
                        .file_asset_base_mut()
                        .add_file_asset_referencer(this);
                })
                .expect("a retained FileAsset remains live while a clone attaches");
        }
    }

    pub fn set_asset_unattached(&mut self, asset: Option<CoreHandle>) {
        debug_assert!(self.referencer.is_none());
        self.file_asset = asset;
    }

    pub fn set_asset(&mut self, this: CoreHandle, asset: Option<CoreHandle>) {
        self.referencer = Some(this.clone());
        if let Some(previous) = self.file_asset.as_ref() {
            let _ = previous.with_mut(|asset| {
                if let Some(file_asset) = asset.as_file_asset_mut() {
                    file_asset
                        .file_asset_base_mut()
                        .remove_file_asset_referencer(&this);
                }
            });
        }
        self.file_asset.clone_from(&asset);
        if let Some(asset) = asset.as_ref() {
            asset
                .with_mut(|asset| {
                    asset
                        .as_file_asset_mut()
                        .expect("a supplied FileAsset handle must expose FileAsset capability")
                        .file_asset_base_mut()
                        .add_file_asset_referencer(this);
                })
                .expect("a supplied FileAsset handle must remain live");
        }
    }

    pub fn detach(&mut self, this: CoreHandle) {
        if let Some(asset) = self.file_asset.as_ref() {
            let _ = asset.with_mut(|asset| {
                if let Some(file_asset) = asset.as_file_asset_mut() {
                    file_asset
                        .file_asset_base_mut()
                        .remove_file_asset_referencer(&this);
                }
            });
        }
        self.file_asset = None;
    }

    pub fn asset_updated(&mut self) {}
}

impl Drop for FileAssetReferencer {
    fn drop(&mut self) {
        let (Some(asset), Some(referencer)) = (self.file_asset.as_ref(), self.referencer.as_ref())
        else {
            return;
        };
        let _ = asset.with_mut(|asset| {
            if let Some(file_asset) = asset.as_file_asset_mut() {
                file_asset
                    .file_asset_base_mut()
                    .remove_file_asset_referencer(referencer);
            }
        });
    }
}
