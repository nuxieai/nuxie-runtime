//! Shared Nuxie product-host contract.
//!
//! This package owns product execution policy shared by the editor and SDK
//! hosts. Its first interface is the renderer-neutral Flow protocol.

pub mod flow_session;

#[cfg(feature = "scripting")]
pub mod scripting {
    pub use nuxie_product_scripting::*;

    use nuxie::{File, ScriptExecutionLimits, host_interfaces::RuntimeFile};

    /// Import bytes authored inside the trusted editor/product process.
    /// # Safety
    ///
    /// `bytes` must be produced inside the trusted local authoring process.
    pub unsafe fn import_locally_authored_file(
        bytes: &[u8],
        limits: ScriptExecutionLimits,
    ) -> anyhow::Result<File> {
        // SAFETY: upheld by this function's caller contract.
        let capability = unsafe { execution_capability_for_locally_authored_artifact(bytes)? };
        File::import_with_execution_capability(bytes, capability, limits)
    }

    /// Adopt a decoded runtime assembled inside the trusted editor process.
    /// # Safety
    ///
    /// `runtime` must be assembled locally and contain no unverified script bytes.
    pub unsafe fn file_from_locally_authored_runtime(
        runtime: RuntimeFile,
        limits: ScriptExecutionLimits,
    ) -> anyhow::Result<File> {
        // SAFETY: upheld by this function's caller contract.
        let capability = unsafe { execution_capability_for_locally_authored_runtime() };
        File::from_runtime_with_execution_capability(runtime, capability, limits)
    }

    /// Import signed remote content after binding execution to its exact scene
    /// bytes. Visual-only authority keeps the ordinary baseline import path.
    pub fn import_authenticated_file(
        bytes: &[u8],
        capability: ScriptImportCapability,
        limits: ScriptExecutionLimits,
    ) -> anyhow::Result<File> {
        match capability.execution_capability_for(bytes)? {
            Some(capability) => File::import_with_execution_capability(bytes, capability, limits),
            None => File::import(bytes),
        }
    }
}

/// Temporary root-level compatibility export for callers that adopted the
/// initial crate seam as `nuxie_product::*`. UNIV-1634 removes this export once
/// every consumer imports `nuxie_product::flow_session` explicitly.
#[doc(hidden)]
pub use flow_session::*;

#[cfg(test)]
mod tests {
    use super::FlowSessionConfig;

    #[test]
    fn root_compatibility_export_keeps_module_type_identity() {
        fn accepts_module(value: super::flow_session::FlowSessionConfig) -> FlowSessionConfig {
            value
        }

        let config = FlowSessionConfig::default();
        assert!(accepts_module(config).artboard_name.is_none());
    }
}
