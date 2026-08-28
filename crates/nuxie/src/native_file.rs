//! Host import boundary for the sole translated runtime.
//!
//! Binary descriptors are inspection/preflight data, never an execution graph.
//! Byte imports always pass the original bytes to the native importer. Encoding
//! below is only for the existing explicitly decoded/local-authoring API.

use anyhow::Result;
pub(crate) use nuxie_binary::encode_runtime_file as encode_decoded_runtime;
use nuxie_binary::{RuntimeImportDropReason, RuntimeImportStatus, read_runtime_metadata};
use nuxie_render_api::Factory;
use nuxie_runtime::mechanical_port::source::{
    factory::RuntimeFactoryHandle,
    file::{File as SourceFile, ImportResult, RuntimeFileHandle, RuntimeImportRecord},
    file_asset_loader::FileAssetLoaderRef,
    lua::scripting_vm::RuntimeScriptingVmHandle,
    status_code::StatusCode,
};

pub(crate) struct ImportedFile {
    pub native: RuntimeFileHandle,
    pub descriptor: std::sync::Arc<nuxie_binary::RuntimeFile>,
}

pub(crate) fn import_with_metadata(
    bytes: &[u8],
    factory: &mut dyn Factory,
    loader: Option<FileAssetLoaderRef>,
    vm: Option<RuntimeScriptingVmHandle>,
    max_objects: Option<usize>,
    max_properties: Option<usize>,
) -> Result<ImportedFile> {
    let metadata = read_runtime_metadata(bytes, max_objects, max_properties)?;
    let factory = retained_factory(factory)?;
    let mut records = Vec::new();
    let mut result = ImportResult::Malformed;
    let native = SourceFile::import_with_records(
        bytes,
        factory,
        Some(&mut result),
        loader,
        vm,
        &mut records,
    )
    .ok_or_else(|| anyhow::anyhow!("Rive runtime import failed: {result:?}"))?;
    let statuses = records
        .iter()
        .map(|record| match record {
            RuntimeImportRecord::NullObject => RuntimeImportStatus::NullObject,
            RuntimeImportRecord::Imported(_) => RuntimeImportStatus::Imported,
            RuntimeImportRecord::Dropped(code) => RuntimeImportStatus::Dropped {
                reason: match code {
                    StatusCode::MissingObject => RuntimeImportDropReason::MissingObject,
                    _ => RuntimeImportDropReason::InvalidObject,
                },
            },
        })
        .collect();
    let asset_ids = native.with_file(|file| {
        file.assets()
            .iter()
            .map(|asset| {
                records.iter().position(|record| {
                matches!(record, RuntimeImportRecord::Imported(object) if object == asset)
            }).ok_or_else(|| anyhow::anyhow!("native asset has no imported source record"))
            })
            .collect::<Result<Vec<_>>>()
    })?;
    let descriptor = metadata.into_runtime_descriptor(statuses, asset_ids)?;
    Ok(ImportedFile {
        native,
        descriptor: std::sync::Arc::new(descriptor),
    })
}

pub(crate) fn import(
    bytes: &[u8],
    factory: &mut dyn Factory,
    loader: Option<FileAssetLoaderRef>,
    vm: Option<RuntimeScriptingVmHandle>,
) -> Result<RuntimeFileHandle> {
    let factory = retained_factory(factory)?;
    let mut result = ImportResult::Malformed;
    SourceFile::import(bytes, factory, Some(&mut result), loader, vm)
        .ok_or_else(|| anyhow::anyhow!("Rive runtime import failed: {result:?}"))
}

fn retained_factory(factory: &mut dyn Factory) -> Result<RuntimeFactoryHandle> {
    RuntimeFactoryHandle::from_factory(factory).ok_or_else(|| {
        anyhow::anyhow!(
            "runtime import requires a retained renderer factory; wrap the factory in PersistentFactory"
        )
    })
}
