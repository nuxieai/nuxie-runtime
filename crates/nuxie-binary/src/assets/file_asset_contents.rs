use crate::importers::{ImportContext, ImportStackKey};
use crate::*;

/// One dense, file-global FileAsset entry and the in-band contents imported
/// for it by a scripting-enabled FileAsset importer.
#[derive(Debug, Clone, Copy)]
pub struct RuntimeFileAssetContents<'a> {
    pub ordinal: usize,
    pub asset: &'a RuntimeObject,
    /// Whether the importer observed at least one `FileAssetContents` record,
    /// independent of whether the selected record carried a bytes property.
    pub has_contents_record: bool,
    /// Payload selected by the last imported contents record for this asset.
    pub contents: Option<&'a [u8]>,
    /// Signature selected from the same imported contents record.
    pub signature: Option<&'a [u8]>,
}

impl<'a> RuntimeFileAssetContents<'a> {
    pub fn bytes(&self) -> Option<&'a [u8]> {
        self.contents
    }

    pub fn signature(&self) -> Option<&'a [u8]> {
        self.signature
    }
}

#[derive(Debug, Clone, Copy)]
enum ImportedFileAssetRecord<'a> {
    Asset {
        asset: &'a RuntimeObject,
        creates_importer: bool,
    },
    Contents {
        bytes: Option<&'a [u8]>,
        signature: Option<&'a [u8]>,
    },
}

/// Pinned `FileAssetContents::import` owner after Core/Super dispatch has
/// established the current import stack. A missing latest FileAssetImporter
/// is `MissingObject`; ownership transfer and Super import are represented by
/// the imported object/status vectors retained by `RuntimeFile`.
pub(crate) fn imports_successfully(context: &ImportContext) -> bool {
    context.latest(ImportStackKey::FileAsset)
}

