use super::*;
use crate::assets::file_asset_contents::ImportedFileAssetRecord;

const SCRIPT_VERIFICATION_PUBLIC_KEY: [u8; libhydrogen::sign::PUBLICKEYBYTES] = [
    159, 202, 90, 135, 12, 153, 157, 21, 112, 103, 62, 130, 59, 196, 187, 236, 103, 210, 239, 227,
    175, 97, 222, 254, 70, 53, 212, 18, 191, 143, 101, 108,
];

/// Borrowed file storage replaces the pinned `SimpleArray` copy while the
/// immutable RuntimeFile owns every byte for the full verification replay.
#[derive(Debug, Clone, Copy)]
struct InBandContent<'a> {
    text_asset: &'a RuntimeObject,
    bytes: &'a [u8],
}

#[derive(Debug, Clone, Copy)]
struct TextAssetContents<'a> {
    bytes: Option<&'a [u8]>,
    signature: Option<&'a [u8]>,
}

/// The borrowed object and retained contents are the Rust equivalents of the
/// pinned `TextAsset*` and inherited `FileAssetImporter::m_content`.
#[derive(Debug)]
struct TextAssetImporter<'a> {
    text_asset: &'a RuntimeObject,
    content: Option<TextAssetContents<'a>>,
}

impl<'a> TextAssetImporter<'a> {
    fn new(text_asset: &'a RuntimeObject) -> Self {
        Self {
            text_asset,
            content: None,
        }
    }

    fn text_asset(&self) -> &'a RuntimeObject {
        self.text_asset
    }

    fn on_file_asset_contents(
        &mut self,
        contents: TextAssetContents<'a>,
        verification_set: &mut Vec<InBandContent<'a>>,
    ) {
        let bytes = contents.bytes.unwrap_or_default();
        if let Some(content) = signed_content(bytes) {
            verification_set.push(InBandContent::new(self.text_asset(), content));
        }

        // Mechanical base-call adaptation: the immutable object stream owns
        // the record, so retaining its borrowed fields replaces unique_ptr.
        debug_assert!(self.content.is_none());
        self.content = Some(contents);
    }
}

impl<'a> InBandContent<'a> {
    fn new(text_asset: &'a RuntimeObject, bytes: &'a [u8]) -> Self {
        Self { text_asset, bytes }
    }
}

impl<'a> TextAssetImporter<'a> {
    fn resolve(
        self,
        verification_set: &mut Vec<InBandContent<'a>>,
        verified_text_assets: &mut BTreeSet<u32>,
    ) {
        // FileAssetImporter::resolve is infallible at this pin. Loader/decode
        // dispatch remains the separately approved Rust-native asset path.
        let Some(signature) = self
            .content
            .and_then(|content| content.signature)
            .filter(|signature| !signature.is_empty())
        else {
            return;
        };

        let mut combined_bytecode = Vec::new();
        for in_band in verification_set.iter() {
            combined_bytecode.extend_from_slice(in_band.bytes);
        }

        let Ok(signature): Result<[u8; libhydrogen::sign::BYTES], _> = signature.try_into() else {
            return;
        };
        let signature = libhydrogen::sign::Signature::from(signature);
        let public_key = libhydrogen::sign::PublicKey::from(SCRIPT_VERIFICATION_PUBLIC_KEY);
        let context = libhydrogen::sign::Context::from("RiveCode");
        let is_verified =
            libhydrogen::sign::verify(&signature, &combined_bytecode, &context, &public_key)
                .is_ok();

        for in_band in verification_set.iter() {
            if is_verified {
                verified_text_assets.insert(in_band.text_asset.id);
            } else {
                verified_text_assets.remove(&in_band.text_asset.id);
            }
        }
        verification_set.clear();
    }
}

/// Replay the pinned shared FileAsset importer key and aggregate verification
/// set. This is the existing immutable two-pass import adaptation: verification
/// is projected onto catalog entries instead of mutating schema objects.
pub(crate) fn verified_text_asset_ids(file: &RuntimeFile) -> BTreeSet<u32> {
    let mut current = None::<TextAssetImporter<'_>>;
    let mut verification_set = Vec::<InBandContent<'_>>::new();
    let mut verified_text_assets = BTreeSet::new();

    for record in file.imported_file_asset_records() {
        match record {
            ImportedFileAssetRecord::Asset {
                asset,
                creates_importer: true,
            } => {
                let next = matches!(asset.type_name, "ScriptAsset" | "ShaderAsset")
                    .then(|| TextAssetImporter::new(asset));
                if let Some(previous) = current.take() {
                    previous.resolve(&mut verification_set, &mut verified_text_assets);
                }
                current = next;
            }
            ImportedFileAssetRecord::Asset {
                creates_importer: false,
                ..
            } => {}
            ImportedFileAssetRecord::Contents { bytes, signature } => {
                if let Some(importer) = current.as_mut() {
                    importer.on_file_asset_contents(
                        TextAssetContents { bytes, signature },
                        &mut verification_set,
                    );
                }
            }
        }
    }

    if let Some(importer) = current {
        importer.resolve(&mut verification_set, &mut verified_text_assets);
    }
    verified_text_assets
}

fn signed_content(bytes: &[u8]) -> Option<&[u8]> {
    let (&flags, _) = bytes.split_first()?;
    let content_offset = if flags & 0x80 == 0 {
        1
    } else {
        1 + libhydrogen::sign::BYTES
    };
    bytes.get(content_offset..)
}

pub(super) fn dispatch_imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.name == "TextAsset" {
        return Some(
            imports_successfully(object, definition, context)
                .expect("TextAsset is owned by TextAssetImporter"),
        );
    }
    None
}

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    (definition.name == "TextAsset").then(|| context.latest(ImportStackKey::Backboard))
}
