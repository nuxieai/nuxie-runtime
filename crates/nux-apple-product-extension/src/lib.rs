//! Product-specific additions to the otherwise product-neutral Apple C runtime.

#[used]
static BUILD_PROVENANCE: &str = env!("NUX_APPLE_PRODUCT_EXTENSION_BUILD_PROVENANCE");

/// Import caller-authenticated Nuxie scene bytes after enabling the authored-
/// data converter format used by published product experiences.
///
/// This upper-leaf entrypoint is deliberately product-named. Baseline
/// [`nux_capi::nux_file_import_configured`] remains product-neutral and never
/// depends upward on Nuxie's authored-data rules.
///
/// # Safety
///
/// The pointers and lengths must satisfy the same contract as
/// [`nux_capi::nux_file_import_configured`].
#[cfg(all(feature = "apple-runtime", any(target_os = "ios", target_os = "macos")))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_product_file_import_configured(
    bytes: *const u8,
    len: usize,
    config: *const nux_capi::NuxFileImportConfig,
    out_file: *mut *mut nux_capi::NuxFile,
    out_result: *mut *mut nux_capi::NuxCapiResult,
) -> nux_capi::NuxStatus {
    nuxie_project_data::install_runtime_adapter();
    unsafe { nux_capi::nux_file_import_configured(bytes, len, config, out_file, out_result) }
}
