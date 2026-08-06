//! Shared Nuxie product-host contract.
//!
//! This package owns product execution policy shared by the editor and SDK
//! hosts. Its first interface is the renderer-neutral Flow protocol.

pub mod flow_session;

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
