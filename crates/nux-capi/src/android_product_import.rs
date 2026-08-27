//! Narrow authored-data seam for the Android product distribution.
//!
//! Android ships `nux-capi` directly and calls the portable configured-import
//! symbol, whereas the other platform archive has an upper-leaf product
//! extension with a distinct entrypoint. Keeping the adapter call in this
//! feature-gated module makes that distribution asymmetry explicit without
//! installing product behavior in baseline, the other platform baseline,
//! renderer-only Android, or portable scripting builds.

pub(crate) fn prepare_configured_import_runtime() {
    nuxie_project_data::install_runtime_adapter();
}
