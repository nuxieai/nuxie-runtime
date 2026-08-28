//! Host import boundary for the sole translated runtime.
//!
//! Binary descriptors are inspection/preflight data, never an execution graph.
//! Byte imports always pass the original bytes to the native importer. Encoding
//! below is only for the existing explicitly decoded/local-authoring API.

use anyhow::Result;
pub(crate) use nuxie_binary::encode_runtime_file as encode_decoded_runtime;
use nuxie_render_api::Factory;
use nuxie_runtime::mechanical_port::source::{
    factory::RuntimeFactoryHandle,
    file::{File as SourceFile, ImportResult, RuntimeFileHandle},
    file_asset_loader::FileAssetLoaderRef,
    lua::scripting_vm::RuntimeScriptingVmHandle,
};

pub(crate) fn import(
    bytes: &[u8],
    factory: &mut dyn Factory,
    loader: Option<FileAssetLoaderRef>,
    vm: Option<RuntimeScriptingVmHandle>,
) -> Result<RuntimeFileHandle> {
    let factory = RuntimeFactoryHandle::from_factory(factory).ok_or_else(|| {
        anyhow::anyhow!(
            "runtime import requires a retained renderer factory; wrap the factory in PersistentFactory"
        )
    })?;
    let mut result = ImportResult::Malformed;
    SourceFile::import(bytes, factory, Some(&mut result), loader, vm)
        .ok_or_else(|| anyhow::anyhow!("Rive runtime import failed: {result:?}"))
}
