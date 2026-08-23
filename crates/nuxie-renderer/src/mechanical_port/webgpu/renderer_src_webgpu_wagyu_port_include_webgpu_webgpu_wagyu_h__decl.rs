//! Generated exact C ABI translation; do not hand edit.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use super::webgpu_decl::*;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_webgpu_wagyu-port_include_webgpu_webgpu_wagyu.h");
pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = 722;
pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = 43_574;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MacroAuthority {
    pub(crate) name: &'static str,
    pub(crate) startLine: usize,
    pub(crate) endLine: usize,
}

pub(crate) const MACRO_AUTHORITIES: &[MacroAuthority] = &[
    MacroAuthority {
        name: "WEBGPU_WAGYU_H",
        startLine: 2,
        endLine: 2,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_EXTENSION_LEVEL",
        startLine: 6,
        endLine: 6,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_RESERVED_RANGE_BASE",
        startLine: 10,
        endLine: 10,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_MAKE_INIT_STRUCT",
        startLine: 12,
        endLine: 12,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_CHAIN_INIT",
        startLine: 14,
        endLine: 15,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_STRLEN",
        startLine: 17,
        endLine: 17,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_PIXEL_LOCAL_STORAGE_SIZE_UNDEFINED",
        startLine: 18,
        endLine: 18,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_NRDP_VERSION_INIT",
        startLine: 129,
        endLine: 130,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_ADAPTER_INFO_INIT",
        startLine: 139,
        endLine: 140,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_COLOR_TARGET_STATE_INIT",
        startLine: 148,
        endLine: 150,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_COMMAND_ENCODER_DESCRIPTOR_INIT",
        startLine: 158,
        endLine: 159,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_COMPUTE_PIPELINE_DESCRIPTOR_INIT",
        startLine: 167,
        endLine: 168,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_ORIGIN_2D_INIT",
        startLine: 176,
        endLine: 177,
    },
    MacroAuthority {
        name: "WGPU_COPY_EXTERNAL_IMAGE_SOURCE_INFO_INIT",
        startLine: 186,
        endLine: 187,
    },
    MacroAuthority {
        name: "WGPU_COPY_EXTERNAL_IMAGE_DEST_INFO_INIT",
        startLine: 199,
        endLine: 200,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_DESCRIPTOR_INIT",
        startLine: 211,
        endLine: 212,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_VALIDATION_INIT",
        startLine: 220,
        endLine: 221,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_FLUSH_CALLBACK_INFO_INIT",
        startLine: 232,
        endLine: 233,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_EXECUTE_CALLBACK_INFO_INIT",
        startLine: 244,
        endLine: 245,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_PIPELINE_BINARY_INIT",
        startLine: 255,
        endLine: 256,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_PIPELINE_BINARY_BLOB_KEY_INIT",
        startLine: 264,
        endLine: 265,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_PIPELINE_BINARY_CACHE_KEY_INIT",
        startLine: 274,
        endLine: 275,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_PIPELINE_BINARY_CACHE_STATISTICS_INIT",
        startLine: 287,
        endLine: 288,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_PIPELINE_BINARY_CACHE_STATISTICS_CALLBACK_INFO_INIT",
        startLine: 299,
        endLine: 300,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_PIPELINE_BINARY_DATA_INIT",
        startLine: 310,
        endLine: 311,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_DEVICE_PIPELINE_BINARY_EVENT_INIT",
        startLine: 321,
        endLine: 322,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_EXTERNAL_TEXTURE_DESCRIPTOR_INIT",
        startLine: 332,
        endLine: 333,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_EXTERNAL_TEXTURE_INFO_INIT",
        startLine: 344,
        endLine: 345,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_EXTERNAL_TEXTURE_BINDING_ENTRY_INIT",
        startLine: 353,
        endLine: 355,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_EXTERNAL_TEXTURE_BINDING_LAYOUT_INIT",
        startLine: 362,
        endLine: 363,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_INPUT_ATTACHMENT_STATE_INIT",
        startLine: 371,
        endLine: 373,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_FRAGMENT_STATE_INIT",
        startLine: 383,
        endLine: 384,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_INPUT_TEXTURE_BINDING_LAYOUT_INIT",
        startLine: 392,
        endLine: 393,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_PIPELINE_BINARY_CALLBACK_INFO_INIT",
        startLine: 404,
        endLine: 405,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_PIPELINE_BINARY_ERROR_CALLBACK_INFO_INIT",
        startLine: 416,
        endLine: 417,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_RECT_INIT",
        startLine: 427,
        endLine: 428,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_RENDER_PASS_INPUT_ATTACHMENT_INIT",
        startLine: 438,
        endLine: 439,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_RENDER_PASS_DESCRIPTOR_INIT",
        startLine: 450,
        endLine: 451,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_RENDER_PIPELINE_DESCRIPTOR_INIT",
        startLine: 459,
        endLine: 460,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_MODULE_ENTRY_POINTS_CALLBACK_INFO_INIT",
        startLine: 471,
        endLine: 472,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_REFLECTION_STRUCT_MEMBER_INIT",
        startLine: 487,
        endLine: 488,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_REFLECTION_LOCATION_INIT",
        startLine: 498,
        endLine: 499,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_REFLECTION_RESOURCE_INIT",
        startLine: 513,
        endLine: 514,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_REFLECTION_SPECIALIZATION_CONSTANT_INIT",
        startLine: 524,
        endLine: 525,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_REFLECTION_DATA_INIT",
        startLine: 540,
        endLine: 541,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_ENTRY_POINT_INIT",
        startLine: 550,
        endLine: 551,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_ENTRY_POINT_ARRAY_INIT",
        startLine: 559,
        endLine: 560,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_MODULE_COMPILATION_HINT_INIT",
        startLine: 572,
        endLine: 573,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SHADER_MODULE_DESCRIPTOR_INIT",
        startLine: 585,
        endLine: 586,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_STRING_ARRAY_INIT",
        startLine: 594,
        endLine: 595,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_SURFACE_CONFIGURATION_INIT",
        startLine: 605,
        endLine: 606,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_TEXTURE_DESCRIPTOR_INIT",
        startLine: 614,
        endLine: 615,
    },
    MacroAuthority {
        name: "WGPU_WAGYU_WGSL_FEATURE_TYPE_ARRAY_INIT",
        startLine: 623,
        endLine: 624,
    },
];

