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
    asset: CoreHandle,
    bytes: Vec<u8>,
}

/// Mirrors verifiesContentSignature: wrong-sized signatures never verify.
pub fn verifies_content_signature(signature: &[u8], content: &[u8]) -> bool {
    let Ok(signature): Result<[u8; nuxie_script_signature::SIGNATURE_BYTES], _> =
        signature.try_into()
    else {
        return false;
    };
    nuxie_script_signature::verify(
        &signature,
        content,
        b"RiveCode",
        &SCRIPT_VERIFICATION_PUBLIC_KEY,
    )
}

impl InBandContent {
    pub fn new(asset: CoreHandle, bytes: &[u8]) -> Self {
        Self {
            asset,
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
        asset: CoreHandle,
        loader: Option<FileAssetLoaderRef>,
        factory: RuntimeFactoryHandle,
        verification_set: Rc<RefCell<Vec<InBandContent>>>,
    ) -> Self {
        Self {
            base: FileAssetImporter::new(asset, loader, factory),
            verification_set,
        }
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
            self.verification_set.borrow_mut().push(InBandContent::new(
                self.base.file_asset.clone(),
                &raw_content,
            ));
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

        let Ok(signature): Result<[u8; nuxie_script_signature::SIGNATURE_BYTES], _> =
            signature.try_into()
        else {
            return StatusCode::Ok;
        };
        let verified = verifies_content_signature(&signature, &combined_bytecode);
        for in_band in verification_set.iter() {
            in_band
                .asset
                .with_mut(|asset| {
                    asset
                        .as_file_asset_mut()
                        .expect("verification participants remain FileAsset-derived")
                        .file_asset_base_mut()
                        .set_verified(verified);
                })
                .expect("verification participant remains alive");
            in_band
                .asset
                .with_downcast_mut::<crate::source::assets::shader_asset::ShaderAsset, _>(
                    |shader| {
                        shader.admit();
                    },
                );
        }
        verification_set.clear();
        StatusCode::Ok
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
