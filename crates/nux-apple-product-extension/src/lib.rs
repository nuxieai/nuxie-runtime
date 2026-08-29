//! Product-specific additions to the otherwise product-neutral Apple C runtime.

#[used]
static BUILD_PROVENANCE: &str = env!("NUX_APPLE_PRODUCT_EXTENSION_BUILD_PROVENANCE");

/// Import caller-authenticated Nuxie scene bytes after enabling the authored-
/// data converter format and trusted native-shader authority used by published
/// product experiences.
///
/// This upper-leaf entrypoint is deliberately product-named. Baseline
/// [`nux_capi::nux_file_import_metal`] remains product-neutral and never
/// depends upward on Nuxie's authored-data rules.
///
/// # Safety
///
/// The pointers and lengths must satisfy the same contract as
/// [`nux_capi::nux_file_import_metal`]. The caller must also establish
/// that native shader payloads were emitted by Nuxie's trusted exporter; a
/// package signature over arbitrary shader source is not sufficient.
#[cfg(all(feature = "apple-runtime", any(target_os = "ios", target_os = "macos")))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_product_file_import_configured(
    renderer: *mut nux_capi::NuxRenderer,
    bytes: *const u8,
    len: usize,
    config: *const nux_capi::NuxFileImportConfig,
    out_file: *mut *mut nux_capi::NuxFile,
    out_result: *mut *mut nux_capi::NuxCapiResult,
) -> nux_capi::NuxStatus {
    unsafe {
        nux_capi::nux_file_import_metal_with_trusted_native_shaders_and_program_adapter(
            renderer,
            bytes,
            len,
            config,
            out_file,
            out_result,
            nuxie_project_data_scripting::ProjectDataScriptProgramAdapter::shared(),
        )
    }
}