pub(crate) fn macroSource(authority: MacroAuthority) -> String {
    PINNED_SOURCE
        .lines()
        .skip(authority.startLine - 1)
        .take(authority.endLine - authority.startLine + 1)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) const WGPU_WAGYU_EXTENSION_LEVEL: u32 = 1;
pub(crate) const WGPU_WAGYU_RESERVED_RANGE_BASE: i32 = 0x0006_0000;
pub(crate) const WGPU_WAGYU_STRLEN: usize = usize::MAX;
pub(crate) const WGPU_WAGYU_PIXEL_LOCAL_STORAGE_SIZE_UNDEFINED: u32 = u32::MAX;

#[repr(C)]
pub(crate) struct WGPUWagyuExternalTextureImpl {
    _private: [u8; 0],
}

pub(crate) type WGPUWagyuExternalTexture = *mut WGPUWagyuExternalTextureImpl;

pub(crate) type WGPUSType_Wagyu = i32;
pub(crate) const WGPUSType_WagyuAdapterInfo: WGPUSType_Wagyu = 393217;
pub(crate) const WGPUSType_WagyuColorTargetState: WGPUSType_Wagyu = 393218;
pub(crate) const WGPUSType_WagyuCommandEncoderDescriptor: WGPUSType_Wagyu = 393219;
pub(crate) const WGPUSType_WagyuComputePipelineDescriptor: WGPUSType_Wagyu = 393220;
pub(crate) const WGPUSType_WagyuDeviceDescriptor: WGPUSType_Wagyu = 393221;
pub(crate) const WGPUSType_WagyuExternalTextureBindingEntry: WGPUSType_Wagyu = 393222;
pub(crate) const WGPUSType_WagyuExternalTextureBindingLayout: WGPUSType_Wagyu = 393223;
pub(crate) const WGPUSType_WagyuFragmentState: WGPUSType_Wagyu = 393224;
pub(crate) const WGPUSType_WagyuInputTextureBindingLayout: WGPUSType_Wagyu = 393225;
pub(crate) const WGPUSType_WagyuRenderPassDescriptor: WGPUSType_Wagyu = 393226;
pub(crate) const WGPUSType_WagyuRenderPipelineDescriptor: WGPUSType_Wagyu = 393227;
pub(crate) const WGPUSType_WagyuShaderModuleDescriptor: WGPUSType_Wagyu = 393228;
pub(crate) const WGPUSType_WagyuSurfaceConfiguration: WGPUSType_Wagyu = 393229;
pub(crate) const WGPUSType_WagyuTextureDescriptor: WGPUSType_Wagyu = 393230;
pub(crate) const WGPUSType_WagyuDeviceWantsValidationDescriptor: WGPUSType_Wagyu = 393231;
pub(crate) const WGPUSType_WagyuForce32: WGPUSType_Wagyu = 2147483647;

pub(crate) type WGPUWagyuDeviceFlushStatus = i32;
pub(crate) const WGPUWagyuDeviceFlushStatus_Success: WGPUWagyuDeviceFlushStatus = 0;
pub(crate) const WGPUWagyuDeviceFlushStatus_Error: WGPUWagyuDeviceFlushStatus = 1;
pub(crate) const WGPUWagyuDeviceFlushStatus_Force32: WGPUWagyuDeviceFlushStatus = 2147483647;

pub(crate) type WGPUWagyuDevicePipelineBinaryCacheError = i32;
pub(crate) const WGPUWagyuDevicePipelineBinaryCacheError_Version:
    WGPUWagyuDevicePipelineBinaryCacheError = 0;
pub(crate) const WGPUWagyuDevicePipelineBinaryCacheError_Corrupt:
    WGPUWagyuDevicePipelineBinaryCacheError = 1;
pub(crate) const WGPUWagyuDevicePipelineBinaryCacheError_Link:
    WGPUWagyuDevicePipelineBinaryCacheError = 2;
pub(crate) const WGPUWagyuDevicePipelineBinaryCacheError_Create:
    WGPUWagyuDevicePipelineBinaryCacheError = 3;
