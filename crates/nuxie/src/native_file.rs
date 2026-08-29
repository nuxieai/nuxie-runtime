//! Optional bounded host admission around the sole native source importer.

use std::rc::Rc;

use anyhow::Result;
use nuxie_render_api::Factory;
use nuxie_runtime::mechanical_port::source::{
    factory::RuntimeFactoryHandle,
    file::{File as SourceFile, ImportResult, RuntimeFileHandle},
    file_asset_loader::FileAssetLoaderRef,
    lua::scripting_vm::RuntimeScriptingVmHandle,
};

use crate::import_limits::{FileImportLimits, NativeImportAdmission};

/// Import original Rive bytes with explicit host resource limits and a retained
/// renderer factory. Scripts are disabled; authorized scripting uses the
/// separate capability-bearing entry point. `File::import` exposes the ordinary
/// upstream policy-free import directly.
pub fn import_native(
    bytes: &[u8],
    factory: &mut dyn Factory,
    loader: Option<FileAssetLoaderRef>,
    limits: FileImportLimits,
) -> Result<RuntimeFileHandle> {
    import(bytes, factory, loader, None, limits)
}

pub(crate) fn import(
    bytes: &[u8],
    factory: &mut dyn Factory,
    loader: Option<FileAssetLoaderRef>,
    vm: Option<RuntimeScriptingVmHandle>,
    limits: FileImportLimits,
) -> Result<RuntimeFileHandle> {
    let factory = RuntimeFactoryHandle::from_factory(factory).ok_or_else(|| {
        anyhow::anyhow!(
            "runtime import requires a retained renderer factory; wrap the factory in PersistentFactory"
        )
    })?;
    let admission = Rc::new(NativeImportAdmission::preflight(bytes, limits)?);
    let mut result = ImportResult::Malformed;
    let native = SourceFile::import_with_admission(
        bytes,
        factory,
        Some(&mut result),
        loader,
        vm,
        admission.clone(),
    );
    admission.finish()?;
    native.ok_or_else(|| anyhow::anyhow!("Rive runtime import failed: {result:?}"))
}
