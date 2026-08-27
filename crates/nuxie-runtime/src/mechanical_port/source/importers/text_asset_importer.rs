#![cfg(feature = "rive_scripting")]

use std::{any::Any, ptr::NonNull};

use crate::mechanical_port::source::{
    assets::{file_asset_contents::FileAssetContents, text_asset::TextAsset},
    factory::Factory,
    file_asset_loader::FileAssetLoaderRef,
    scripting::verify_hydrogen_signature_with_key,
    signed_content_header::SignedContentHeader,
    status_code::StatusCode,
};

use super::{file_asset_importer::FileAssetImporter, import_stack::ImportStackObject};

#[cfg(feature = "rive_test_signature")]
pub const SCRIPT_VERIFICATION_PUBLIC_KEY: [u8; 32] = [
    180, 113, 86, 235, 225, 24, 110, 236, 105, 86, 201, 6, 73, 5, 203, 102, 81, 179, 12, 240, 226,
    55, 103, 134, 227, 94, 82, 187, 51, 178, 96, 46,
];

#[cfg(not(feature = "rive_test_signature"))]
pub const SCRIPT_VERIFICATION_PUBLIC_KEY: [u8; 32] = [
    159, 202, 90, 135, 12, 153, 157, 21, 112, 103, 62, 130, 59, 196, 187, 236, 103, 210, 239, 227,
    175, 97, 222, 254, 70, 53, 212, 18, 191, 143, 101, 108,
];

pub struct InBandContent {
    text_asset: NonNull<TextAsset>,
    bytes: Vec<u8>,
}

impl InBandContent {
    pub fn new(text_asset: NonNull<TextAsset>, bytes: &[u8]) -> Self {
        Self {
            text_asset,
            bytes: bytes.to_vec(),
        }
    }
}

pub struct TextAssetImporter {
    base: FileAssetImporter,
    verification_set: NonNull<Vec<InBandContent>>,
}

impl TextAssetImporter {
    pub fn new(
        text_asset: NonNull<TextAsset>,
        loader: Option<FileAssetLoaderRef>,
        factory: NonNull<Factory>,
        verification_set: NonNull<Vec<InBandContent>>,
    ) -> Self {
        Self {
            base: FileAssetImporter::new(text_asset.cast(), loader, factory),
            verification_set,
        }
    }

    pub fn text_asset(&self) -> NonNull<TextAsset> {
        self.base.file_asset.cast()
    }

    pub fn on_file_asset_contents(&mut self, mut contents: Box<FileAssetContents>) {
        let header = SignedContentHeader::new(contents.bytes().as_slice());
        if header.is_valid() {
            unsafe {
                self.verification_set
                    .as_mut()
                    .push(InBandContent::new(self.text_asset(), header.content()))
            };
        }
        self.base.on_file_asset_contents(contents);
    }
}

impl ImportStackObject for TextAssetImporter {
    fn resolve(&mut self) -> StatusCode {
        let status = self.base.resolve();
        if status != StatusCode::Ok {
            return status;
        }

        let Some(content) = self.base.content.as_mut() else {
            return StatusCode::Ok;
        };
        if content.signature().is_empty() {
            return StatusCode::Ok;
        }

        let verification_set = unsafe { self.verification_set.as_mut() };
        let mut combined_bytecode = Vec::new();
        for in_band in verification_set.iter() {
            combined_bytecode.extend_from_slice(&in_band.bytes);
        }

        let signature = content.signature();
        if signature.len() != 64 {
            return StatusCode::Ok;
        }
        let verified = verify_hydrogen_signature_with_key(
            signature,
            &combined_bytecode,
            b"RiveCode",
            &SCRIPT_VERIFICATION_PUBLIC_KEY,
        );
        for in_band in verification_set.iter_mut() {
            unsafe { in_band.text_asset.as_mut().set_verified(verified) };
        }
        verification_set.clear();
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