pub(crate) const WGPUWagyuDevicePipelineBinaryCacheError_Force32:
    WGPUWagyuDevicePipelineBinaryCacheError = 2147483647;

pub(crate) type WGPUWagyuShaderLanguage = i32;
pub(crate) const WGPUWagyuShaderLanguage_Detect: WGPUWagyuShaderLanguage = 0;
pub(crate) const WGPUWagyuShaderLanguage_GLSL: WGPUWagyuShaderLanguage = 1;
pub(crate) const WGPUWagyuShaderLanguage_GLSLRAW: WGPUWagyuShaderLanguage = 2;
pub(crate) const WGPUWagyuShaderLanguage_WGSL: WGPUWagyuShaderLanguage = 3;
pub(crate) const WGPUWagyuShaderLanguage_SPIRV: WGPUWagyuShaderLanguage = 4;
pub(crate) const WGPUWagyuShaderLanguage_Force32: WGPUWagyuShaderLanguage = 2147483647;

pub(crate) type WGPUWagyuWGSLFeatureType = i32;
pub(crate) const WGPUWagyuWGSLFeatureType_Testing: WGPUWagyuWGSLFeatureType = 1;
pub(crate) const WGPUWagyuWGSLFeatureType_UnsafeExperimental: WGPUWagyuWGSLFeatureType = 2;
pub(crate) const WGPUWagyuWGSLFeatureType_Experimental: WGPUWagyuWGSLFeatureType = 4;
pub(crate) const WGPUWagyuWGSLFeatureType_All: WGPUWagyuWGSLFeatureType = 7;
pub(crate) const WGPUWagyuWGSLFeatureType_Force32: WGPUWagyuWGSLFeatureType = 2147483647;

pub(crate) type WGPUWagyuFragmentStateFeaturesFlags = WGPUFlags;

pub(crate) type WGPUWagyuDeviceFlushCallback = Option<
    unsafe extern "C" fn(
        WGPUDevice,
        WGPUWagyuDeviceFlushStatus,
        *mut std::ffi::c_void,
        *mut std::ffi::c_void,
    ),
>;
pub(crate) type WGPUWagyuExecuteCallback =
    Option<unsafe extern "C" fn(*mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPUWagyuPipelineBinaryCacheStatisticsCallback = Option<
    unsafe extern "C" fn(
        WGPUDevice,
        WGPUWagyuDeviceFlushStatus,
        *const WGPUWagyuDevicePipelineBinaryCacheStatistics,
        *mut std::ffi::c_void,
        *mut std::ffi::c_void,
    ),
>;
pub(crate) type WGPUWagyuPipelineBinaryCallback = Option<
    unsafe extern "C" fn(
        WGPUDevice,
        *const WGPUWagyuDevicePipelineBinaryEvent,
        *mut std::ffi::c_void,
        *mut std::ffi::c_void,
    ),
>;
pub(crate) type WGPUWagyuPipelineBinaryErrorCallback = Option<
    unsafe extern "C" fn(
        WGPUDevice,
        WGPUStringView,
        WGPUStringView,
        *mut std::ffi::c_void,
        *mut std::ffi::c_void,
    ),
>;
pub(crate) type WGPUWagyuShaderModuleEntryPointsCallback = Option<
    unsafe extern "C" fn(
        WGPUShaderModule,
        WGPUWagyuDeviceFlushStatus,
        *const WGPUWagyuShaderEntryPointArray,
        *mut std::ffi::c_void,
        *mut std::ffi::c_void,
    ),
>;

pub(crate) const WGPUFeatureName_WagyuBlendEquationAdvancedCoherent: WGPUFeatureName = 393217;
pub(crate) const WGPUBlendOperation_WagyuMultiply: WGPUBlendOperation = 393217;
pub(crate) const WGPUBlendOperation_WagyuScreen: WGPUBlendOperation = 393218;
pub(crate) const WGPUBlendOperation_WagyuOverlay: WGPUBlendOperation = 393219;
pub(crate) const WGPUBlendOperation_WagyuDarken: WGPUBlendOperation = 393220;
pub(crate) const WGPUBlendOperation_WagyuLighten: WGPUBlendOperation = 393221;
pub(crate) const WGPUBlendOperation_WagyuColorDodge: WGPUBlendOperation = 393222;
pub(crate) const WGPUBlendOperation_WagyuColorBurn: WGPUBlendOperation = 393223;
pub(crate) const WGPUBlendOperation_WagyuHardLight: WGPUBlendOperation = 393224;
pub(crate) const WGPUBlendOperation_WagyuSoftLight: WGPUBlendOperation = 393225;
pub(crate) const WGPUBlendOperation_WagyuDifference: WGPUBlendOperation = 393226;
pub(crate) const WGPUBlendOperation_WagyuExclusion: WGPUBlendOperation = 393227;
pub(crate) const WGPUBlendOperation_WagyuHue: WGPUBlendOperation = 393228;
pub(crate) const WGPUBlendOperation_WagyuSaturation: WGPUBlendOperation = 393229;
pub(crate) const WGPUBlendOperation_WagyuColor: WGPUBlendOperation = 393230;
pub(crate) const WGPUBlendOperation_WagyuLuminosity: WGPUBlendOperation = 393231;
pub(crate) const WGPUWagyuFragmentStateFeaturesFlags_None: WGPUWagyuFragmentStateFeaturesFlags =
    0 as WGPUWagyuFragmentStateFeaturesFlags;
pub(crate) const WGPUWagyuFragmentStateFeaturesFlags_RasterizationOrderAttachmentAccess:
    WGPUWagyuFragmentStateFeaturesFlags = 1 as WGPUWagyuFragmentStateFeaturesFlags;
pub(crate) const WGPUTextureUsage_WagyuInputAttachment: WGPUTextureUsage =
    1073741824 as WGPUTextureUsage;
pub(crate) const WGPUTextureUsage_WagyuTransientAttachment: WGPUTextureUsage =
    536870912 as WGPUTextureUsage;
pub(crate) const WGPUTextureUsage_WagyuMSAAResolveSource: WGPUTextureUsage =
    2147483648 as WGPUTextureUsage;

#[repr(C)]
pub(crate) struct WGPUWagyuNrdpVersion {
    pub(crate) major: u32,
    pub(crate) minor: u32,
    pub(crate) patch: u32,
    pub(crate) rev: u32,
}

#[repr(C)]
pub(crate) struct WGPUWagyuAdapterInfo {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) extensionLevel: u32,
    pub(crate) nrdpVersion: WGPUWagyuNrdpVersion,
}

