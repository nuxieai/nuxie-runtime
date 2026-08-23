//! Complete dependency-authority translation of
//! `renderer/src/webgpu/wagyu-port/src/library_webgpu_wagyu_stubs.js`.
//!
//! These declarations require the Emscripten host to provide every Wagyu
//! extension. They are intentionally not Rust fallback implementations.

pub(crate) const PINNED_SOURCE: &str = include_str!(
    "source/renderer_src_webgpu_wagyu-port_src_library_webgpu_wagyu_stubs.js"
);
pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = 82;
pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = 4_088;
pub(crate) const LIBRARY_NAME: &str = "LibraryWebGPUExtensions";
pub(crate) const REGISTRATION_OPERATION: &str = "addToLibrary(LibraryWebGPUExtensions)";

pub(crate) const UNDEFINED_HOST_SYMBOLS: &[&str] = &[
    "wgpuWagyuAdapterGetBackend",
    "wgpuWagyuAdapterGetExtensions",
    "wgpuWagyuAdapterGetGraphicsReport",
    "wgpuWagyuAdapterGetName",
    "wgpuWagyuAdapterRequestDeviceSync",
    "wgpuWagyuCommandEncoderBlit",
    "wgpuWagyuCommandEncoderExecuteCallback",
    "wgpuWagyuCommandEncoderGenerateMipmap",
    "wgpuWagyuComputePassEncoderExecuteCallback",
    "wgpuWagyuDeviceClearPipelineBinaryCache",
    "wgpuWagyuDeviceEnableImaginationWorkarounds",
    "wgpuWagyuDeviceFlush",
    "wgpuWagyuDeviceGetExtensions",
    "wgpuWagyuDeviceImportExternalTexture",
    "wgpuWagyuDeviceIntrospectShaderCode",
    "wgpuWagyuDevicePipelineBinaryCacheStatistics",
    "wgpuWagyuDevicePopulatePipelineBinaryCache",
    "wgpuWagyuDeviceSetPipelineBinaryCallback",
    "wgpuWagyuDeviceSetPipelineBinaryErrorCallback",
    "wgpuWagyuExternalTextureAddRef",
    "wgpuWagyuExternalTextureGetInfo",
    "wgpuWagyuExternalTextureRelease",
    "wgpuWagyuExternalTextureSetLabel",
    "wgpuWagyuInstanceEnableImaginationWorkarounds",
    "wgpuWagyuInstanceGetApiVersion",
    "wgpuWagyuInstanceGetBackend",
    "wgpuWagyuInstanceGetExposedWgslFeatures",
    "wgpuWagyuInstanceGetScreenDirectFormat",
    "wgpuWagyuInstanceGetScreenIndirectFormat",
    "wgpuWagyuInstanceGetSync",
    "wgpuWagyuInstanceRequestAdapterSync",
    "wgpuWagyuInstanceSetCommandBufferLimit",
    "wgpuWagyuInstanceSetExposedWgslFeatures",
    "wgpuWagyuInstanceSetImmediate",
    "wgpuWagyuInstanceSetRunBarriersOnIncoherent",
    "wgpuWagyuInstanceSetStagingBufferCacheSize",
    "wgpuWagyuInstanceSetSync",
    "wgpuWagyuQueueCopyExternalImageToTexture",
    "wgpuWagyuRenderBundleEncoderClearColorAttachments",
    "wgpuWagyuRenderBundleEncoderClearDepthAttachment",
    "wgpuWagyuRenderBundleEncoderClearPixelLocalStorage",
    "wgpuWagyuRenderBundleEncoderClearStencilAttachment",
    "wgpuWagyuRenderBundleEncoderExecuteCallback",
    "wgpuWagyuRenderBundleEncoderSetScissorRect",
    "wgpuWagyuRenderBundleEncoderSetScissorRectIndirect",
    "wgpuWagyuRenderBundleEncoderSetViewport",
    "wgpuWagyuRenderBundleEncoderSetViewportWithDepthIndirect",
    "wgpuWagyuRenderBundleEncoderSetViewportWithoutDepthIndirect",
    "wgpuWagyuRenderPassEncoderClearColorAttachments",
    "wgpuWagyuRenderPassEncoderClearDepthAttachment",
    "wgpuWagyuRenderPassEncoderClearPixelLocalStorage",
    "wgpuWagyuRenderPassEncoderClearStencilAttachment",
    "wgpuWagyuRenderPassEncoderExecuteBundle",
    "wgpuWagyuRenderPassEncoderExecuteCallback",
    "wgpuWagyuRenderPassEncoderSetShaderPixelLocalStorageEnabled",
    "wgpuWagyuShaderEntryPointArrayFreeMembers",
    "wgpuWagyuShaderModuleDestroy",
    "wgpuWagyuShaderModuleEntryPoints",
    "wgpuWagyuShaderModuleIntrospect",
    "wgpuWagyuStringArrayFreeMembers",
    "wgpuWagyuSurfaceDestroy",
    "wgpuWagyuSurfaceGetCurrentDepthStencilTexture",
    "wgpuWagyuSurfaceGetHeight",
    "wgpuWagyuSurfaceGetWidth",
    "wgpuWagyuSurfaceGetX",
    "wgpuWagyuSurfaceGetY",
    "wgpuWagyuSurfacePresent",
    "wgpuWagyuSurfaceSetHeight",
    "wgpuWagyuSurfaceSetWidth",
    "wgpuWagyuSurfaceSetX",
    "wgpuWagyuSurfaceSetY",
    "wgpuWagyuTextureIsSwapchain",
    "wgpuWagyuTextureReadPixels",
    "wgpuWagyuWGSLFeatureTypeArrayFreeMembers",
];

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
const _: [(); 74] = [(); UNDEFINED_HOST_SYMBOLS.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn complete_wagyu_extension_symbol_denominator_is_frozen() {
        assert_eq!(UNDEFINED_HOST_SYMBOLS.len(), 74);
        assert_eq!(UNDEFINED_HOST_SYMBOLS.iter().copied().collect::<BTreeSet<_>>().len(), 74);
        assert_eq!(UNDEFINED_HOST_SYMBOLS.first(), Some(&"wgpuWagyuAdapterGetBackend"));
        assert_eq!(
            UNDEFINED_HOST_SYMBOLS.last(),
            Some(&"wgpuWagyuWGSLFeatureTypeArrayFreeMembers")
        );
        for symbol in UNDEFINED_HOST_SYMBOLS {
            assert!(PINNED_SOURCE.contains(&format!("{symbol}: undefined")));
        }
    }

    #[test]
    fn source_registers_one_undefined_extension_library() {
        assert!(PINNED_SOURCE.contains("const LibraryWebGPUExtensions = {"));
        assert!(PINNED_SOURCE.contains("addToLibrary(LibraryWebGPUExtensions);"));
        assert_eq!(LIBRARY_NAME, "LibraryWebGPUExtensions");
        assert_eq!(
            REGISTRATION_OPERATION,
            "addToLibrary(LibraryWebGPUExtensions)"
        );
    }

    #[test]
    fn pls_and_external_texture_extensions_cannot_fall_back() {
        for symbol in [
            "wgpuWagyuRenderPassEncoderClearPixelLocalStorage",
            "wgpuWagyuRenderPassEncoderSetShaderPixelLocalStorageEnabled",
            "wgpuWagyuDeviceImportExternalTexture",
            "wgpuWagyuTextureReadPixels",
        ] {
            assert!(UNDEFINED_HOST_SYMBOLS.contains(&symbol));
        }
    }
}
