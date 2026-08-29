use std::{any::Any, cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    assets::file_asset_contents::FileAssetContents, core::CoreHandle,
    factory::RuntimeFactoryHandle, file_asset_loader::FileAssetLoaderRef,
    signed_content_header::SignedContentHeader, status_code::StatusCode,
};

use super::{
    file_asset_importer::{FileAssetImporter, FileAssetImporterBehavior},
    import_stack::ImportStackObject,
};

#[cfg(feature = "test-script-signature")]
pub const SCRIPT_VERIFICATION_PUBLIC_KEY: [u8; 32] = [
    180, 113, 86, 235, 225, 24, 110, 236, 105, 86, 201, 6, 73, 5, 203, 102, 81, 179, 12, 240, 226,
    55, 103, 134, 227, 94, 82, 187, 51, 178, 96, 46,
];

#[cfg(not(feature = "test-script-signature"))]
pub const SCRIPT_VERIFICATION_PUBLIC_KEY: [u8; 32] = [
    159, 202, 90, 135, 12, 153, 157, 21, 112, 103, 62, 130, 59, 196, 187, 236, 103, 210, 239, 227,
    175, 97, 222, 254, 70, 53, 212, 18, 191, 143, 101, 108,
];

pub struct InBandContent {
    text_asset: CoreHandle,
    bytes: Vec<u8>,
}

impl InBandContent {
    pub fn new(text_asset: CoreHandle, bytes: &[u8]) -> Self {
        Self {
            text_asset,
            bytes: bytes.to_vec(),
        }
    }
}

pub struct TextAssetImporter {
    base: FileAssetImporter,
    verification_set: Rc<RefCell<Vec<InBandContent>>>,
}

impl TextAssetImporter {
    pub fn new(
        text_asset: CoreHandle,
        loader: Option<FileAssetLoaderRef>,
        factory: RuntimeFactoryHandle,
        verification_set: Rc<RefCell<Vec<InBandContent>>>,
    ) -> Self {
        Self {
            base: FileAssetImporter::new(text_asset, loader, factory),
            verification_set,
        }
    }

    pub fn text_asset(&self) -> CoreHandle {
        self.base.file_asset.clone()
    }

    pub fn with_admission(
        mut self,
        admission: Option<crate::mechanical_port::source::file::ImportAdmissionRef>,
    ) -> Self {
        self.base = self.base.with_admission(admission);
        self
    }

    fn retain_text_asset_contents(&mut self, contents: CoreHandle) {
        let raw_content = contents
            .with_downcast_mut::<FileAssetContents, _>(|contents| {
                let header = SignedContentHeader::new(contents.bytes().as_slice());
                header.is_valid().then(|| header.content().to_vec())
            })
            .expect("TextAssetImporter content is FileAssetContents");
        if let Some(raw_content) = raw_content {
            self.verification_set
                .borrow_mut()
                .push(InBandContent::new(self.text_asset(), &raw_content));
        }
        FileAssetImporterBehavior::on_file_asset_contents(&mut self.base, contents);
    }
}

impl FileAssetImporterBehavior for TextAssetImporter {
    fn on_file_asset_contents(&mut self, contents: CoreHandle) {
        self.retain_text_asset_contents(contents);
    }
}

impl ImportStackObject for TextAssetImporter {
    fn resolve(&mut self) -> StatusCode {
        let status = self.base.resolve();
        if status != StatusCode::Ok {
            return status;
        }

        let Some(content) = self.base.content.as_ref() else {
            return StatusCode::Ok;
        };
        let signature = content
            .with_downcast_mut::<FileAssetContents, _>(|content| content.signature().clone())
            .expect("TextAssetImporter content is FileAssetContents");
        if signature.is_empty() {
            return StatusCode::Ok;
        }

        let mut verification_set = self.verification_set.borrow_mut();
        let mut combined_bytecode = Vec::new();
        for in_band in verification_set.iter() {
            combined_bytecode.extend_from_slice(&in_band.bytes);
        }

        let Ok(signature): Result<[u8; libhydrogen::sign::BYTES], _> = signature.try_into() else {
            return StatusCode::Ok;
        };
        let signature = libhydrogen::sign::Signature::from(signature);
        let public_key = libhydrogen::sign::PublicKey::from(SCRIPT_VERIFICATION_PUBLIC_KEY);
        let context = libhydrogen::sign::Context::from("RiveCode");
        let verified =
            libhydrogen::sign::verify(&signature, &combined_bytecode, &context, &public_key)
                .is_ok();
        for in_band in verification_set.iter() {
            in_band
                .text_asset
                .with_mut(|text_asset| text_asset.text_asset_set_verified(verified))
                .filter(|set| *set)
                .expect("verification participants remain TextAsset-derived");
        }
        verification_set.clear();
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