#[repr(C)]
pub(crate) struct WGPUWagyuColorTargetState {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) usedAsInput: WGPUOptionalBool,
}

#[repr(C)]
pub(crate) struct WGPUWagyuCommandEncoderDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) measureExecutionTime: WGPUOptionalBool,
}

#[repr(C)]
pub(crate) struct WGPUWagyuComputePipelineDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) cacheKey: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUWagyuOrigin2D {
    pub(crate) x: u32,
    pub(crate) y: u32,
}

#[repr(C)]
pub(crate) struct WGPUWagyuCopyExternalImageSourceInfo {
    pub(crate) source: WGPUStringView,
    pub(crate) origin: WGPUWagyuOrigin2D,
    pub(crate) flipY: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUWagyuCopyExternalImageDestInfo {
    pub(crate) texture: WGPUTexture,
    pub(crate) mipLevel: u32,
    pub(crate) origin: WGPUOrigin3D,
    pub(crate) aspect: WGPUTextureAspect,
    pub(crate) colorSpace: WGPUPredefinedColorSpace,
    pub(crate) premultipliedAlpha: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDeviceDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) dataBufferNeedsDetach: WGPUOptionalBool,
    pub(crate) wantsIndirectRendering: WGPUOptionalBool,
    pub(crate) wantsBufferClear: WGPUOptionalBool,
    pub(crate) wantsTextureClear: WGPUOptionalBool,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDeviceWantsValidationDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) wantsValidation: WGPUOptionalBool,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDeviceFlushCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUWagyuDeviceFlushCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUWagyuExecuteCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUWagyuExecuteCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDevicePipelineBinary {
    pub(crate) binarySize: usize,
    pub(crate) binary: *mut std::ffi::c_void,
    pub(crate) blobKeySize: usize,
    pub(crate) blobKey: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDevicePipelineBinaryBlobKey {
    pub(crate) size: usize,
    pub(crate) data: *const std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDevicePipelineBinaryCacheKey {
    pub(crate) cacheKey: WGPUStringView,
    pub(crate) blobKeysLength: usize,
    pub(crate) blobKeys: *mut WGPUWagyuDevicePipelineBinaryBlobKey,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDevicePipelineBinaryCacheStatistics {
    pub(crate) hits: u32,
    pub(crate) misses: u32,
    pub(crate) entryCount: usize,
    pub(crate) entries: *mut WGPUStringView,
    pub(crate) errorCount: usize,
    pub(crate) errors: *mut WGPUWagyuDevicePipelineBinaryCacheError,
}

#[repr(C)]
pub(crate) struct WGPUWagyuPipelineBinaryCacheStatisticsCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUWagyuPipelineBinaryCacheStatisticsCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDevicePipelineBinaryData {
    pub(crate) binariesLength: usize,
    pub(crate) binaries: *mut WGPUWagyuDevicePipelineBinary,
    pub(crate) cacheKeysLength: usize,
    pub(crate) cacheKeys: *mut WGPUWagyuDevicePipelineBinaryCacheKey,
}

#[repr(C)]
pub(crate) struct WGPUWagyuDevicePipelineBinaryEvent {
    pub(crate) cacheKey: WGPUStringView,
    pub(crate) pipelineLabel: WGPUStringView,
    pub(crate) binariesLength: usize,
    pub(crate) binaries: *const WGPUWagyuDevicePipelineBinary,
}

#[repr(C)]
pub(crate) struct WGPUWagyuExternalTextureDescriptor {
    pub(crate) nextInChain: *const WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) source: WGPUStringView,
    pub(crate) colorSpace: WGPUPredefinedColorSpace,
}

#[repr(C)]
pub(crate) struct WGPUWagyuExternalTextureInfo {
    pub(crate) visibleWidth: u32,
    pub(crate) visibleHeight: u32,
    pub(crate) textureWidth: u32,
    pub(crate) textureHeight: u32,
    pub(crate) pts: i64,
}

#[repr(C)]
pub(crate) struct WGPUWagyuExternalTextureBindingEntry {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) externalTexture: WGPUWagyuExternalTexture,
}

#[repr(C)]
pub(crate) struct WGPUWagyuExternalTextureBindingLayout {
    pub(crate) chain: WGPUChainedStruct,
}

#[repr(C)]
pub(crate) struct WGPUWagyuInputAttachmentState {
    pub(crate) format: WGPUTextureFormat,
    pub(crate) usedAsColor: WGPUOptionalBool,
}

#[repr(C)]
pub(crate) struct WGPUWagyuFragmentState {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) inputCount: usize,
    pub(crate) inputs: *mut WGPUWagyuInputAttachmentState,
    pub(crate) featureFlags: WGPUWagyuFragmentStateFeaturesFlags,
}

