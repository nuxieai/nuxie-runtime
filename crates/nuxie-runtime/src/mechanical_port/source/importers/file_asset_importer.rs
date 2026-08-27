use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    assets::{file_asset::FileAsset, file_asset_contents::FileAssetContents},
    factory::Factory,
    file_asset_loader::FileAssetLoaderRef,
    status_code::StatusCode,
};

use super::import_stack::ImportStackObject;

pub struct FileAssetImporter {
    pub(crate) file_asset: NonNull<FileAsset>,
    pub(crate) file_asset_loader: Option<FileAssetLoaderRef>,
    pub(crate) factory: NonNull<Factory>,
    pub(crate) content: Option<Box<FileAssetContents>>,
}

impl FileAssetImporter {
    pub fn new(
        file_asset: NonNull<FileAsset>,
        file_asset_loader: Option<FileAssetLoaderRef>,
        factory: NonNull<Factory>,
    ) -> Self {
        Self {
            file_asset,
            file_asset_loader,
            factory,
            content: None,
        }
    }

    pub fn on_file_asset_contents(&mut self, contents: Box<FileAssetContents>) {
        assert!(self.content.is_none());
        self.content = Some(contents);
    }
}

impl ImportStackObject for FileAssetImporter {
    fn resolve(&mut self) -> StatusCode {
        let bytes = self
            .content
            .as_mut()
            .map_or(&[][..], |content| content.bytes().as_slice());
        let file_asset = unsafe { self.file_asset.as_mut() };
        let factory = unsafe { self.factory.as_mut() };
        if self
            .file_asset_loader
            .as_mut()
            .is_some_and(|loader| loader.load_contents(file_asset, bytes, factory))
        {
            return StatusCode::Ok;
        } else if !bytes.is_empty() {
            let content = self
                .content
                .as_mut()
                .expect("non-empty bytes came from in-band contents");
            file_asset.decode(content.bytes(), factory);
        }
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
