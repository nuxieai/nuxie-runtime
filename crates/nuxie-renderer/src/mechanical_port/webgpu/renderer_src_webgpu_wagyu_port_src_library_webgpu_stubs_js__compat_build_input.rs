//! Complete dependency-authority translation of
//! `renderer/src/webgpu/wagyu-port/src/library_webgpu_stubs.js`.
//!
//! These are Emscripten link-symbol declarations, not fallback renderer
//! implementations. Every value is deliberately `undefined` in the source so
//! the host WebGPU library must provide the actual implementation.

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_webgpu_wagyu-port_src_library_webgpu_stubs.js");
pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = 207;
pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = 8_989;
pub(crate) const LIBRARY_NAME: &str = "LibraryWebGPU";
pub(crate) const REGISTRATION_OPERATION: &str = "addToLibrary(LibraryWebGPU)";

pub(crate) const UNDEFINED_HOST_SYMBOLS: &[&str] = &[
    "wgpuAdapterAddRef",
    "wgpuAdapterGetFeatures",
    "wgpuAdapterGetInfo",
    "wgpuAdapterGetLimits",
    "wgpuAdapterHasFeature",
    "wgpuAdapterInfoFreeMembers",
    "wgpuAdapterRelease",
    "wgpuAdapterRequestDevice",
    "wgpuBindGroupAddRef",
    "wgpuBindGroupLayoutAddRef",
    "wgpuBindGroupLayoutRelease",
    "wgpuBindGroupLayoutSetLabel",
    "wgpuBindGroupRelease",
    "wgpuBindGroupSetLabel",
    "wgpuBufferAddRef",
    "wgpuBufferDestroy",
    "wgpuBufferGetConstMappedRange",
    "wgpuBufferGetMapState",
    "wgpuBufferGetMappedRange",
    "wgpuBufferGetSize",
    "wgpuBufferGetUsage",
    "wgpuBufferMapAsync",
    "wgpuBufferReadMappedRange",
    "wgpuBufferRelease",
    "wgpuBufferSetLabel",
    "wgpuBufferUnmap",
    "wgpuBufferWriteMappedRange",
    "wgpuCommandBufferAddRef",
    "wgpuCommandBufferRelease",
    "wgpuCommandBufferSetLabel",
    "wgpuCommandEncoderAddRef",
    "wgpuCommandEncoderBeginComputePass",
    "wgpuCommandEncoderBeginRenderPass",
    "wgpuCommandEncoderClearBuffer",
    "wgpuCommandEncoderCopyBufferToBuffer",
    "wgpuCommandEncoderCopyBufferToTexture",
    "wgpuCommandEncoderCopyTextureToBuffer",
    "wgpuCommandEncoderCopyTextureToTexture",
    "wgpuCommandEncoderFinish",
    "wgpuCommandEncoderInsertDebugMarker",
    "wgpuCommandEncoderPopDebugGroup",
    "wgpuCommandEncoderPushDebugGroup",
    "wgpuCommandEncoderRelease",
    "wgpuCommandEncoderResolveQuerySet",
    "wgpuCommandEncoderSetLabel",
    "wgpuCommandEncoderWriteTimestamp",
    "wgpuComputePassEncoderAddRef",
    "wgpuComputePassEncoderDispatchWorkgroups",
    "wgpuComputePassEncoderDispatchWorkgroupsIndirect",
    "wgpuComputePassEncoderEnd",
    "wgpuComputePassEncoderInsertDebugMarker",
    "wgpuComputePassEncoderPopDebugGroup",
    "wgpuComputePassEncoderPushDebugGroup",
    "wgpuComputePassEncoderRelease",
    "wgpuComputePassEncoderSetBindGroup",
    "wgpuComputePassEncoderSetLabel",
    "wgpuComputePassEncoderSetPipeline",
    "wgpuComputePassEncoderWriteTimestamp",
    "wgpuComputePipelineAddRef",
    "wgpuComputePipelineGetBindGroupLayout",
    "wgpuComputePipelineRelease",
    "wgpuComputePipelineSetLabel",
    "wgpuCreateInstance",
    "wgpuDeviceAddRef",
    "wgpuDeviceCreateBindGroup",
    "wgpuDeviceCreateBindGroupLayout",
    "wgpuDeviceCreateBuffer",
    "wgpuDeviceCreateCommandEncoder",
    "wgpuDeviceCreateComputePipeline",
    "wgpuDeviceCreateComputePipelineAsync",
    "wgpuDeviceCreatePipelineLayout",
    "wgpuDeviceCreateQuerySet",
    "wgpuDeviceCreateRenderBundleEncoder",
    "wgpuDeviceCreateRenderPipeline",
    "wgpuDeviceCreateRenderPipelineAsync",
    "wgpuDeviceCreateSampler",
    "wgpuDeviceCreateShaderModule",
    "wgpuDeviceCreateTexture",
    "wgpuDeviceDestroy",
    "wgpuDeviceGetAdapterInfo",
    "wgpuDeviceGetFeatures",
    "wgpuDeviceGetLimits",
    "wgpuDeviceGetLostFuture",
    "wgpuDeviceGetQueue",
    "wgpuDeviceHasFeature",
    "wgpuDevicePopErrorScope",
    "wgpuDevicePushErrorScope",
    "wgpuDeviceRelease",
    "wgpuDeviceSetLabel",
    "wgpuGetInstanceCapabilities",
    "wgpuGetInstanceFeatures",
    "wgpuGetInstanceLimits",
    "wgpuGetProcAddress",
    "wgpuHasInstanceFeature",
    "wgpuInstanceAddRef",
    "wgpuInstanceCreateSurface",
    "wgpuInstanceGetWGSLLanguageFeatures",
    "wgpuInstanceHasWGSLLanguageFeature",
    "wgpuInstanceProcessEvents",
    "wgpuInstanceRelease",
    "wgpuInstanceRequestAdapter",
    "wgpuInstanceWaitAny",
    "wgpuPipelineLayoutAddRef",
    "wgpuPipelineLayoutRelease",
    "wgpuPipelineLayoutSetLabel",
    "wgpuQuerySetAddRef",
    "wgpuQuerySetDestroy",
    "wgpuQuerySetGetCount",
    "wgpuQuerySetGetType",
    "wgpuQuerySetRelease",
    "wgpuQuerySetSetLabel",
    "wgpuQueueAddRef",
    "wgpuQueueOnSubmittedWorkDone",
    "wgpuQueueRelease",
    "wgpuQueueSetLabel",
    "wgpuQueueSubmit",
    "wgpuQueueWriteBuffer",
    "wgpuQueueWriteTexture",
    "wgpuRenderBundleAddRef",
    "wgpuRenderBundleEncoderAddRef",
    "wgpuRenderBundleEncoderDraw",
    "wgpuRenderBundleEncoderDrawIndexed",
    "wgpuRenderBundleEncoderDrawIndexedIndirect",
    "wgpuRenderBundleEncoderDrawIndirect",
    "wgpuRenderBundleEncoderFinish",
    "wgpuRenderBundleEncoderInsertDebugMarker",
    "wgpuRenderBundleEncoderPopDebugGroup",
    "wgpuRenderBundleEncoderPushDebugGroup",
    "wgpuRenderBundleEncoderRelease",
    "wgpuRenderBundleEncoderSetBindGroup",
    "wgpuRenderBundleEncoderSetIndexBuffer",
    "wgpuRenderBundleEncoderSetLabel",
    "wgpuRenderBundleEncoderSetPipeline",
    "wgpuRenderBundleEncoderSetVertexBuffer",
    "wgpuRenderBundleRelease",
    "wgpuRenderBundleSetLabel",
    "wgpuRenderPassEncoderAddRef",
    "wgpuRenderPassEncoderBeginOcclusionQuery",
    "wgpuRenderPassEncoderDraw",
    "wgpuRenderPassEncoderDrawIndexed",
    "wgpuRenderPassEncoderDrawIndexedIndirect",
    "wgpuRenderPassEncoderDrawIndirect",
    "wgpuRenderPassEncoderEnd",
    "wgpuRenderPassEncoderEndOcclusionQuery",
    "wgpuRenderPassEncoderExecuteBundles",
    "wgpuRenderPassEncoderInsertDebugMarker",
    "wgpuRenderPassEncoderMultiDrawIndexedIndirect",
    "wgpuRenderPassEncoderMultiDrawIndirect",
    "wgpuRenderPassEncoderPopDebugGroup",
    "wgpuRenderPassEncoderPushDebugGroup",
    "wgpuRenderPassEncoderRelease",
    "wgpuRenderPassEncoderSetBindGroup",
    "wgpuRenderPassEncoderSetBlendConstant",
    "wgpuRenderPassEncoderSetIndexBuffer",
    "wgpuRenderPassEncoderSetLabel",
    "wgpuRenderPassEncoderSetPipeline",
    "wgpuRenderPassEncoderSetScissorRect",
    "wgpuRenderPassEncoderSetStencilReference",
    "wgpuRenderPassEncoderSetVertexBuffer",
    "wgpuRenderPassEncoderSetViewport",
    "wgpuRenderPassEncoderWriteTimestamp",
    "wgpuRenderPipelineAddRef",
    "wgpuRenderPipelineGetBindGroupLayout",
    "wgpuRenderPipelineRelease",
    "wgpuRenderPipelineSetLabel",
    "wgpuSamplerAddRef",
    "wgpuSamplerRelease",
    "wgpuSamplerSetLabel",
    "wgpuShaderModuleAddRef",
    "wgpuShaderModuleGetCompilationInfo",
    "wgpuShaderModuleRelease",
    "wgpuShaderModuleSetLabel",
    "wgpuSupportedFeaturesFreeMembers",
    "wgpuSupportedWGSLLanguageFeaturesFreeMembers",
    "wgpuSurfaceAddRef",
    "wgpuSurfaceCapabilitiesFreeMembers",
    "wgpuSurfaceConfigure",
    "wgpuSurfaceGetCapabilities",
    "wgpuSurfaceGetCurrentTexture",
    "wgpuSurfacePresent",
    "wgpuSurfaceRelease",
    "wgpuSurfaceSetLabel",
    "wgpuSurfaceUnconfigure",
    "wgpuTextureAddRef",
    "wgpuTextureCreateView",
    "wgpuTextureDestroy",
    "wgpuTextureGetDepthOrArrayLayers",
    "wgpuTextureGetDimension",
    "wgpuTextureGetFormat",
    "wgpuTextureGetHeight",
    "wgpuTextureGetMipLevelCount",
    "wgpuTextureGetSampleCount",
    "wgpuTextureGetUsage",
    "wgpuTextureGetWidth",
    "wgpuTextureRelease",
    "wgpuTextureSetLabel",
    "wgpuTextureViewAddRef",
    "wgpuTextureViewRelease",
    "wgpuTextureViewSetLabel",
];

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
const _: [(); 199] = [(); UNDEFINED_HOST_SYMBOLS.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn complete_undefined_symbol_denominator_is_frozen() {
        assert_eq!(UNDEFINED_HOST_SYMBOLS.len(), 199);
        assert_eq!(UNDEFINED_HOST_SYMBOLS.iter().copied().collect::<BTreeSet<_>>().len(), 199);
        assert_eq!(UNDEFINED_HOST_SYMBOLS.first(), Some(&"wgpuAdapterAddRef"));
        assert_eq!(UNDEFINED_HOST_SYMBOLS.last(), Some(&"wgpuTextureViewSetLabel"));
        for symbol in UNDEFINED_HOST_SYMBOLS {
            assert!(PINNED_SOURCE.contains(&format!("{symbol}: undefined")));
        }
    }

    #[test]
    fn source_registers_one_undefined_host_library() {
        assert!(PINNED_SOURCE.contains("const LibraryWebGPU = {"));
        assert!(PINNED_SOURCE.contains("addToLibrary(LibraryWebGPU);"));
        assert_eq!(LIBRARY_NAME, "LibraryWebGPU");
        assert_eq!(REGISTRATION_OPERATION, "addToLibrary(LibraryWebGPU)");
    }

    #[test]
    fn version_transition_symbols_are_both_required() {
        for symbol in [
            "wgpuGetInstanceCapabilities",
            "wgpuGetInstanceFeatures",
            "wgpuGetInstanceLimits",
            "wgpuHasInstanceFeature",
        ] {
            assert!(UNDEFINED_HOST_SYMBOLS.contains(&symbol));
        }
    }
}