#[repr(C)]
pub(crate) struct WGPUWagyuInputTextureBindingLayout {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) viewDimension: WGPUTextureViewDimension,
}

#[repr(C)]
pub(crate) struct WGPUWagyuPipelineBinaryCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUWagyuPipelineBinaryCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUWagyuPipelineBinaryErrorCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUWagyuPipelineBinaryErrorCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUWagyuRect {
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) width: u32,
    pub(crate) height: u32,
}

#[repr(C)]
pub(crate) struct WGPUWagyuRenderPassInputAttachment {
    pub(crate) view: WGPUTextureView,
    pub(crate) clearValue: *mut WGPUColor,
    pub(crate) loadOp: WGPULoadOp,
    pub(crate) storeOp: WGPUStoreOp,
}

#[repr(C)]
pub(crate) struct WGPUWagyuRenderPassDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) inputAttachmentCount: usize,
    pub(crate) inputAttachments: *mut WGPUWagyuRenderPassInputAttachment,
    pub(crate) pixelLocalStorageEnabled: WGPUOptionalBool,
    pub(crate) pixelLocalStorageSize: u32,
}

#[repr(C)]
pub(crate) struct WGPUWagyuRenderPipelineDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) cacheKey: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderModuleEntryPointsCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUWagyuShaderModuleEntryPointsCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderReflectionStructMember {
    pub(crate) name: WGPUStringView,
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) offset: u32,
    pub(crate) size: u32,
    pub(crate) r#type: u32,
    pub(crate) imageMultisampled: WGPUBool,
    pub(crate) imageDimension: WGPUTextureViewDimension,
    pub(crate) imageFormat: WGPUTextureFormat,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderReflectionLocation {
    pub(crate) name: WGPUStringView,
    pub(crate) location: u32,
    pub(crate) size: u32,
    pub(crate) r#type: u32,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderReflectionResource {
    pub(crate) name: WGPUStringView,
    pub(crate) group: u32,
    pub(crate) binding: u32,
    pub(crate) bindingType: u32,
    pub(crate) multisampled: WGPUBool,
    pub(crate) dimension: WGPUTextureViewDimension,
    pub(crate) format: WGPUTextureFormat,
    pub(crate) bufferSize: u64,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderReflectionSpecializationConstant {
    pub(crate) id: u32,
    pub(crate) internalId: u32,
    pub(crate) r#type: u32,
    pub(crate) name: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderReflectionData {
    pub(crate) resourceCount: usize,
    pub(crate) resources: *mut WGPUWagyuShaderReflectionResource,
    pub(crate) constantCount: usize,
    pub(crate) constants: *mut WGPUWagyuShaderReflectionSpecializationConstant,
    pub(crate) uniformCount: usize,
    pub(crate) uniforms: *mut WGPUWagyuShaderReflectionStructMember,
    pub(crate) attributeCount: usize,
    pub(crate) attributes: *mut WGPUWagyuShaderReflectionLocation,
    pub(crate) wgsl: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderEntryPoint {
    pub(crate) entryPoint: WGPUStringView,
    pub(crate) stage: WGPUShaderStage,
    pub(crate) reflection: WGPUWagyuShaderReflectionData,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderEntryPointArray {
    pub(crate) entryPointCount: usize,
    pub(crate) entryPoints: *mut WGPUWagyuShaderEntryPoint,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderModuleCompilationHint {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) entryPoint: WGPUStringView,
    pub(crate) layout: WGPUPipelineLayout,
}

#[repr(C)]
pub(crate) struct WGPUWagyuShaderModuleDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) codeSize: usize,
    pub(crate) code: *const std::ffi::c_void,
    pub(crate) language: WGPUWagyuShaderLanguage,
    pub(crate) compilationHintCount: usize,
    pub(crate) compilationHints: *const WGPUWagyuShaderModuleCompilationHint,
}

#[repr(C)]
pub(crate) struct WGPUWagyuStringArray {
    pub(crate) stringCount: usize,
    pub(crate) strings: *mut WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUWagyuSurfaceConfiguration {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) indirectRenderTargets: *mut i32,
    pub(crate) colorSpace: WGPUPredefinedColorSpace,
    pub(crate) toneMappingMode: WGPUToneMappingMode,
}

#[repr(C)]
pub(crate) struct WGPUWagyuTextureDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) useSurfaceCache: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUWagyuWGSLFeatureTypeArray {
    pub(crate) featureCount: usize,
    pub(crate) features: *mut WGPUWagyuWGSLFeatureType,
}

unsafe extern "C" {
    pub(crate) fn wgpuWagyuAdapterGetBackend(arg0: WGPUAdapter) -> WGPUBackendType;
    pub(crate) fn wgpuWagyuAdapterGetExtensions(arg0: WGPUAdapter, arg1: *mut WGPUWagyuStringArray);
    pub(crate) fn wgpuWagyuAdapterGetGraphicsReport(arg0: WGPUAdapter, arg1: *mut WGPUStringView);
    pub(crate) fn wgpuWagyuAdapterGetName(arg0: WGPUAdapter, arg1: *mut WGPUStringView);
    pub(crate) fn wgpuWagyuAdapterRequestDeviceSync(
        arg0: WGPUAdapter,
        arg1: *const WGPUDeviceDescriptor,
    ) -> WGPUDevice;
    pub(crate) fn wgpuWagyuCommandEncoderBlit(
        arg0: WGPUCommandEncoder,
        arg1: *const WGPUTexelCopyTextureInfo,
        arg2: *const WGPUExtent3D,
        arg3: *const WGPUTexelCopyTextureInfo,
        arg4: *const WGPUExtent3D,
        arg5: WGPUFilterMode,
    );
    pub(crate) fn wgpuWagyuCommandEncoderExecuteCallback(
        arg0: WGPUCommandEncoder,
        arg1: WGPUWagyuExecuteCallbackInfo,
    );
    pub(crate) fn wgpuWagyuCommandEncoderGenerateMipmap(
        arg0: WGPUCommandEncoder,
        arg1: WGPUTexture,
    );
    pub(crate) fn wgpuWagyuComputePassEncoderExecuteCallback(
        arg0: WGPUComputePassEncoder,
        arg1: WGPUWagyuExecuteCallbackInfo,
    );
    pub(crate) fn wgpuWagyuDeviceClearPipelineBinaryCache(arg0: WGPUDevice);
    pub(crate) fn wgpuWagyuDeviceEnableImaginationWorkarounds(arg0: WGPUDevice, arg1: WGPUBool);
    pub(crate) fn wgpuWagyuDeviceGetExtensions(arg0: WGPUDevice, arg1: *mut WGPUWagyuStringArray);
    pub(crate) fn wgpuWagyuDeviceFlush(
        arg0: WGPUDevice,
        arg1: WGPUWagyuDeviceFlushCallbackInfo,
    ) -> WGPUFuture;
    pub(crate) fn wgpuWagyuDeviceImportExternalTexture(
        arg0: WGPUDevice,
        arg1: *const WGPUWagyuExternalTextureDescriptor,
    ) -> WGPUWagyuExternalTexture;
    pub(crate) fn wgpuWagyuDeviceIntrospectShaderCode(
        arg0: WGPUDevice,
        arg1: WGPUShaderStage,
        arg2: *const WGPUShaderModuleDescriptor,
        arg3: *mut WGPUWagyuShaderEntryPointArray,
    );
    pub(crate) fn wgpuWagyuDevicePipelineBinaryCacheStatistics(
        arg0: WGPUDevice,
        arg1: WGPUWagyuPipelineBinaryCacheStatisticsCallbackInfo,
    ) -> WGPUFuture;
    pub(crate) fn wgpuWagyuDevicePopulatePipelineBinaryCache(
        arg0: WGPUDevice,
        arg1: *const WGPUWagyuDevicePipelineBinaryData,
    );
    pub(crate) fn wgpuWagyuDeviceSetPipelineBinaryCallback(
        arg0: WGPUDevice,
        arg1: WGPUWagyuPipelineBinaryCallbackInfo,
    );
    pub(crate) fn wgpuWagyuDeviceSetPipelineBinaryErrorCallback(
        arg0: WGPUDevice,
        arg1: WGPUWagyuPipelineBinaryErrorCallbackInfo,
    );
    pub(crate) fn wgpuWagyuExternalTextureAddRef(arg0: WGPUWagyuExternalTexture);
    pub(crate) fn wgpuWagyuExternalTextureGetInfo(
        arg0: WGPUWagyuExternalTexture,
        arg1: *mut WGPUWagyuExternalTextureInfo,
    );
    pub(crate) fn wgpuWagyuExternalTextureRelease(arg0: WGPUWagyuExternalTexture);
    pub(crate) fn wgpuWagyuExternalTextureSetLabel(
        arg0: WGPUWagyuExternalTexture,
        arg1: WGPUStringView,
    );
    pub(crate) fn wgpuWagyuInstanceEnableImaginationWorkarounds(arg0: WGPUInstance, arg1: WGPUBool);
    pub(crate) fn wgpuWagyuInstanceGetApiVersion(arg0: WGPUInstance) -> u32;
    pub(crate) fn wgpuWagyuInstanceGetBackend(arg0: WGPUInstance) -> WGPUBackendType;
    pub(crate) fn wgpuWagyuInstanceGetExposedWgslFeatures(
        arg0: WGPUInstance,
        arg1: *mut WGPUWagyuWGSLFeatureTypeArray,
    );
    pub(crate) fn wgpuWagyuInstanceGetScreenDirectFormat(arg0: WGPUInstance) -> WGPUTextureFormat;
    pub(crate) fn wgpuWagyuInstanceGetScreenIndirectFormat(arg0: WGPUInstance)
        -> WGPUTextureFormat;
    pub(crate) fn wgpuWagyuInstanceGetSync(arg0: WGPUInstance) -> WGPUBool;
    pub(crate) fn wgpuWagyuInstanceRequestAdapterSync(
        arg0: WGPUInstance,
        arg1: *const WGPURequestAdapterOptions,
    ) -> WGPUAdapter;
    pub(crate) fn wgpuWagyuInstanceSetCommandBufferLimit(arg0: WGPUInstance, arg1: u32);
    pub(crate) fn wgpuWagyuInstanceSetExposedWgslFeatures(
        arg0: WGPUInstance,
        arg1: *const WGPUWagyuWGSLFeatureTypeArray,
    );
    pub(crate) fn wgpuWagyuInstanceSetImmediate(arg0: WGPUInstance, arg1: WGPUBool);
    pub(crate) fn wgpuWagyuInstanceSetRunBarriersOnIncoherent(arg0: WGPUInstance, arg1: WGPUBool);
    pub(crate) fn wgpuWagyuInstanceSetStagingBufferCacheSize(arg0: WGPUInstance, arg1: u32);
    pub(crate) fn wgpuWagyuInstanceSetSync(arg0: WGPUInstance, arg1: WGPUBool);
    pub(crate) fn wgpuWagyuQueueCopyExternalImageToTexture(
        arg0: WGPUQueue,
        arg1: *const WGPUWagyuCopyExternalImageSourceInfo,
        arg2: *const WGPUWagyuCopyExternalImageDestInfo,
        arg3: *const WGPUExtent3D,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderClearColorAttachments(
        arg0: WGPURenderBundleEncoder,
        arg1: *const WGPUWagyuRect,
        arg2: u32,
        arg3: u32,
        arg4: *const WGPUColor,
        arg5: u32,
        arg6: u32,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderClearDepthAttachment(
        arg0: WGPURenderBundleEncoder,
        arg1: *const WGPUWagyuRect,
        arg2: f32,
        arg3: u32,
        arg4: u32,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderClearPixelLocalStorage(
        arg0: WGPURenderBundleEncoder,
        arg1: u32,
        arg2: u32,
        arg3: *const u32,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderClearStencilAttachment(
        arg0: WGPURenderBundleEncoder,
        arg1: *const WGPUWagyuRect,
        arg2: u32,
        arg3: u32,
        arg4: u32,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderExecuteCallback(
        arg0: WGPURenderBundleEncoder,
        arg1: WGPUWagyuExecuteCallbackInfo,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderSetScissorRect(
        arg0: WGPURenderBundleEncoder,
        arg1: u32,
        arg2: u32,
        arg3: u32,
        arg4: u32,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderSetScissorRectIndirect(
        arg0: WGPURenderBundleEncoder,
        arg1: u64,
        arg2: *const u32,
        arg3: usize,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderSetViewport(
        arg0: WGPURenderBundleEncoder,
        arg1: f32,
        arg2: f32,
        arg3: f32,
        arg4: f32,
        arg5: f32,
        arg6: f32,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderSetViewportWithDepthIndirect(
        arg0: WGPURenderBundleEncoder,
        arg1: u64,
        arg2: *const f32,
        arg3: usize,
    );
    pub(crate) fn wgpuWagyuRenderBundleEncoderSetViewportWithoutDepthIndirect(
        arg0: WGPURenderBundleEncoder,
        arg1: u64,
        arg2: *const f32,
        arg3: usize,
    );
    pub(crate) fn wgpuWagyuRenderPassEncoderClearColorAttachments(
        arg0: WGPURenderPassEncoder,
        arg1: *const WGPUWagyuRect,
        arg2: u32,
        arg3: u32,
        arg4: *const WGPUColor,
        arg5: u32,
        arg6: u32,
    );
    pub(crate) fn wgpuWagyuRenderPassEncoderClearDepthAttachment(
        arg0: WGPURenderPassEncoder,
        arg1: *const WGPUWagyuRect,
        arg2: f32,
        arg3: u32,
        arg4: u32,
    );
    pub(crate) fn wgpuWagyuRenderPassEncoderClearStencilAttachment(
        arg0: WGPURenderPassEncoder,
        arg1: *const WGPUWagyuRect,
        arg2: u32,
        arg3: u32,
        arg4: u32,
    );
    pub(crate) fn wgpuWagyuRenderPassEncoderClearPixelLocalStorage(
        arg0: WGPURenderPassEncoder,
        arg1: u32,
        arg2: u32,
        arg3: *const u32,
    );
    pub(crate) fn wgpuWagyuRenderPassEncoderExecuteBundle(
        arg0: WGPURenderPassEncoder,
        arg1: WGPURenderBundle,
    );
    pub(crate) fn wgpuWagyuRenderPassEncoderExecuteCallback(
        arg0: WGPURenderPassEncoder,
        arg1: WGPUWagyuExecuteCallbackInfo,
    );
    pub(crate) fn wgpuWagyuShaderEntryPointArrayFreeMembers(arg0: WGPUWagyuShaderEntryPointArray);
    pub(crate) fn wgpuWagyuShaderModuleDestroy(arg0: WGPUShaderModule);
    pub(crate) fn wgpuWagyuShaderModuleEntryPoints(
        arg0: WGPUShaderModule,
        arg1: u32,
        arg2: WGPUWagyuShaderModuleEntryPointsCallbackInfo,
    ) -> WGPUFuture;
    pub(crate) fn wgpuWagyuShaderModuleIntrospect(
        arg0: WGPUShaderModule,
        arg1: WGPUShaderStage,
        arg2: *mut WGPUWagyuShaderEntryPointArray,
    );
    pub(crate) fn wgpuWagyuStringArrayFreeMembers(arg0: WGPUWagyuStringArray);
    pub(crate) fn wgpuWagyuSurfaceDestroy(arg0: WGPUSurface);
    pub(crate) fn wgpuWagyuSurfaceGetCurrentDepthStencilTexture(arg0: WGPUSurface) -> WGPUTexture;
    pub(crate) fn wgpuWagyuSurfaceGetHeight(arg0: WGPUSurface) -> f32;
    pub(crate) fn wgpuWagyuSurfaceGetWidth(arg0: WGPUSurface) -> f32;
    pub(crate) fn wgpuWagyuSurfaceGetX(arg0: WGPUSurface) -> f32;
    pub(crate) fn wgpuWagyuSurfaceGetY(arg0: WGPUSurface) -> f32;
    pub(crate) fn wgpuWagyuSurfacePresent(arg0: WGPUSurface, arg1: WGPUTexture);
    pub(crate) fn wgpuWagyuSurfaceSetHeight(arg0: WGPUSurface, arg1: f32);
    pub(crate) fn wgpuWagyuSurfaceSetWidth(arg0: WGPUSurface, arg1: f32);
    pub(crate) fn wgpuWagyuSurfaceSetX(arg0: WGPUSurface, arg1: f32);
    pub(crate) fn wgpuWagyuSurfaceSetY(arg0: WGPUSurface, arg1: f32);
    pub(crate) fn wgpuWagyuTextureIsSwapchain(arg0: WGPUTexture) -> WGPUBool;
    pub(crate) fn wgpuWagyuTextureReadPixels(
        arg0: WGPUTexture,
        arg1: *mut std::ffi::c_void,
        arg2: usize,
    );
    pub(crate) fn wgpuWagyuWGSLFeatureTypeArrayFreeMembers(arg0: WGPUWagyuWGSLFeatureTypeArray);
}

pub(crate) const ABI_ENUM_COUNT: usize = 5;
pub(crate) const ABI_STRUCT_COUNT: usize = 46;
pub(crate) const ABI_FIELD_COUNT: usize = 177;
pub(crate) const ABI_HANDLE_COUNT: usize = 1;
pub(crate) const ABI_FUNCTION_POINTER_COUNT: usize = 6;
pub(crate) const ABI_STATIC_CONSTANT_COUNT: usize = 21;
pub(crate) const ABI_FUNCTION_COUNT: usize = 73;
pub(crate) const PREPROCESSOR_DEFINITION_COUNT: usize = 53;
const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn complete_preprocessor_denominator_is_frozen() {
        assert_eq!(MACRO_AUTHORITIES.len(), PREPROCESSOR_DEFINITION_COUNT);
        for authority in MACRO_AUTHORITIES {
            assert!(macroSource(*authority).trim_start().starts_with('#'));
            assert!(macroSource(*authority).contains("define"));
        }
    }

    #[test]
    fn exported_function_names_are_unique() {
        let mut names = BTreeSet::new();
        for line in PINNED_SOURCE.lines() {
            if let Some(index) = line.find(" wgpu") {
                let name = line[index + 1..].split('(').next().unwrap_or("");
                if name.starts_with("wgpu") {
                    names.insert(name);
                }
            }
        }
        assert!(names.len() >= ABI_FUNCTION_COUNT);
    }

    #[test]
    fn reserved_range_constants_keep_their_source_offsets() {
        assert_eq!(WGPUFeatureName_WagyuBlendEquationAdvancedCoherent, 393217);
        assert_eq!(WGPUBlendOperation_WagyuMultiply, 393217);
        assert_eq!(WGPUBlendOperation_WagyuScreen, 393218);
        assert_eq!(WGPUBlendOperation_WagyuOverlay, 393219);
        assert_eq!(WGPUBlendOperation_WagyuDarken, 393220);
        assert_eq!(WGPUBlendOperation_WagyuLighten, 393221);
        assert_eq!(WGPUBlendOperation_WagyuColorDodge, 393222);
        assert_eq!(WGPUBlendOperation_WagyuColorBurn, 393223);
        assert_eq!(WGPUBlendOperation_WagyuHardLight, 393224);
        assert_eq!(WGPUBlendOperation_WagyuSoftLight, 393225);
        assert_eq!(WGPUBlendOperation_WagyuDifference, 393226);
        assert_eq!(WGPUBlendOperation_WagyuExclusion, 393227);
        assert_eq!(WGPUBlendOperation_WagyuHue, 393228);
        assert_eq!(WGPUBlendOperation_WagyuSaturation, 393229);
        assert_eq!(WGPUBlendOperation_WagyuColor, 393230);
        assert_eq!(WGPUBlendOperation_WagyuLuminosity, 393231);
    }
}
