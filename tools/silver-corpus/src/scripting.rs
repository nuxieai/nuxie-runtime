//! Silver fixtures use the same File-owned registration and occurrence lifecycle
//! as the native runtime. No harness-side script graph or manual hydration.
use anyhow::{Context, Result};
use nuxie_runtime::source::lua::scripting_vm::RuntimeScriptingVmHandle;
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};
use nuxie_scripting::vm::{ScriptExecutionLimits, ScriptVm};

pub(crate) fn import_file(
    bytes: &[u8],
    factory: RuntimeFactoryHandle,
) -> Result<RuntimeFileHandle> {
    let vm = ScriptVm::new_with_execution_limits(ScriptExecutionLimits::default())
        .map_err(|error| anyhow::anyhow!(error.to_string()))
        .context("create native scripting VM")?;
    // Native TextAssetImporter verifies the original fixture envelope; File
    // registers the actual ScriptAsset programs before constructing instances.
    File::import(
        bytes,
        factory,
        None,
        None,
        Some(RuntimeScriptingVmHandle::new(Box::new(vm))),
    )
    .context("native File import")
}