impl RuntimeFile {
    /// Dense FileAsset catalog with `FileAssetContents` associated by the
    /// scripting-enabled importer stack rather than by record adjacency.
    ///
    /// This is the extraction profile used by script-executing hosts. It scans
    /// the object stream once, tracks the latest imported FileAsset that
    /// creates an importer in a WITH_RIVE_SCRIPTING build, and attaches only
    /// imported `FileAssetContents` records to that entry.
    pub fn scripting_file_assets_with_contents(&self) -> Vec<RuntimeFileAssetContents<'_>> {
        let assets = self.file_assets();
        self.file_assets_with_contents(assets)
    }

    /// Every imported `FileAsset`, including importer-owning `ManifestAsset`
    /// records, with in-band contents associated by the scripting-enabled
    /// importer stack.
    ///
    /// Unlike [`Self::scripting_file_assets_with_contents`], this catalog is
    /// for validation and inspection rather than Rive's dense public asset
    /// ordinals. Consumers must identify entries by serialized `assetId`.
    pub fn imported_file_assets_with_contents(&self) -> Vec<RuntimeFileAssetContents<'_>> {
        self.imported_file_assets_with_contents_bounded(usize::MAX)
            .expect("usize::MAX cannot be exceeded by a Vec")
    }

    /// Bounded form of [`Self::imported_file_assets_with_contents`]. The scan
    /// stops before allocating an entry beyond `max_assets`.
    pub fn imported_file_assets_with_contents_bounded(
        &self,
        max_assets: usize,
    ) -> Option<Vec<RuntimeFileAssetContents<'_>>> {
        let mut assets = Vec::<RuntimeFileAssetContents<'_>>::new();
        let mut latest_ordinal = None;
        for record in self.imported_file_asset_records() {
            match record {
                ImportedFileAssetRecord::Asset {
                    asset,
                    creates_importer,
                } => {
                    if assets.len() == max_assets {
                        return None;
                    }
                    let ordinal = assets.len();
                    assets.push(RuntimeFileAssetContents {
                        ordinal,
                        asset,
                        has_contents_record: false,
                        contents: None,
                        signature: None,
                    });
                    if creates_importer {
                        latest_ordinal = Some(ordinal);
                    }
                }
                ImportedFileAssetRecord::Contents { bytes, signature } => {
                    if let Some(entry) = latest_ordinal.and_then(|ordinal| assets.get_mut(ordinal))
                    {
                        entry.has_contents_record = true;
                        entry.contents = bytes;
                        entry.signature = signature;
                    }
                }
            }
        }
        Some(assets)
    }

    /// Embedded bytes owned by one imported FileAsset under the
    /// scripting-enabled importer stack. Importer-owning assets excluded from
    /// the dense public catalog, including ManifestAsset, still delimit
    /// ownership and can never donate their contents to the preceding asset.
    pub fn imported_file_asset_contents(&self, asset_global_id: u32) -> Option<&[u8]> {
        let mut selected_asset_is_latest_importer = false;
        let mut selected_contents = None;
        for record in self.imported_file_asset_records() {
            match record {
                ImportedFileAssetRecord::Asset {
                    asset,
                    creates_importer: true,
                } => {
                    selected_asset_is_latest_importer = asset.id == asset_global_id;
                    if selected_asset_is_latest_importer {
                        selected_contents = None;
                    }
                }
                ImportedFileAssetRecord::Asset {
                    creates_importer: false,
                    ..
                } => {}
                ImportedFileAssetRecord::Contents { bytes, .. }
                    if selected_asset_is_latest_importer =>
                {
                    selected_contents = bytes;
                }
                ImportedFileAssetRecord::Contents { .. } => {}
            }
        }
        selected_contents
    }

    fn imported_file_asset_records(&self) -> impl Iterator<Item = ImportedFileAssetRecord<'_>> {
        self.objects
            .iter()
            .enumerate()
            .filter_map(|(index, object)| {
                if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                    return None;
                }
                let object = object.as_ref()?;
                let definition = definition_by_type_key(object.type_key)?;
                if definition.is_a("FileAsset") {
                    return Some(ImportedFileAssetRecord::Asset {
                        asset: object,
                        creates_importer: file_asset_creates_importer(object.type_name, true),
                    });
                }
                (object.type_name == "FileAssetContents").then(|| {
                    ImportedFileAssetRecord::Contents {
                        bytes: object.bytes_property("bytes"),
                        signature: object.bytes_property("signature"),
                    }
                })
            })
    }

    fn file_assets_with_contents<'a>(
        &'a self,
        assets: Vec<&'a RuntimeObject>,
    ) -> Vec<RuntimeFileAssetContents<'a>> {
        let ordinals_by_global = assets
            .iter()
            .enumerate()
            .map(|(ordinal, asset)| (asset.id, ordinal))
            .collect::<BTreeMap<_, _>>();
        let mut contents = vec![None; assets.len()];
        let mut signatures = vec![None; assets.len()];
        let mut has_contents_record = vec![false; assets.len()];
        let mut latest_ordinal = None;

        for record in self.imported_file_asset_records() {
            match record {
                ImportedFileAssetRecord::Asset {
                    asset,
                    creates_importer: true,
                } => {
                    // Importer-owning FileAsset kinds such as ManifestAsset are
                    // intentionally absent from the public dense catalog. They
                    // still delimit contents ownership, so reset the candidate
                    // even when there is no ordinal to publish.
                    latest_ordinal = ordinals_by_global.get(&asset.id).copied();
                }
                ImportedFileAssetRecord::Asset {
                    creates_importer: false,
                    ..
                } => {}
                ImportedFileAssetRecord::Contents { bytes, signature } => {
                    if let Some(ordinal) = latest_ordinal
                        && let Some(slot) = contents.get_mut(ordinal)
                        && let Some(signature_slot) = signatures.get_mut(ordinal)
                        && let Some(has_record) = has_contents_record.get_mut(ordinal)
                    {
                        *has_record = true;
                        *slot = bytes;
                        *signature_slot = signature;
                    }
                }
            }
        }

        assets
            .into_iter()
            .enumerate()
            .map(|(ordinal, asset)| RuntimeFileAssetContents {
                ordinal,
                asset,
                has_contents_record: has_contents_record.get(ordinal).copied().unwrap_or(false),
                contents: contents.get(ordinal).copied().flatten(),
                signature: signatures.get(ordinal).copied().flatten(),
            })
            .collect()
    }
}
