//! Product-owned scripting policy shared by Nuxie editor and SDK hosts.
//!
//! The baseline VM executes imported Rive bytecode. This crate injects the
//! private `require("nuxie")` module, records its effects, enforces product
//! effect quotas, and mints exact-artifact execution capabilities.

mod host_commands;
mod script_import;

pub use host_commands::{HostCommand, HostCycleCheckpoint, HostEffectCheckpoint, HostValue};
pub use nuxie::ScriptExecutionCapability;
pub use script_import::{ScriptAuthenticationError, ScriptImportCapability};

/// Product package grammar, verification, and canonical writer vocabulary.
pub mod nux {
    pub use nux_container::*;
}

use nuxie::{ScriptHostExtension, ScriptHostExtensionInstance};
use nuxie_scripting::vm::{Result, ScriptVm};
use std::sync::Arc;

/// Installs the private Nuxie module into a baseline VM on explicit product import.
#[derive(Debug)]
pub struct NuxieScriptHostExtension;

/// Mint product execution authority for bytes known to be locally authored.
///
/// # Safety
///
/// `artifact_bytes` must originate inside the trusted product authoring
/// process, never from an unauthenticated network, package, or cache source.
pub unsafe fn execution_capability_for_locally_authored_artifact(
    artifact_bytes: &[u8],
) -> std::result::Result<ScriptExecutionCapability, ScriptAuthenticationError> {
    u64::try_from(artifact_bytes.len())
        .map_err(|_| ScriptAuthenticationError::ArtifactSizeMismatch)?;
    // SAFETY: upheld by this function's caller contract.
    unsafe {
        ScriptExecutionCapability::for_verified_artifact_unchecked(
            artifact_bytes,
            Arc::new(NuxieScriptHostExtension),
        )
    }
    .map_err(|_| ScriptAuthenticationError::ArtifactSizeMismatch)
}

/// Mint product execution authority for a locally assembled decoded runtime.
///
/// # Safety
///
/// The runtime must be assembled inside the trusted product authoring process
/// and must not contain unverified externally supplied script bytecode.
pub unsafe fn execution_capability_for_locally_authored_runtime() -> ScriptExecutionCapability {
    // SAFETY: upheld by this function's caller contract.
    unsafe {
        ScriptExecutionCapability::for_locally_authored_runtime_unchecked(Arc::new(
            NuxieScriptHostExtension,
        ))
    }
}

impl ScriptHostExtension for NuxieScriptHostExtension {
    fn install(
        &self,
        vm: &ScriptVm,
    ) -> std::result::Result<Box<dyn ScriptHostExtensionInstance>, nuxie::ScriptError> {
        NuxieScriptHost::install(vm)
            .map(|host| Box::new(host) as Box<dyn ScriptHostExtensionInstance>)
            .map_err(|error| nuxie::ScriptError::new(error.to_string()))
    }
}

/// Per-VM adapter for the private Nuxie module and its ordered effect queue.
#[derive(Clone)]
pub struct NuxieScriptHost {
    queue: host_commands::HostCommandQueue,
}

impl std::fmt::Debug for NuxieScriptHost {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("NuxieScriptHost")
    }
}

impl ScriptHostExtensionInstance for NuxieScriptHost {
    fn effects_type_id(&self) -> std::any::TypeId {
        std::any::TypeId::of::<Vec<HostCommand>>()
    }

    fn begin_cycle(&self) -> Box<dyn std::any::Any> {
        Box::new(self.begin_cycle())
    }

    fn rollback_cycle(
        &self,
        checkpoint: Box<dyn std::any::Any>,
    ) -> std::result::Result<(), nuxie::ScriptError> {
        let checkpoint = checkpoint.downcast::<HostCycleCheckpoint>().map_err(|_| {
            nuxie::ScriptError::new("script host cycle checkpoint belongs to another extension")
        })?;
        self.rollback_cycle(*checkpoint);
        Ok(())
    }

    fn checkpoint_effects(&self) -> Box<dyn std::any::Any> {
        Box::new(self.checkpoint_effects())
    }

    fn rollback_effects(
        &self,
        checkpoint: Box<dyn std::any::Any>,
    ) -> std::result::Result<(), nuxie::ScriptError> {
        let checkpoint = checkpoint.downcast::<HostEffectCheckpoint>().map_err(|_| {
            nuxie::ScriptError::new("script host effect checkpoint belongs to another extension")
        })?;
        self.rollback_effects(*checkpoint);
        Ok(())
    }

    fn drain_effects(&self) -> nuxie::ScriptHostEffects {
        nuxie::ScriptHostEffects::new(self.drain())
    }
}

impl NuxieScriptHost {
    /// Install the product module into an already booted baseline VM.
    pub fn install(vm: &ScriptVm) -> Result<Self> {
        let queue = host_commands::HostCommandQueue::new(vm.resource_guard());
        let module = host_commands::nuxie_module(vm.lua(), queue.clone())?;
        vm.register_host_module("nuxie", module)?;
        Ok(Self { queue })
    }

    pub fn begin_cycle(&self) -> HostCycleCheckpoint {
        self.queue.begin_cycle()
    }

    pub fn rollback_cycle(&self, checkpoint: HostCycleCheckpoint) {
        self.queue.rollback(checkpoint);
    }

    pub fn checkpoint_effects(&self) -> HostEffectCheckpoint {
        self.queue.checkpoint_effects()
    }

    pub fn rollback_effects(&self, checkpoint: HostEffectCheckpoint) {
        self.queue.rollback_effects(checkpoint);
    }

    pub fn drain(&self) -> Vec<HostCommand> {
        self.queue.drain()
    }
}
