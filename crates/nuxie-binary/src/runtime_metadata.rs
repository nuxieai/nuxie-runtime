//! Structural decoding for the native runtime's host metadata boundary.
//! No importer, dependency graph, lifecycle, or data-binding behavior runs here.

use anyhow::{Context, Result, ensure};

use crate::{
    RuntimeHeader, RuntimeImportStatus, RuntimeObject, RuntimePropertyBudget,
    SUPPORTED_MAJOR_VERSION, core::binary_reader::BinaryReader, read_runtime_object,
};

#[derive(Debug, Clone)]
pub struct DecodedRuntimeMetadata {
    pub header: RuntimeHeader,
    pub objects: Vec<Option<RuntimeObject>>,
    decoded_property_count: usize,
}

impl DecodedRuntimeMetadata {
    /// Includes header entries and every decoded wire property, even when a
    /// duplicate, skipped, or unknown record did not retain that property.
    pub fn decoded_property_count(&self) -> usize {
        self.decoded_property_count
    }

    /// Attach outcomes observed from the real source importer. The metadata
    /// parser does not decide which objects or assets the runtime accepts.
    pub fn into_runtime_descriptor(
        self,
        import_statuses: Vec<RuntimeImportStatus>,
        file_asset_object_ids: Vec<usize>,
    ) -> Result<crate::RuntimeFile> {
        ensure!(
            import_statuses.len() == self.objects.len(),
            "native import outcomes must correspond to every decoded record"
        );
        ensure!(
            file_asset_object_ids.iter().all(|&id| {
                self.objects.get(id).is_some_and(Option::is_some)
                    && import_statuses.get(id) == Some(&RuntimeImportStatus::Imported)
            }),
            "native file assets must identify imported source records"
        );
        Ok(crate::RuntimeFile {
            header: self.header,
            objects: self.objects,
            import_statuses,
            file_asset_object_ids,
        })
    }
}

pub fn read_runtime_metadata(
    bytes: &[u8],
    max_runtime_objects: Option<usize>,
    max_runtime_properties: Option<usize>,
) -> Result<DecodedRuntimeMetadata> {
    let mut reader = BinaryReader::new(bytes);
    let mut budget = RuntimePropertyBudget::new(max_runtime_properties);
    let header = RuntimeHeader::read(&mut reader, &mut budget)?;
    ensure!(
        header.major_version == SUPPORTED_MAJOR_VERSION,
        "unsupported major version {}.{}",
        header.major_version,
        header.minor_version
    );
    let mut objects = Vec::new();
    while !reader.reached_end() {
        if let Some(maximum) = max_runtime_objects {
            ensure!(
                objects.len() < maximum,
                "Rive file contains more than {maximum} runtime objects"
            );
        }
        let id = u32::try_from(objects.len()).context("runtime object id does not fit in u32")?;
        objects.push(
            read_runtime_object(&mut reader, &header, id, &mut budget)
                .with_context(|| format!("reading object {id}"))?,
        );
    }
    Ok(DecodedRuntimeMetadata {
        header,
        objects,
        decoded_property_count: budget.consumed,
    })
}

/// Continue the same host allocation budget when the actual source importer
/// identifies a ManifestAsset payload. This does not infer asset ownership or
/// run the superseded binary import lifecycle. The host calls it before the
/// native manifest decoder allocates entries; malformed payloads remain soft
/// failures, while a configured allocation-limit violation is an error.
pub fn validate_manifest_payload_budget(
    bytes: &[u8],
    maximum: Option<usize>,
    consumed: &mut usize,
) -> Result<()> {
    if maximum.is_none() {
        return Ok(());
    }
    let mut budget = RuntimePropertyBudget {
        maximum,
        consumed: *consumed,
    };
    let result =
        crate::assets::manifest_asset::validate_cpp_manifest_asset_with_budget(bytes, &mut budget);
    *consumed = budget.consumed;
    result
}
