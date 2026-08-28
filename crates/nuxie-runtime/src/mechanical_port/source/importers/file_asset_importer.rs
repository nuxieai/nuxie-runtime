use std::any::Any;

use crate::mechanical_port::source::{
    assets::file_asset_contents::FileAssetContents, core::CoreHandle,
    factory::RuntimeFactoryHandle, file_asset_loader::FileAssetLoaderRef, status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub trait FileAssetImporterBehavior {
    fn on_file_asset_contents(&mut self, contents: CoreHandle);
}

pub struct FileAssetImporter {
    pub(crate) file_asset: CoreHandle,
    pub(crate) file_asset_loader: Option<FileAssetLoaderRef>,
    pub(crate) factory: RuntimeFactoryHandle,
    pub(crate) content: Option<CoreHandle>,
}

impl FileAssetImporter {
    pub fn new(
        file_asset: CoreHandle,
        file_asset_loader: Option<FileAssetLoaderRef>,
        factory: RuntimeFactoryHandle,
    ) -> Self {
        Self {
            file_asset,
            file_asset_loader,
            factory,
            content: None,
        }
    }

    fn retain_file_asset_contents(&mut self, contents: CoreHandle) {
        assert!(self.content.is_none());
        self.content = Some(contents);
    }
}

impl FileAssetImporterBehavior for FileAssetImporter {
    fn on_file_asset_contents(&mut self, contents: CoreHandle) {
        self.retain_file_asset_contents(contents);
    }
}

impl ImportStackObject for FileAssetImporter {
    fn resolve(&mut self) -> StatusCode {
        let mut bytes = self.content.as_ref().map_or_else(Vec::new, |content| {
            content
                .with_downcast_mut::<FileAssetContents, _>(|content| {
                    std::mem::take(content.bytes())
                })
                .expect("FileAssetImporter content is FileAssetContents")
        });
        let loaded = self.file_asset_loader.as_ref().is_some_and(|loader| {
            loader.with_loader_mut(|loader| {
                loader.load_contents(self.file_asset.clone(), &bytes, &self.factory)
            })
        });
        if loaded {
            return StatusCode::Ok;
        } else if !bytes.is_empty() {
            self.file_asset
                .with_mut(|file_asset| {
                    file_asset
                        .as_file_asset_mut()
                        .expect("FileAssetImporter retains a FileAsset")
                        .file_asset_decode(&mut bytes, &self.factory);
                })
                .expect("FileAssetImporter file asset remains live");
        }
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
