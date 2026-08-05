//! Shared Nuxie product-host contract.
//!
//! This package is the migration owner for product execution policy shared by
//! the editor and SDK hosts. Its first interface is the existing Flow protocol,
//! re-exported without wrapping so callers preserve type identity and behavior.
//! UNIV-1630 moves the implementation here; until then `nuxie::flow_session`
//! remains the temporary compatibility path.

pub use nuxie::flow_session::*;

#[cfg(test)]
mod tests {
    use super::FlowSessionConfig;

    #[test]
    fn flow_contract_keeps_legacy_type_identity() {
        fn accepts_legacy(value: nuxie::flow_session::FlowSessionConfig) -> FlowSessionConfig {
            value
        }

        let config = FlowSessionConfig::default();
        assert!(accepts_legacy(config).artboard_name.is_none());
    }
}
