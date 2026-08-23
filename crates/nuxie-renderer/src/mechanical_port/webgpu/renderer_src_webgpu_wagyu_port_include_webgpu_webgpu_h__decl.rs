//! Generated exact C ABI translation; do not hand edit.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

pub(crate) const PINNED_SOURCE: &str = include_str!("source/renderer_src_webgpu_wagyu-port_include_webgpu_webgpu.h");
pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = 2828;
pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = 148_027;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MacroAuthority {
    pub(crate) name: &'static str,
    pub(crate) startLine: usize,
    pub(crate) endLine: usize,
}

pub(crate) const MACRO_AUTHORITIES: &[MacroAuthority] = &[
    MacroAuthority { name: "WEBGPU_H_", startLine: 32, endLine: 32 },
    MacroAuthority { name: "WGPU_EXPORT", startLine: 37, endLine: 37 },
    MacroAuthority { name: "WGPU_EXPORT", startLine: 39, endLine: 39 },
    MacroAuthority { name: "WGPU_EXPORT", startLine: 43, endLine: 43 },
    MacroAuthority { name: "WGPU_EXPORT", startLine: 45, endLine: 45 },
    MacroAuthority { name: "WGPU_EXPORT", startLine: 49, endLine: 49 },
    MacroAuthority { name: "WGPU_OBJECT_ATTRIBUTE", startLine: 53, endLine: 53 },
    MacroAuthority { name: "WGPU_ENUM_ATTRIBUTE", startLine: 56, endLine: 56 },
    MacroAuthority { name: "WGPU_STRUCTURE_ATTRIBUTE", startLine: 59, endLine: 59 },
    MacroAuthority { name: "WGPU_FUNCTION_ATTRIBUTE", startLine: 62, endLine: 62 },
    MacroAuthority { name: "WGPU_NULLABLE", startLine: 65, endLine: 65 },
    MacroAuthority { name: "_wgpu_COMMA", startLine: 72, endLine: 72 },
    MacroAuthority { name: "_wgpu_ENUM_ZERO_INIT", startLine: 74, endLine: 74 },
    MacroAuthority { name: "_wgpu_STRUCT_ZERO_INIT", startLine: 75, endLine: 75 },
    MacroAuthority { name: "_wgpu_MAKE_INIT_STRUCT", startLine: 77, endLine: 77 },
    MacroAuthority { name: "_wgpu_MAKE_INIT_STRUCT", startLine: 79, endLine: 79 },
    MacroAuthority { name: "_wgpu_ENUM_ZERO_INIT", startLine: 82, endLine: 82 },
    MacroAuthority { name: "_wgpu_STRUCT_ZERO_INIT", startLine: 83, endLine: 83 },
    MacroAuthority { name: "_wgpu_MAKE_INIT_STRUCT", startLine: 85, endLine: 85 },
    MacroAuthority { name: "_wgpu_MAKE_INIT_STRUCT", startLine: 87, endLine: 87 },
    MacroAuthority { name: "WGPU_TRUE", startLine: 91, endLine: 91 },
    MacroAuthority { name: "WGPU_FALSE", startLine: 92, endLine: 92 },
    MacroAuthority { name: "WGPU_ARRAY_LAYER_COUNT_UNDEFINED", startLine: 93, endLine: 93 },
    MacroAuthority { name: "WGPU_COPY_STRIDE_UNDEFINED", startLine: 94, endLine: 94 },
    MacroAuthority { name: "WGPU_DEPTH_CLEAR_VALUE_UNDEFINED", startLine: 95, endLine: 95 },
    MacroAuthority { name: "WGPU_DEPTH_SLICE_UNDEFINED", startLine: 96, endLine: 96 },
    MacroAuthority { name: "WGPU_LIMIT_U32_UNDEFINED", startLine: 97, endLine: 97 },
    MacroAuthority { name: "WGPU_LIMIT_U64_UNDEFINED", startLine: 98, endLine: 98 },
    MacroAuthority { name: "WGPU_MIP_LEVEL_COUNT_UNDEFINED", startLine: 99, endLine: 99 },
    MacroAuthority { name: "WGPU_QUERY_SET_INDEX_UNDEFINED", startLine: 100, endLine: 100 },
    MacroAuthority { name: "WGPU_STRLEN", startLine: 101, endLine: 101 },
    MacroAuthority { name: "WGPU_WHOLE_MAP_SIZE", startLine: 102, endLine: 102 },
    MacroAuthority { name: "WGPU_WHOLE_SIZE", startLine: 103, endLine: 103 },
    MacroAuthority { name: "WGPU_STRING_VIEW_INIT", startLine: 110, endLine: 113 },
    MacroAuthority { name: "WGPU_BUFFER_MAP_CALLBACK_INFO_INIT", startLine: 927, endLine: 933 },
    MacroAuthority { name: "WGPU_COMPILATION_INFO_CALLBACK_INFO_INIT", startLine: 943, endLine: 949 },
    MacroAuthority { name: "WGPU_CREATE_COMPUTE_PIPELINE_ASYNC_CALLBACK_INFO_INIT", startLine: 959, endLine: 965 },
    MacroAuthority { name: "WGPU_CREATE_RENDER_PIPELINE_ASYNC_CALLBACK_INFO_INIT", startLine: 975, endLine: 981 },
    MacroAuthority { name: "WGPU_DEVICE_LOST_CALLBACK_INFO_INIT", startLine: 991, endLine: 997 },
    MacroAuthority { name: "WGPU_POP_ERROR_SCOPE_CALLBACK_INFO_INIT", startLine: 1007, endLine: 1013 },
    MacroAuthority { name: "WGPU_QUEUE_WORK_DONE_CALLBACK_INFO_INIT", startLine: 1023, endLine: 1029 },
    MacroAuthority { name: "WGPU_REQUEST_ADAPTER_CALLBACK_INFO_INIT", startLine: 1039, endLine: 1045 },
    MacroAuthority { name: "WGPU_REQUEST_DEVICE_CALLBACK_INFO_INIT", startLine: 1055, endLine: 1061 },
    MacroAuthority { name: "WGPU_UNCAPTURED_ERROR_CALLBACK_INFO_INIT", startLine: 1070, endLine: 1075 },
    MacroAuthority { name: "WGPU_ADAPTER_INFO_INIT", startLine: 1091, endLine: 1103 },
    MacroAuthority { name: "WGPU_BIND_GROUP_ENTRY_INIT", startLine: 1115, endLine: 1123 },
    MacroAuthority { name: "WGPU_BLEND_COMPONENT_INIT", startLine: 1131, endLine: 1135 },
    MacroAuthority { name: "WGPU_BUFFER_BINDING_LAYOUT_INIT", startLine: 1144, endLine: 1149 },
    MacroAuthority { name: "WGPU_BUFFER_DESCRIPTOR_INIT", startLine: 1159, endLine: 1165 },
    MacroAuthority { name: "WGPU_COLOR_INIT", startLine: 1174, endLine: 1179 },
    MacroAuthority { name: "WGPU_COMMAND_BUFFER_DESCRIPTOR_INIT", startLine: 1186, endLine: 1189 },
    MacroAuthority { name: "WGPU_COMMAND_ENCODER_DESCRIPTOR_INIT", startLine: 1196, endLine: 1199 },
    MacroAuthority { name: "WGPU_COMPATIBILITY_MODE_LIMITS_INIT", startLine: 1210, endLine: 1219 },
    MacroAuthority { name: "WGPU_CONSTANT_ENTRY_INIT", startLine: 1227, endLine: 1231 },
    MacroAuthority { name: "WGPU_DAWN_COMPILATION_MESSAGE_UTF16_INIT", startLine: 1241, endLine: 1249 },
    MacroAuthority { name: "WGPU_EMSCRIPTEN_SURFACE_SOURCE_CANVAS_HTML_SELECTOR_INIT", startLine: 1257, endLine: 1263 },
    MacroAuthority { name: "WGPU_EXTENT_3D_INIT", startLine: 1271, endLine: 1275 },
    MacroAuthority { name: "WGPU_FUTURE_INIT", startLine: 1281, endLine: 1283 },
    MacroAuthority { name: "WGPU_INSTANCE_LIMITS_INIT", startLine: 1290, endLine: 1293 },
    MacroAuthority { name: "WGPU_INTERNAL_HAVE_EMDAWNWEBGPU_HEADER_INIT", startLine: 1299, endLine: 1301 },
    MacroAuthority { name: "WGPU_MULTISAMPLE_STATE_INIT", startLine: 1310, endLine: 1315 },
    MacroAuthority { name: "WGPU_ORIGIN_3D_INIT", startLine: 1323, endLine: 1327 },
    MacroAuthority { name: "WGPU_PASS_TIMESTAMP_WRITES_INIT", startLine: 1336, endLine: 1341 },
    MacroAuthority { name: "WGPU_PIPELINE_LAYOUT_DESCRIPTOR_INIT", startLine: 1351, endLine: 1357 },
    MacroAuthority { name: "WGPU_PRIMITIVE_STATE_INIT", startLine: 1368, endLine: 1375 },
    MacroAuthority { name: "WGPU_QUERY_SET_DESCRIPTOR_INIT", startLine: 1384, endLine: 1389 },
    MacroAuthority { name: "WGPU_QUEUE_DESCRIPTOR_INIT", startLine: 1396, endLine: 1399 },
    MacroAuthority { name: "WGPU_RENDER_BUNDLE_DESCRIPTOR_INIT", startLine: 1406, endLine: 1409 },
    MacroAuthority { name: "WGPU_RENDER_BUNDLE_ENCODER_DESCRIPTOR_INIT", startLine: 1422, endLine: 1431 },
    MacroAuthority { name: "WGPU_RENDER_PASS_DEPTH_STENCIL_ATTACHMENT_INIT", startLine: 1446, endLine: 1457 },
    MacroAuthority { name: "WGPU_RENDER_PASS_MAX_DRAW_COUNT_INIT", startLine: 1465, endLine: 1471 },
    MacroAuthority { name: "WGPU_REQUEST_ADAPTER_WEBXR_OPTIONS_INIT", startLine: 1479, endLine: 1485 },
    MacroAuthority { name: "WGPU_SAMPLER_BINDING_LAYOUT_INIT", startLine: 1492, endLine: 1495 },
    MacroAuthority { name: "WGPU_SAMPLER_DESCRIPTOR_INIT", startLine: 1512, endLine: 1525 },
    MacroAuthority { name: "WGPU_SHADER_SOURCE_SPIRV_INIT", startLine: 1534, endLine: 1541 },
    MacroAuthority { name: "WGPU_SHADER_SOURCE_WGSL_INIT", startLine: 1549, endLine: 1555 },
    MacroAuthority { name: "WGPU_STENCIL_FACE_STATE_INIT", startLine: 1564, endLine: 1569 },
    MacroAuthority { name: "WGPU_STORAGE_TEXTURE_BINDING_LAYOUT_INIT", startLine: 1578, endLine: 1583 },
    MacroAuthority { name: "WGPU_SUPPORTED_FEATURES_INIT", startLine: 1590, endLine: 1593 },
    MacroAuthority { name: "WGPU_SUPPORTED_INSTANCE_FEATURES_INIT", startLine: 1600, endLine: 1603 },
    MacroAuthority { name: "WGPU_SUPPORTED_WGSL_LANGUAGE_FEATURES_INIT", startLine: 1610, endLine: 1613 },
    MacroAuthority { name: "WGPU_SURFACE_CAPABILITIES_INIT", startLine: 1626, endLine: 1635 },
    MacroAuthority { name: "WGPU_SURFACE_COLOR_MANAGEMENT_INIT", startLine: 1644, endLine: 1651 },
    MacroAuthority { name: "WGPU_SURFACE_CONFIGURATION_INIT", startLine: 1666, endLine: 1677 },
    MacroAuthority { name: "WGPU_SURFACE_TEXTURE_INIT", startLine: 1685, endLine: 1689 },
    MacroAuthority { name: "WGPU_TEXEL_COPY_BUFFER_LAYOUT_INIT", startLine: 1697, endLine: 1701 },
    MacroAuthority { name: "WGPU_TEXTURE_BINDING_LAYOUT_INIT", startLine: 1710, endLine: 1715 },
    MacroAuthority { name: "WGPU_TEXTURE_BINDING_VIEW_DIMENSION_DESCRIPTOR_INIT", startLine: 1723, endLine: 1729 },
    MacroAuthority { name: "WGPU_TEXTURE_COMPONENT_SWIZZLE_INIT", startLine: 1738, endLine: 1743 },
    MacroAuthority { name: "WGPU_VERTEX_ATTRIBUTE_INIT", startLine: 1752, endLine: 1757 },
    MacroAuthority { name: "WGPU_BIND_GROUP_DESCRIPTOR_INIT", startLine: 1767, endLine: 1773 },
    MacroAuthority { name: "WGPU_BIND_GROUP_LAYOUT_ENTRY_INIT", startLine: 1786, endLine: 1795 },
    MacroAuthority { name: "WGPU_BLEND_STATE_INIT", startLine: 1802, endLine: 1805 },
    MacroAuthority { name: "WGPU_COMPILATION_MESSAGE_INIT", startLine: 1817, endLine: 1825 },
    MacroAuthority { name: "WGPU_COMPUTE_PASS_DESCRIPTOR_INIT", startLine: 1833, endLine: 1837 },
    MacroAuthority { name: "WGPU_COMPUTE_STATE_INIT", startLine: 1847, endLine: 1853 },
    MacroAuthority { name: "WGPU_DEPTH_STENCIL_STATE_INIT", startLine: 1869, endLine: 1881 },
    MacroAuthority { name: "WGPU_FUTURE_WAIT_INFO_INIT", startLine: 1888, endLine: 1891 },
    MacroAuthority { name: "WGPU_INSTANCE_DESCRIPTOR_INIT", startLine: 1900, endLine: 1905 },
    MacroAuthority { name: "WGPU_LIMITS_INIT", startLine: 1943, endLine: 1977 },
    MacroAuthority { name: "WGPU_RENDER_PASS_COLOR_ATTACHMENT_INIT", startLine: 1989, endLine: 1997 },
    MacroAuthority { name: "WGPU_REQUEST_ADAPTER_OPTIONS_INIT", startLine: 2008, endLine: 2015 },
    MacroAuthority { name: "WGPU_SHADER_MODULE_DESCRIPTOR_INIT", startLine: 2022, endLine: 2025 },
    MacroAuthority { name: "WGPU_SURFACE_DESCRIPTOR_INIT", startLine: 2032, endLine: 2035 },
    MacroAuthority { name: "WGPU_TEXEL_COPY_BUFFER_INFO_INIT", startLine: 2042, endLine: 2045 },
    MacroAuthority { name: "WGPU_TEXEL_COPY_TEXTURE_INFO_INIT", startLine: 2054, endLine: 2059 },
    MacroAuthority { name: "WGPU_TEXTURE_COMPONENT_SWIZZLE_DESCRIPTOR_INIT", startLine: 2067, endLine: 2073 },
    MacroAuthority { name: "WGPU_TEXTURE_DESCRIPTOR_INIT", startLine: 2088, endLine: 2099 },
    MacroAuthority { name: "WGPU_VERTEX_BUFFER_LAYOUT_INIT", startLine: 2109, endLine: 2115 },
    MacroAuthority { name: "WGPU_BIND_GROUP_LAYOUT_DESCRIPTOR_INIT", startLine: 2124, endLine: 2129 },
    MacroAuthority { name: "WGPU_COLOR_TARGET_STATE_INIT", startLine: 2138, endLine: 2143 },
    MacroAuthority { name: "WGPU_COMPILATION_INFO_INIT", startLine: 2151, endLine: 2155 },
    MacroAuthority { name: "WGPU_COMPUTE_PIPELINE_DESCRIPTOR_INIT", startLine: 2164, endLine: 2169 },
    MacroAuthority { name: "WGPU_DEVICE_DESCRIPTOR_INIT", startLine: 2182, endLine: 2191 },
    MacroAuthority { name: "WGPU_RENDER_PASS_DESCRIPTOR_INIT", startLine: 2203, endLine: 2211 },
    MacroAuthority { name: "WGPU_TEXTURE_VIEW_DESCRIPTOR_INIT", startLine: 2226, endLine: 2237 },
    MacroAuthority { name: "WGPU_VERTEX_STATE_INIT", startLine: 2249, endLine: 2257 },
    MacroAuthority { name: "WGPU_FRAGMENT_STATE_INIT", startLine: 2269, endLine: 2277 },
    MacroAuthority { name: "WGPU_RENDER_PIPELINE_DESCRIPTOR_INIT", startLine: 2290, endLine: 2299 },
];

pub(crate) fn macroSource(authority: MacroAuthority) -> String {
    PINNED_SOURCE
        .lines()
        .skip(authority.startLine - 1)
        .take(authority.endLine - authority.startLine + 1)
        .collect::<Vec<_>>()
        .join("\n")
}

pub(crate) const WGPU_TRUE: u32 = 1;
pub(crate) const WGPU_FALSE: u32 = 0;
pub(crate) const WGPU_ARRAY_LAYER_COUNT_UNDEFINED: u32 = u32::MAX;
pub(crate) const WGPU_COPY_STRIDE_UNDEFINED: u32 = u32::MAX;
pub(crate) const WGPU_DEPTH_CLEAR_VALUE_UNDEFINED: f64 = f64::NAN;
pub(crate) const WGPU_DEPTH_SLICE_UNDEFINED: u32 = u32::MAX;
pub(crate) const WGPU_LIMIT_U32_UNDEFINED: u32 = u32::MAX;
pub(crate) const WGPU_LIMIT_U64_UNDEFINED: u64 = u64::MAX;
pub(crate) const WGPU_MIP_LEVEL_COUNT_UNDEFINED: u32 = u32::MAX;
pub(crate) const WGPU_QUERY_SET_INDEX_UNDEFINED: u32 = u32::MAX;
pub(crate) const WGPU_STRLEN: usize = usize::MAX;
pub(crate) const WGPU_WHOLE_MAP_SIZE: usize = usize::MAX;
pub(crate) const WGPU_WHOLE_SIZE: u64 = u64::MAX;

#[repr(C)]
pub(crate) struct WGPUAdapterImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUBindGroupImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUBindGroupLayoutImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUBufferImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUCommandBufferImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUCommandEncoderImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUComputePassEncoderImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUComputePipelineImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUDeviceImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUInstanceImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUPipelineLayoutImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUQuerySetImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUQueueImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPURenderBundleEncoderImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPURenderBundleImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPURenderPassEncoderImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPURenderPipelineImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUSamplerImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUShaderModuleImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUSurfaceImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUTextureImpl { _private: [u8; 0] }
#[repr(C)]
pub(crate) struct WGPUTextureViewImpl { _private: [u8; 0] }

pub(crate) type WGPUAdapter = *mut WGPUAdapterImpl;
pub(crate) type WGPUBindGroup = *mut WGPUBindGroupImpl;
pub(crate) type WGPUBindGroupLayout = *mut WGPUBindGroupLayoutImpl;
pub(crate) type WGPUBuffer = *mut WGPUBufferImpl;
pub(crate) type WGPUCommandBuffer = *mut WGPUCommandBufferImpl;
pub(crate) type WGPUCommandEncoder = *mut WGPUCommandEncoderImpl;
pub(crate) type WGPUComputePassEncoder = *mut WGPUComputePassEncoderImpl;
pub(crate) type WGPUComputePipeline = *mut WGPUComputePipelineImpl;
pub(crate) type WGPUDevice = *mut WGPUDeviceImpl;
pub(crate) type WGPUInstance = *mut WGPUInstanceImpl;
pub(crate) type WGPUPipelineLayout = *mut WGPUPipelineLayoutImpl;
pub(crate) type WGPUQuerySet = *mut WGPUQuerySetImpl;
pub(crate) type WGPUQueue = *mut WGPUQueueImpl;
pub(crate) type WGPURenderBundle = *mut WGPURenderBundleImpl;
pub(crate) type WGPURenderBundleEncoder = *mut WGPURenderBundleEncoderImpl;
pub(crate) type WGPURenderPassEncoder = *mut WGPURenderPassEncoderImpl;
pub(crate) type WGPURenderPipeline = *mut WGPURenderPipelineImpl;
pub(crate) type WGPUSampler = *mut WGPUSamplerImpl;
pub(crate) type WGPUShaderModule = *mut WGPUShaderModuleImpl;
pub(crate) type WGPUSurface = *mut WGPUSurfaceImpl;
pub(crate) type WGPUTexture = *mut WGPUTextureImpl;
pub(crate) type WGPUTextureView = *mut WGPUTextureViewImpl;

pub(crate) type WGPUAdapterType = i32;
pub(crate) const WGPUAdapterType_DiscreteGPU: WGPUAdapterType = 1;
pub(crate) const WGPUAdapterType_IntegratedGPU: WGPUAdapterType = 2;
pub(crate) const WGPUAdapterType_CPU: WGPUAdapterType = 3;
pub(crate) const WGPUAdapterType_Unknown: WGPUAdapterType = 4;
pub(crate) const WGPUAdapterType_Force32: WGPUAdapterType = 2147483647;

pub(crate) type WGPUAddressMode = i32;
pub(crate) const WGPUAddressMode_Undefined: WGPUAddressMode = 0;
pub(crate) const WGPUAddressMode_ClampToEdge: WGPUAddressMode = 1;
pub(crate) const WGPUAddressMode_Repeat: WGPUAddressMode = 2;
pub(crate) const WGPUAddressMode_MirrorRepeat: WGPUAddressMode = 3;
pub(crate) const WGPUAddressMode_Force32: WGPUAddressMode = 2147483647;

pub(crate) type WGPUBackendType = i32;
pub(crate) const WGPUBackendType_Undefined: WGPUBackendType = 0;
pub(crate) const WGPUBackendType_Null: WGPUBackendType = 1;
pub(crate) const WGPUBackendType_WebGPU: WGPUBackendType = 2;
pub(crate) const WGPUBackendType_D3D11: WGPUBackendType = 3;
pub(crate) const WGPUBackendType_D3D12: WGPUBackendType = 4;
pub(crate) const WGPUBackendType_Metal: WGPUBackendType = 5;
pub(crate) const WGPUBackendType_Vulkan: WGPUBackendType = 6;
pub(crate) const WGPUBackendType_OpenGL: WGPUBackendType = 7;
pub(crate) const WGPUBackendType_OpenGLES: WGPUBackendType = 8;
pub(crate) const WGPUBackendType_Force32: WGPUBackendType = 2147483647;

pub(crate) type WGPUBlendFactor = i32;
pub(crate) const WGPUBlendFactor_Undefined: WGPUBlendFactor = 0;
pub(crate) const WGPUBlendFactor_Zero: WGPUBlendFactor = 1;
pub(crate) const WGPUBlendFactor_One: WGPUBlendFactor = 2;
pub(crate) const WGPUBlendFactor_Src: WGPUBlendFactor = 3;
pub(crate) const WGPUBlendFactor_OneMinusSrc: WGPUBlendFactor = 4;
pub(crate) const WGPUBlendFactor_SrcAlpha: WGPUBlendFactor = 5;
pub(crate) const WGPUBlendFactor_OneMinusSrcAlpha: WGPUBlendFactor = 6;
pub(crate) const WGPUBlendFactor_Dst: WGPUBlendFactor = 7;
pub(crate) const WGPUBlendFactor_OneMinusDst: WGPUBlendFactor = 8;
pub(crate) const WGPUBlendFactor_DstAlpha: WGPUBlendFactor = 9;
pub(crate) const WGPUBlendFactor_OneMinusDstAlpha: WGPUBlendFactor = 10;
pub(crate) const WGPUBlendFactor_SrcAlphaSaturated: WGPUBlendFactor = 11;
pub(crate) const WGPUBlendFactor_Constant: WGPUBlendFactor = 12;
pub(crate) const WGPUBlendFactor_OneMinusConstant: WGPUBlendFactor = 13;
pub(crate) const WGPUBlendFactor_Src1: WGPUBlendFactor = 14;
pub(crate) const WGPUBlendFactor_OneMinusSrc1: WGPUBlendFactor = 15;
pub(crate) const WGPUBlendFactor_Src1Alpha: WGPUBlendFactor = 16;
pub(crate) const WGPUBlendFactor_OneMinusSrc1Alpha: WGPUBlendFactor = 17;
pub(crate) const WGPUBlendFactor_Force32: WGPUBlendFactor = 2147483647;

pub(crate) type WGPUBlendOperation = i32;
pub(crate) const WGPUBlendOperation_Undefined: WGPUBlendOperation = 0;
pub(crate) const WGPUBlendOperation_Add: WGPUBlendOperation = 1;
pub(crate) const WGPUBlendOperation_Subtract: WGPUBlendOperation = 2;
pub(crate) const WGPUBlendOperation_ReverseSubtract: WGPUBlendOperation = 3;
pub(crate) const WGPUBlendOperation_Min: WGPUBlendOperation = 4;
pub(crate) const WGPUBlendOperation_Max: WGPUBlendOperation = 5;
pub(crate) const WGPUBlendOperation_Force32: WGPUBlendOperation = 2147483647;

pub(crate) type WGPUBufferBindingType = i32;
pub(crate) const WGPUBufferBindingType_BindingNotUsed: WGPUBufferBindingType = 0;
pub(crate) const WGPUBufferBindingType_Undefined: WGPUBufferBindingType = 1;
pub(crate) const WGPUBufferBindingType_Uniform: WGPUBufferBindingType = 2;
pub(crate) const WGPUBufferBindingType_Storage: WGPUBufferBindingType = 3;
pub(crate) const WGPUBufferBindingType_ReadOnlyStorage: WGPUBufferBindingType = 4;
pub(crate) const WGPUBufferBindingType_Force32: WGPUBufferBindingType = 2147483647;

pub(crate) type WGPUBufferMapState = i32;
pub(crate) const WGPUBufferMapState_Unmapped: WGPUBufferMapState = 1;
pub(crate) const WGPUBufferMapState_Pending: WGPUBufferMapState = 2;
pub(crate) const WGPUBufferMapState_Mapped: WGPUBufferMapState = 3;
pub(crate) const WGPUBufferMapState_Force32: WGPUBufferMapState = 2147483647;

pub(crate) type WGPUCallbackMode = i32;
pub(crate) const WGPUCallbackMode_WaitAnyOnly: WGPUCallbackMode = 1;
pub(crate) const WGPUCallbackMode_AllowProcessEvents: WGPUCallbackMode = 2;
pub(crate) const WGPUCallbackMode_AllowSpontaneous: WGPUCallbackMode = 3;
pub(crate) const WGPUCallbackMode_Force32: WGPUCallbackMode = 2147483647;

pub(crate) type WGPUCompareFunction = i32;
pub(crate) const WGPUCompareFunction_Undefined: WGPUCompareFunction = 0;
pub(crate) const WGPUCompareFunction_Never: WGPUCompareFunction = 1;
pub(crate) const WGPUCompareFunction_Less: WGPUCompareFunction = 2;
pub(crate) const WGPUCompareFunction_Equal: WGPUCompareFunction = 3;
pub(crate) const WGPUCompareFunction_LessEqual: WGPUCompareFunction = 4;
pub(crate) const WGPUCompareFunction_Greater: WGPUCompareFunction = 5;
pub(crate) const WGPUCompareFunction_NotEqual: WGPUCompareFunction = 6;
pub(crate) const WGPUCompareFunction_GreaterEqual: WGPUCompareFunction = 7;
pub(crate) const WGPUCompareFunction_Always: WGPUCompareFunction = 8;
pub(crate) const WGPUCompareFunction_Force32: WGPUCompareFunction = 2147483647;

pub(crate) type WGPUCompilationInfoRequestStatus = i32;
pub(crate) const WGPUCompilationInfoRequestStatus_Success: WGPUCompilationInfoRequestStatus = 1;
pub(crate) const WGPUCompilationInfoRequestStatus_CallbackCancelled: WGPUCompilationInfoRequestStatus = 2;
pub(crate) const WGPUCompilationInfoRequestStatus_Force32: WGPUCompilationInfoRequestStatus = 2147483647;

pub(crate) type WGPUCompilationMessageType = i32;
pub(crate) const WGPUCompilationMessageType_Error: WGPUCompilationMessageType = 1;
pub(crate) const WGPUCompilationMessageType_Warning: WGPUCompilationMessageType = 2;
pub(crate) const WGPUCompilationMessageType_Info: WGPUCompilationMessageType = 3;
pub(crate) const WGPUCompilationMessageType_Force32: WGPUCompilationMessageType = 2147483647;

pub(crate) type WGPUComponentSwizzle = i32;
pub(crate) const WGPUComponentSwizzle_Undefined: WGPUComponentSwizzle = 0;
pub(crate) const WGPUComponentSwizzle_Zero: WGPUComponentSwizzle = 1;
pub(crate) const WGPUComponentSwizzle_One: WGPUComponentSwizzle = 2;
pub(crate) const WGPUComponentSwizzle_R: WGPUComponentSwizzle = 3;
pub(crate) const WGPUComponentSwizzle_G: WGPUComponentSwizzle = 4;
pub(crate) const WGPUComponentSwizzle_B: WGPUComponentSwizzle = 5;
pub(crate) const WGPUComponentSwizzle_A: WGPUComponentSwizzle = 6;
pub(crate) const WGPUComponentSwizzle_Force32: WGPUComponentSwizzle = 2147483647;

pub(crate) type WGPUCompositeAlphaMode = i32;
pub(crate) const WGPUCompositeAlphaMode_Auto: WGPUCompositeAlphaMode = 0;
pub(crate) const WGPUCompositeAlphaMode_Opaque: WGPUCompositeAlphaMode = 1;
pub(crate) const WGPUCompositeAlphaMode_Premultiplied: WGPUCompositeAlphaMode = 2;
pub(crate) const WGPUCompositeAlphaMode_Unpremultiplied: WGPUCompositeAlphaMode = 3;
pub(crate) const WGPUCompositeAlphaMode_Inherit: WGPUCompositeAlphaMode = 4;
pub(crate) const WGPUCompositeAlphaMode_Force32: WGPUCompositeAlphaMode = 2147483647;

pub(crate) type WGPUCreatePipelineAsyncStatus = i32;
pub(crate) const WGPUCreatePipelineAsyncStatus_Success: WGPUCreatePipelineAsyncStatus = 1;
pub(crate) const WGPUCreatePipelineAsyncStatus_CallbackCancelled: WGPUCreatePipelineAsyncStatus = 2;
pub(crate) const WGPUCreatePipelineAsyncStatus_ValidationError: WGPUCreatePipelineAsyncStatus = 3;
pub(crate) const WGPUCreatePipelineAsyncStatus_InternalError: WGPUCreatePipelineAsyncStatus = 4;
pub(crate) const WGPUCreatePipelineAsyncStatus_Force32: WGPUCreatePipelineAsyncStatus = 2147483647;

pub(crate) type WGPUCullMode = i32;
pub(crate) const WGPUCullMode_Undefined: WGPUCullMode = 0;
pub(crate) const WGPUCullMode_None: WGPUCullMode = 1;
pub(crate) const WGPUCullMode_Front: WGPUCullMode = 2;
pub(crate) const WGPUCullMode_Back: WGPUCullMode = 3;
pub(crate) const WGPUCullMode_Force32: WGPUCullMode = 2147483647;

pub(crate) type WGPUDeviceLostReason = i32;
pub(crate) const WGPUDeviceLostReason_Unknown: WGPUDeviceLostReason = 1;
pub(crate) const WGPUDeviceLostReason_Destroyed: WGPUDeviceLostReason = 2;
pub(crate) const WGPUDeviceLostReason_CallbackCancelled: WGPUDeviceLostReason = 3;
pub(crate) const WGPUDeviceLostReason_FailedCreation: WGPUDeviceLostReason = 4;
pub(crate) const WGPUDeviceLostReason_Force32: WGPUDeviceLostReason = 2147483647;

pub(crate) type WGPUErrorFilter = i32;
pub(crate) const WGPUErrorFilter_Validation: WGPUErrorFilter = 1;
pub(crate) const WGPUErrorFilter_OutOfMemory: WGPUErrorFilter = 2;
pub(crate) const WGPUErrorFilter_Internal: WGPUErrorFilter = 3;
pub(crate) const WGPUErrorFilter_Force32: WGPUErrorFilter = 2147483647;

pub(crate) type WGPUErrorType = i32;
pub(crate) const WGPUErrorType_NoError: WGPUErrorType = 1;
pub(crate) const WGPUErrorType_Validation: WGPUErrorType = 2;
pub(crate) const WGPUErrorType_OutOfMemory: WGPUErrorType = 3;
pub(crate) const WGPUErrorType_Internal: WGPUErrorType = 4;
pub(crate) const WGPUErrorType_Unknown: WGPUErrorType = 5;
pub(crate) const WGPUErrorType_Force32: WGPUErrorType = 2147483647;

pub(crate) type WGPUFeatureLevel = i32;
pub(crate) const WGPUFeatureLevel_Undefined: WGPUFeatureLevel = 0;
pub(crate) const WGPUFeatureLevel_Compatibility: WGPUFeatureLevel = 1;
pub(crate) const WGPUFeatureLevel_Core: WGPUFeatureLevel = 2;
pub(crate) const WGPUFeatureLevel_Force32: WGPUFeatureLevel = 2147483647;

pub(crate) type WGPUFeatureName = i32;
pub(crate) const WGPUFeatureName_CoreFeaturesAndLimits: WGPUFeatureName = 1;
pub(crate) const WGPUFeatureName_DepthClipControl: WGPUFeatureName = 2;
pub(crate) const WGPUFeatureName_Depth32FloatStencil8: WGPUFeatureName = 3;
pub(crate) const WGPUFeatureName_TextureCompressionBC: WGPUFeatureName = 4;
pub(crate) const WGPUFeatureName_TextureCompressionBCSliced3D: WGPUFeatureName = 5;
pub(crate) const WGPUFeatureName_TextureCompressionETC2: WGPUFeatureName = 6;
pub(crate) const WGPUFeatureName_TextureCompressionASTC: WGPUFeatureName = 7;
pub(crate) const WGPUFeatureName_TextureCompressionASTCSliced3D: WGPUFeatureName = 8;
pub(crate) const WGPUFeatureName_TimestampQuery: WGPUFeatureName = 9;
pub(crate) const WGPUFeatureName_IndirectFirstInstance: WGPUFeatureName = 10;
pub(crate) const WGPUFeatureName_ShaderF16: WGPUFeatureName = 11;
pub(crate) const WGPUFeatureName_RG11B10UfloatRenderable: WGPUFeatureName = 12;
pub(crate) const WGPUFeatureName_BGRA8UnormStorage: WGPUFeatureName = 13;
pub(crate) const WGPUFeatureName_Float32Filterable: WGPUFeatureName = 14;
pub(crate) const WGPUFeatureName_Float32Blendable: WGPUFeatureName = 15;
pub(crate) const WGPUFeatureName_ClipDistances: WGPUFeatureName = 16;
pub(crate) const WGPUFeatureName_DualSourceBlending: WGPUFeatureName = 17;
pub(crate) const WGPUFeatureName_Subgroups: WGPUFeatureName = 18;
pub(crate) const WGPUFeatureName_TextureFormatsTier1: WGPUFeatureName = 19;
pub(crate) const WGPUFeatureName_TextureFormatsTier2: WGPUFeatureName = 20;
pub(crate) const WGPUFeatureName_PrimitiveIndex: WGPUFeatureName = 21;
pub(crate) const WGPUFeatureName_TextureComponentSwizzle: WGPUFeatureName = 22;
pub(crate) const WGPUFeatureName_Unorm16TextureFormats: WGPUFeatureName = 327692;
pub(crate) const WGPUFeatureName_Snorm16TextureFormats: WGPUFeatureName = 327693;
pub(crate) const WGPUFeatureName_MultiDrawIndirect: WGPUFeatureName = 327732;
pub(crate) const WGPUFeatureName_Force32: WGPUFeatureName = 2147483647;

pub(crate) type WGPUFilterMode = i32;
pub(crate) const WGPUFilterMode_Undefined: WGPUFilterMode = 0;
pub(crate) const WGPUFilterMode_Nearest: WGPUFilterMode = 1;
pub(crate) const WGPUFilterMode_Linear: WGPUFilterMode = 2;
pub(crate) const WGPUFilterMode_Force32: WGPUFilterMode = 2147483647;

pub(crate) type WGPUFrontFace = i32;
pub(crate) const WGPUFrontFace_Undefined: WGPUFrontFace = 0;
pub(crate) const WGPUFrontFace_CCW: WGPUFrontFace = 1;
pub(crate) const WGPUFrontFace_CW: WGPUFrontFace = 2;
pub(crate) const WGPUFrontFace_Force32: WGPUFrontFace = 2147483647;

pub(crate) type WGPUIndexFormat = i32;
pub(crate) const WGPUIndexFormat_Undefined: WGPUIndexFormat = 0;
pub(crate) const WGPUIndexFormat_Uint16: WGPUIndexFormat = 1;
pub(crate) const WGPUIndexFormat_Uint32: WGPUIndexFormat = 2;
pub(crate) const WGPUIndexFormat_Force32: WGPUIndexFormat = 2147483647;

pub(crate) type WGPUInstanceFeatureName = i32;
pub(crate) const WGPUInstanceFeatureName_TimedWaitAny: WGPUInstanceFeatureName = 1;
pub(crate) const WGPUInstanceFeatureName_ShaderSourceSPIRV: WGPUInstanceFeatureName = 2;
pub(crate) const WGPUInstanceFeatureName_MultipleDevicesPerAdapter: WGPUInstanceFeatureName = 3;
pub(crate) const WGPUInstanceFeatureName_Force32: WGPUInstanceFeatureName = 2147483647;

pub(crate) type WGPULoadOp = i32;
pub(crate) const WGPULoadOp_Undefined: WGPULoadOp = 0;
pub(crate) const WGPULoadOp_Load: WGPULoadOp = 1;
pub(crate) const WGPULoadOp_Clear: WGPULoadOp = 2;
pub(crate) const WGPULoadOp_Force32: WGPULoadOp = 2147483647;

pub(crate) type WGPUMapAsyncStatus = i32;
pub(crate) const WGPUMapAsyncStatus_Success: WGPUMapAsyncStatus = 1;
pub(crate) const WGPUMapAsyncStatus_CallbackCancelled: WGPUMapAsyncStatus = 2;
pub(crate) const WGPUMapAsyncStatus_Error: WGPUMapAsyncStatus = 3;
pub(crate) const WGPUMapAsyncStatus_Aborted: WGPUMapAsyncStatus = 4;
pub(crate) const WGPUMapAsyncStatus_Force32: WGPUMapAsyncStatus = 2147483647;

pub(crate) type WGPUMipmapFilterMode = i32;
pub(crate) const WGPUMipmapFilterMode_Undefined: WGPUMipmapFilterMode = 0;
pub(crate) const WGPUMipmapFilterMode_Nearest: WGPUMipmapFilterMode = 1;
pub(crate) const WGPUMipmapFilterMode_Linear: WGPUMipmapFilterMode = 2;
pub(crate) const WGPUMipmapFilterMode_Force32: WGPUMipmapFilterMode = 2147483647;

pub(crate) type WGPUOptionalBool = i32;
pub(crate) const WGPUOptionalBool_False: WGPUOptionalBool = 0;
pub(crate) const WGPUOptionalBool_True: WGPUOptionalBool = 1;
pub(crate) const WGPUOptionalBool_Undefined: WGPUOptionalBool = 2;
pub(crate) const WGPUOptionalBool_Force32: WGPUOptionalBool = 2147483647;

pub(crate) type WGPUPopErrorScopeStatus = i32;
pub(crate) const WGPUPopErrorScopeStatus_Success: WGPUPopErrorScopeStatus = 1;
pub(crate) const WGPUPopErrorScopeStatus_CallbackCancelled: WGPUPopErrorScopeStatus = 2;
pub(crate) const WGPUPopErrorScopeStatus_Error: WGPUPopErrorScopeStatus = 3;
pub(crate) const WGPUPopErrorScopeStatus_Force32: WGPUPopErrorScopeStatus = 2147483647;

pub(crate) type WGPUPowerPreference = i32;
pub(crate) const WGPUPowerPreference_Undefined: WGPUPowerPreference = 0;
pub(crate) const WGPUPowerPreference_LowPower: WGPUPowerPreference = 1;
pub(crate) const WGPUPowerPreference_HighPerformance: WGPUPowerPreference = 2;
pub(crate) const WGPUPowerPreference_Force32: WGPUPowerPreference = 2147483647;

pub(crate) type WGPUPredefinedColorSpace = i32;
pub(crate) const WGPUPredefinedColorSpace_SRGB: WGPUPredefinedColorSpace = 1;
pub(crate) const WGPUPredefinedColorSpace_DisplayP3: WGPUPredefinedColorSpace = 2;
pub(crate) const WGPUPredefinedColorSpace_Force32: WGPUPredefinedColorSpace = 2147483647;

pub(crate) type WGPUPresentMode = i32;
pub(crate) const WGPUPresentMode_Undefined: WGPUPresentMode = 0;
pub(crate) const WGPUPresentMode_Fifo: WGPUPresentMode = 1;
pub(crate) const WGPUPresentMode_FifoRelaxed: WGPUPresentMode = 2;
pub(crate) const WGPUPresentMode_Immediate: WGPUPresentMode = 3;
pub(crate) const WGPUPresentMode_Mailbox: WGPUPresentMode = 4;
pub(crate) const WGPUPresentMode_Force32: WGPUPresentMode = 2147483647;

pub(crate) type WGPUPrimitiveTopology = i32;
pub(crate) const WGPUPrimitiveTopology_Undefined: WGPUPrimitiveTopology = 0;
pub(crate) const WGPUPrimitiveTopology_PointList: WGPUPrimitiveTopology = 1;
pub(crate) const WGPUPrimitiveTopology_LineList: WGPUPrimitiveTopology = 2;
pub(crate) const WGPUPrimitiveTopology_LineStrip: WGPUPrimitiveTopology = 3;
pub(crate) const WGPUPrimitiveTopology_TriangleList: WGPUPrimitiveTopology = 4;
pub(crate) const WGPUPrimitiveTopology_TriangleStrip: WGPUPrimitiveTopology = 5;
pub(crate) const WGPUPrimitiveTopology_Force32: WGPUPrimitiveTopology = 2147483647;

pub(crate) type WGPUQueryType = i32;
pub(crate) const WGPUQueryType_Occlusion: WGPUQueryType = 1;
pub(crate) const WGPUQueryType_Timestamp: WGPUQueryType = 2;
pub(crate) const WGPUQueryType_Force32: WGPUQueryType = 2147483647;

pub(crate) type WGPUQueueWorkDoneStatus = i32;
pub(crate) const WGPUQueueWorkDoneStatus_Success: WGPUQueueWorkDoneStatus = 1;
pub(crate) const WGPUQueueWorkDoneStatus_CallbackCancelled: WGPUQueueWorkDoneStatus = 2;
pub(crate) const WGPUQueueWorkDoneStatus_Error: WGPUQueueWorkDoneStatus = 3;
pub(crate) const WGPUQueueWorkDoneStatus_Force32: WGPUQueueWorkDoneStatus = 2147483647;

pub(crate) type WGPURequestAdapterStatus = i32;
pub(crate) const WGPURequestAdapterStatus_Success: WGPURequestAdapterStatus = 1;
pub(crate) const WGPURequestAdapterStatus_CallbackCancelled: WGPURequestAdapterStatus = 2;
pub(crate) const WGPURequestAdapterStatus_Unavailable: WGPURequestAdapterStatus = 3;
pub(crate) const WGPURequestAdapterStatus_Error: WGPURequestAdapterStatus = 4;
pub(crate) const WGPURequestAdapterStatus_Force32: WGPURequestAdapterStatus = 2147483647;

pub(crate) type WGPURequestDeviceStatus = i32;
pub(crate) const WGPURequestDeviceStatus_Success: WGPURequestDeviceStatus = 1;
pub(crate) const WGPURequestDeviceStatus_CallbackCancelled: WGPURequestDeviceStatus = 2;
pub(crate) const WGPURequestDeviceStatus_Error: WGPURequestDeviceStatus = 3;
pub(crate) const WGPURequestDeviceStatus_Force32: WGPURequestDeviceStatus = 2147483647;

pub(crate) type WGPUSType = i32;
pub(crate) const WGPUSType_ShaderSourceSPIRV: WGPUSType = 1;
pub(crate) const WGPUSType_ShaderSourceWGSL: WGPUSType = 2;
pub(crate) const WGPUSType_RenderPassMaxDrawCount: WGPUSType = 3;
pub(crate) const WGPUSType_SurfaceSourceMetalLayer: WGPUSType = 4;
pub(crate) const WGPUSType_SurfaceSourceWindowsHWND: WGPUSType = 5;
pub(crate) const WGPUSType_SurfaceSourceXlibWindow: WGPUSType = 6;
pub(crate) const WGPUSType_SurfaceSourceWaylandSurface: WGPUSType = 7;
pub(crate) const WGPUSType_SurfaceSourceAndroidNativeWindow: WGPUSType = 8;
pub(crate) const WGPUSType_SurfaceSourceXCBWindow: WGPUSType = 9;
pub(crate) const WGPUSType_SurfaceColorManagement: WGPUSType = 10;
pub(crate) const WGPUSType_RequestAdapterWebXROptions: WGPUSType = 11;
pub(crate) const WGPUSType_TextureComponentSwizzleDescriptor: WGPUSType = 12;
pub(crate) const WGPUSType_CompatibilityModeLimits: WGPUSType = 131072;
pub(crate) const WGPUSType_TextureBindingViewDimensionDescriptor: WGPUSType = 131073;
pub(crate) const WGPUSType_EmscriptenSurfaceSourceCanvasHTMLSelector: WGPUSType = 262144;
pub(crate) const WGPUSType_DawnCompilationMessageUtf16: WGPUSType = 327743;
pub(crate) const WGPUSType_Force32: WGPUSType = 2147483647;

pub(crate) type WGPUSamplerBindingType = i32;
pub(crate) const WGPUSamplerBindingType_BindingNotUsed: WGPUSamplerBindingType = 0;
pub(crate) const WGPUSamplerBindingType_Undefined: WGPUSamplerBindingType = 1;
pub(crate) const WGPUSamplerBindingType_Filtering: WGPUSamplerBindingType = 2;
pub(crate) const WGPUSamplerBindingType_NonFiltering: WGPUSamplerBindingType = 3;
pub(crate) const WGPUSamplerBindingType_Comparison: WGPUSamplerBindingType = 4;
pub(crate) const WGPUSamplerBindingType_Force32: WGPUSamplerBindingType = 2147483647;

pub(crate) type WGPUStatus = i32;
pub(crate) const WGPUStatus_Success: WGPUStatus = 1;
pub(crate) const WGPUStatus_Error: WGPUStatus = 2;
pub(crate) const WGPUStatus_Force32: WGPUStatus = 2147483647;

pub(crate) type WGPUStencilOperation = i32;
pub(crate) const WGPUStencilOperation_Undefined: WGPUStencilOperation = 0;
pub(crate) const WGPUStencilOperation_Keep: WGPUStencilOperation = 1;
pub(crate) const WGPUStencilOperation_Zero: WGPUStencilOperation = 2;
pub(crate) const WGPUStencilOperation_Replace: WGPUStencilOperation = 3;
pub(crate) const WGPUStencilOperation_Invert: WGPUStencilOperation = 4;
pub(crate) const WGPUStencilOperation_IncrementClamp: WGPUStencilOperation = 5;
pub(crate) const WGPUStencilOperation_DecrementClamp: WGPUStencilOperation = 6;
pub(crate) const WGPUStencilOperation_IncrementWrap: WGPUStencilOperation = 7;
pub(crate) const WGPUStencilOperation_DecrementWrap: WGPUStencilOperation = 8;
pub(crate) const WGPUStencilOperation_Force32: WGPUStencilOperation = 2147483647;

pub(crate) type WGPUStorageTextureAccess = i32;
pub(crate) const WGPUStorageTextureAccess_BindingNotUsed: WGPUStorageTextureAccess = 0;
pub(crate) const WGPUStorageTextureAccess_Undefined: WGPUStorageTextureAccess = 1;
pub(crate) const WGPUStorageTextureAccess_WriteOnly: WGPUStorageTextureAccess = 2;
pub(crate) const WGPUStorageTextureAccess_ReadOnly: WGPUStorageTextureAccess = 3;
pub(crate) const WGPUStorageTextureAccess_ReadWrite: WGPUStorageTextureAccess = 4;
pub(crate) const WGPUStorageTextureAccess_Force32: WGPUStorageTextureAccess = 2147483647;

pub(crate) type WGPUStoreOp = i32;
pub(crate) const WGPUStoreOp_Undefined: WGPUStoreOp = 0;
pub(crate) const WGPUStoreOp_Store: WGPUStoreOp = 1;
pub(crate) const WGPUStoreOp_Discard: WGPUStoreOp = 2;
pub(crate) const WGPUStoreOp_Force32: WGPUStoreOp = 2147483647;

pub(crate) type WGPUSurfaceGetCurrentTextureStatus = i32;
pub(crate) const WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal: WGPUSurfaceGetCurrentTextureStatus = 1;
pub(crate) const WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal: WGPUSurfaceGetCurrentTextureStatus = 2;
pub(crate) const WGPUSurfaceGetCurrentTextureStatus_Timeout: WGPUSurfaceGetCurrentTextureStatus = 3;
pub(crate) const WGPUSurfaceGetCurrentTextureStatus_Outdated: WGPUSurfaceGetCurrentTextureStatus = 4;
pub(crate) const WGPUSurfaceGetCurrentTextureStatus_Lost: WGPUSurfaceGetCurrentTextureStatus = 5;
pub(crate) const WGPUSurfaceGetCurrentTextureStatus_Error: WGPUSurfaceGetCurrentTextureStatus = 6;
pub(crate) const WGPUSurfaceGetCurrentTextureStatus_Force32: WGPUSurfaceGetCurrentTextureStatus = 2147483647;

pub(crate) type WGPUTextureAspect = i32;
pub(crate) const WGPUTextureAspect_Undefined: WGPUTextureAspect = 0;
pub(crate) const WGPUTextureAspect_All: WGPUTextureAspect = 1;
pub(crate) const WGPUTextureAspect_StencilOnly: WGPUTextureAspect = 2;
pub(crate) const WGPUTextureAspect_DepthOnly: WGPUTextureAspect = 3;
pub(crate) const WGPUTextureAspect_Force32: WGPUTextureAspect = 2147483647;

pub(crate) type WGPUTextureDimension = i32;
pub(crate) const WGPUTextureDimension_Undefined: WGPUTextureDimension = 0;
pub(crate) const WGPUTextureDimension_1D: WGPUTextureDimension = 1;
pub(crate) const WGPUTextureDimension_2D: WGPUTextureDimension = 2;
pub(crate) const WGPUTextureDimension_3D: WGPUTextureDimension = 3;
pub(crate) const WGPUTextureDimension_Force32: WGPUTextureDimension = 2147483647;

pub(crate) type WGPUTextureFormat = i32;
pub(crate) const WGPUTextureFormat_Undefined: WGPUTextureFormat = 0;
pub(crate) const WGPUTextureFormat_R8Unorm: WGPUTextureFormat = 1;
pub(crate) const WGPUTextureFormat_R8Snorm: WGPUTextureFormat = 2;
pub(crate) const WGPUTextureFormat_R8Uint: WGPUTextureFormat = 3;
pub(crate) const WGPUTextureFormat_R8Sint: WGPUTextureFormat = 4;
pub(crate) const WGPUTextureFormat_R16Unorm: WGPUTextureFormat = 5;
pub(crate) const WGPUTextureFormat_R16Snorm: WGPUTextureFormat = 6;
pub(crate) const WGPUTextureFormat_R16Uint: WGPUTextureFormat = 7;
pub(crate) const WGPUTextureFormat_R16Sint: WGPUTextureFormat = 8;
pub(crate) const WGPUTextureFormat_R16Float: WGPUTextureFormat = 9;
pub(crate) const WGPUTextureFormat_RG8Unorm: WGPUTextureFormat = 10;
pub(crate) const WGPUTextureFormat_RG8Snorm: WGPUTextureFormat = 11;
pub(crate) const WGPUTextureFormat_RG8Uint: WGPUTextureFormat = 12;
pub(crate) const WGPUTextureFormat_RG8Sint: WGPUTextureFormat = 13;
pub(crate) const WGPUTextureFormat_R32Float: WGPUTextureFormat = 14;
pub(crate) const WGPUTextureFormat_R32Uint: WGPUTextureFormat = 15;
pub(crate) const WGPUTextureFormat_R32Sint: WGPUTextureFormat = 16;
pub(crate) const WGPUTextureFormat_RG16Unorm: WGPUTextureFormat = 17;
pub(crate) const WGPUTextureFormat_RG16Snorm: WGPUTextureFormat = 18;
pub(crate) const WGPUTextureFormat_RG16Uint: WGPUTextureFormat = 19;
pub(crate) const WGPUTextureFormat_RG16Sint: WGPUTextureFormat = 20;
pub(crate) const WGPUTextureFormat_RG16Float: WGPUTextureFormat = 21;
pub(crate) const WGPUTextureFormat_RGBA8Unorm: WGPUTextureFormat = 22;
pub(crate) const WGPUTextureFormat_RGBA8UnormSrgb: WGPUTextureFormat = 23;
pub(crate) const WGPUTextureFormat_RGBA8Snorm: WGPUTextureFormat = 24;
pub(crate) const WGPUTextureFormat_RGBA8Uint: WGPUTextureFormat = 25;
pub(crate) const WGPUTextureFormat_RGBA8Sint: WGPUTextureFormat = 26;
pub(crate) const WGPUTextureFormat_BGRA8Unorm: WGPUTextureFormat = 27;
pub(crate) const WGPUTextureFormat_BGRA8UnormSrgb: WGPUTextureFormat = 28;
pub(crate) const WGPUTextureFormat_RGB10A2Uint: WGPUTextureFormat = 29;
pub(crate) const WGPUTextureFormat_RGB10A2Unorm: WGPUTextureFormat = 30;
pub(crate) const WGPUTextureFormat_RG11B10Ufloat: WGPUTextureFormat = 31;
pub(crate) const WGPUTextureFormat_RGB9E5Ufloat: WGPUTextureFormat = 32;
pub(crate) const WGPUTextureFormat_RG32Float: WGPUTextureFormat = 33;
pub(crate) const WGPUTextureFormat_RG32Uint: WGPUTextureFormat = 34;
pub(crate) const WGPUTextureFormat_RG32Sint: WGPUTextureFormat = 35;
pub(crate) const WGPUTextureFormat_RGBA16Unorm: WGPUTextureFormat = 36;
pub(crate) const WGPUTextureFormat_RGBA16Snorm: WGPUTextureFormat = 37;
pub(crate) const WGPUTextureFormat_RGBA16Uint: WGPUTextureFormat = 38;
pub(crate) const WGPUTextureFormat_RGBA16Sint: WGPUTextureFormat = 39;
pub(crate) const WGPUTextureFormat_RGBA16Float: WGPUTextureFormat = 40;
pub(crate) const WGPUTextureFormat_RGBA32Float: WGPUTextureFormat = 41;
pub(crate) const WGPUTextureFormat_RGBA32Uint: WGPUTextureFormat = 42;
pub(crate) const WGPUTextureFormat_RGBA32Sint: WGPUTextureFormat = 43;
pub(crate) const WGPUTextureFormat_Stencil8: WGPUTextureFormat = 44;
pub(crate) const WGPUTextureFormat_Depth16Unorm: WGPUTextureFormat = 45;
pub(crate) const WGPUTextureFormat_Depth24Plus: WGPUTextureFormat = 46;
pub(crate) const WGPUTextureFormat_Depth24PlusStencil8: WGPUTextureFormat = 47;
pub(crate) const WGPUTextureFormat_Depth32Float: WGPUTextureFormat = 48;
pub(crate) const WGPUTextureFormat_Depth32FloatStencil8: WGPUTextureFormat = 49;
pub(crate) const WGPUTextureFormat_BC1RGBAUnorm: WGPUTextureFormat = 50;
pub(crate) const WGPUTextureFormat_BC1RGBAUnormSrgb: WGPUTextureFormat = 51;
pub(crate) const WGPUTextureFormat_BC2RGBAUnorm: WGPUTextureFormat = 52;
pub(crate) const WGPUTextureFormat_BC2RGBAUnormSrgb: WGPUTextureFormat = 53;
pub(crate) const WGPUTextureFormat_BC3RGBAUnorm: WGPUTextureFormat = 54;
pub(crate) const WGPUTextureFormat_BC3RGBAUnormSrgb: WGPUTextureFormat = 55;
pub(crate) const WGPUTextureFormat_BC4RUnorm: WGPUTextureFormat = 56;
pub(crate) const WGPUTextureFormat_BC4RSnorm: WGPUTextureFormat = 57;
pub(crate) const WGPUTextureFormat_BC5RGUnorm: WGPUTextureFormat = 58;
pub(crate) const WGPUTextureFormat_BC5RGSnorm: WGPUTextureFormat = 59;
pub(crate) const WGPUTextureFormat_BC6HRGBUfloat: WGPUTextureFormat = 60;
pub(crate) const WGPUTextureFormat_BC6HRGBFloat: WGPUTextureFormat = 61;
pub(crate) const WGPUTextureFormat_BC7RGBAUnorm: WGPUTextureFormat = 62;
pub(crate) const WGPUTextureFormat_BC7RGBAUnormSrgb: WGPUTextureFormat = 63;
pub(crate) const WGPUTextureFormat_ETC2RGB8Unorm: WGPUTextureFormat = 64;
pub(crate) const WGPUTextureFormat_ETC2RGB8UnormSrgb: WGPUTextureFormat = 65;
pub(crate) const WGPUTextureFormat_ETC2RGB8A1Unorm: WGPUTextureFormat = 66;
pub(crate) const WGPUTextureFormat_ETC2RGB8A1UnormSrgb: WGPUTextureFormat = 67;
pub(crate) const WGPUTextureFormat_ETC2RGBA8Unorm: WGPUTextureFormat = 68;
pub(crate) const WGPUTextureFormat_ETC2RGBA8UnormSrgb: WGPUTextureFormat = 69;
pub(crate) const WGPUTextureFormat_EACR11Unorm: WGPUTextureFormat = 70;
pub(crate) const WGPUTextureFormat_EACR11Snorm: WGPUTextureFormat = 71;
pub(crate) const WGPUTextureFormat_EACRG11Unorm: WGPUTextureFormat = 72;
pub(crate) const WGPUTextureFormat_EACRG11Snorm: WGPUTextureFormat = 73;
pub(crate) const WGPUTextureFormat_ASTC4x4Unorm: WGPUTextureFormat = 74;
pub(crate) const WGPUTextureFormat_ASTC4x4UnormSrgb: WGPUTextureFormat = 75;
pub(crate) const WGPUTextureFormat_ASTC5x4Unorm: WGPUTextureFormat = 76;
pub(crate) const WGPUTextureFormat_ASTC5x4UnormSrgb: WGPUTextureFormat = 77;
pub(crate) const WGPUTextureFormat_ASTC5x5Unorm: WGPUTextureFormat = 78;
pub(crate) const WGPUTextureFormat_ASTC5x5UnormSrgb: WGPUTextureFormat = 79;
pub(crate) const WGPUTextureFormat_ASTC6x5Unorm: WGPUTextureFormat = 80;
pub(crate) const WGPUTextureFormat_ASTC6x5UnormSrgb: WGPUTextureFormat = 81;
pub(crate) const WGPUTextureFormat_ASTC6x6Unorm: WGPUTextureFormat = 82;
pub(crate) const WGPUTextureFormat_ASTC6x6UnormSrgb: WGPUTextureFormat = 83;
pub(crate) const WGPUTextureFormat_ASTC8x5Unorm: WGPUTextureFormat = 84;
pub(crate) const WGPUTextureFormat_ASTC8x5UnormSrgb: WGPUTextureFormat = 85;
pub(crate) const WGPUTextureFormat_ASTC8x6Unorm: WGPUTextureFormat = 86;
pub(crate) const WGPUTextureFormat_ASTC8x6UnormSrgb: WGPUTextureFormat = 87;
pub(crate) const WGPUTextureFormat_ASTC8x8Unorm: WGPUTextureFormat = 88;
pub(crate) const WGPUTextureFormat_ASTC8x8UnormSrgb: WGPUTextureFormat = 89;
pub(crate) const WGPUTextureFormat_ASTC10x5Unorm: WGPUTextureFormat = 90;
pub(crate) const WGPUTextureFormat_ASTC10x5UnormSrgb: WGPUTextureFormat = 91;
pub(crate) const WGPUTextureFormat_ASTC10x6Unorm: WGPUTextureFormat = 92;
pub(crate) const WGPUTextureFormat_ASTC10x6UnormSrgb: WGPUTextureFormat = 93;
pub(crate) const WGPUTextureFormat_ASTC10x8Unorm: WGPUTextureFormat = 94;
pub(crate) const WGPUTextureFormat_ASTC10x8UnormSrgb: WGPUTextureFormat = 95;
pub(crate) const WGPUTextureFormat_ASTC10x10Unorm: WGPUTextureFormat = 96;
pub(crate) const WGPUTextureFormat_ASTC10x10UnormSrgb: WGPUTextureFormat = 97;
pub(crate) const WGPUTextureFormat_ASTC12x10Unorm: WGPUTextureFormat = 98;
pub(crate) const WGPUTextureFormat_ASTC12x10UnormSrgb: WGPUTextureFormat = 99;
pub(crate) const WGPUTextureFormat_ASTC12x12Unorm: WGPUTextureFormat = 100;
pub(crate) const WGPUTextureFormat_ASTC12x12UnormSrgb: WGPUTextureFormat = 101;
pub(crate) const WGPUTextureFormat_Force32: WGPUTextureFormat = 2147483647;

pub(crate) type WGPUTextureSampleType = i32;
pub(crate) const WGPUTextureSampleType_BindingNotUsed: WGPUTextureSampleType = 0;
pub(crate) const WGPUTextureSampleType_Undefined: WGPUTextureSampleType = 1;
pub(crate) const WGPUTextureSampleType_Float: WGPUTextureSampleType = 2;
pub(crate) const WGPUTextureSampleType_UnfilterableFloat: WGPUTextureSampleType = 3;
pub(crate) const WGPUTextureSampleType_Depth: WGPUTextureSampleType = 4;
pub(crate) const WGPUTextureSampleType_Sint: WGPUTextureSampleType = 5;
pub(crate) const WGPUTextureSampleType_Uint: WGPUTextureSampleType = 6;
pub(crate) const WGPUTextureSampleType_Force32: WGPUTextureSampleType = 2147483647;

pub(crate) type WGPUTextureViewDimension = i32;
pub(crate) const WGPUTextureViewDimension_Undefined: WGPUTextureViewDimension = 0;
pub(crate) const WGPUTextureViewDimension_1D: WGPUTextureViewDimension = 1;
pub(crate) const WGPUTextureViewDimension_2D: WGPUTextureViewDimension = 2;
pub(crate) const WGPUTextureViewDimension_2DArray: WGPUTextureViewDimension = 3;
pub(crate) const WGPUTextureViewDimension_Cube: WGPUTextureViewDimension = 4;
pub(crate) const WGPUTextureViewDimension_CubeArray: WGPUTextureViewDimension = 5;
pub(crate) const WGPUTextureViewDimension_3D: WGPUTextureViewDimension = 6;
pub(crate) const WGPUTextureViewDimension_Force32: WGPUTextureViewDimension = 2147483647;

pub(crate) type WGPUToneMappingMode = i32;
pub(crate) const WGPUToneMappingMode_Standard: WGPUToneMappingMode = 1;
pub(crate) const WGPUToneMappingMode_Extended: WGPUToneMappingMode = 2;
pub(crate) const WGPUToneMappingMode_Force32: WGPUToneMappingMode = 2147483647;

pub(crate) type WGPUVertexFormat = i32;
pub(crate) const WGPUVertexFormat_Uint8: WGPUVertexFormat = 1;
pub(crate) const WGPUVertexFormat_Uint8x2: WGPUVertexFormat = 2;
pub(crate) const WGPUVertexFormat_Uint8x4: WGPUVertexFormat = 3;
pub(crate) const WGPUVertexFormat_Sint8: WGPUVertexFormat = 4;
pub(crate) const WGPUVertexFormat_Sint8x2: WGPUVertexFormat = 5;
pub(crate) const WGPUVertexFormat_Sint8x4: WGPUVertexFormat = 6;
pub(crate) const WGPUVertexFormat_Unorm8: WGPUVertexFormat = 7;
pub(crate) const WGPUVertexFormat_Unorm8x2: WGPUVertexFormat = 8;
pub(crate) const WGPUVertexFormat_Unorm8x4: WGPUVertexFormat = 9;
pub(crate) const WGPUVertexFormat_Snorm8: WGPUVertexFormat = 10;
pub(crate) const WGPUVertexFormat_Snorm8x2: WGPUVertexFormat = 11;
pub(crate) const WGPUVertexFormat_Snorm8x4: WGPUVertexFormat = 12;
pub(crate) const WGPUVertexFormat_Uint16: WGPUVertexFormat = 13;
pub(crate) const WGPUVertexFormat_Uint16x2: WGPUVertexFormat = 14;
pub(crate) const WGPUVertexFormat_Uint16x4: WGPUVertexFormat = 15;
pub(crate) const WGPUVertexFormat_Sint16: WGPUVertexFormat = 16;
pub(crate) const WGPUVertexFormat_Sint16x2: WGPUVertexFormat = 17;
pub(crate) const WGPUVertexFormat_Sint16x4: WGPUVertexFormat = 18;
pub(crate) const WGPUVertexFormat_Unorm16: WGPUVertexFormat = 19;
pub(crate) const WGPUVertexFormat_Unorm16x2: WGPUVertexFormat = 20;
pub(crate) const WGPUVertexFormat_Unorm16x4: WGPUVertexFormat = 21;
pub(crate) const WGPUVertexFormat_Snorm16: WGPUVertexFormat = 22;
pub(crate) const WGPUVertexFormat_Snorm16x2: WGPUVertexFormat = 23;
pub(crate) const WGPUVertexFormat_Snorm16x4: WGPUVertexFormat = 24;
pub(crate) const WGPUVertexFormat_Float16: WGPUVertexFormat = 25;
pub(crate) const WGPUVertexFormat_Float16x2: WGPUVertexFormat = 26;
pub(crate) const WGPUVertexFormat_Float16x4: WGPUVertexFormat = 27;
pub(crate) const WGPUVertexFormat_Float32: WGPUVertexFormat = 28;
pub(crate) const WGPUVertexFormat_Float32x2: WGPUVertexFormat = 29;
pub(crate) const WGPUVertexFormat_Float32x3: WGPUVertexFormat = 30;
pub(crate) const WGPUVertexFormat_Float32x4: WGPUVertexFormat = 31;
pub(crate) const WGPUVertexFormat_Uint32: WGPUVertexFormat = 32;
pub(crate) const WGPUVertexFormat_Uint32x2: WGPUVertexFormat = 33;
pub(crate) const WGPUVertexFormat_Uint32x3: WGPUVertexFormat = 34;
pub(crate) const WGPUVertexFormat_Uint32x4: WGPUVertexFormat = 35;
pub(crate) const WGPUVertexFormat_Sint32: WGPUVertexFormat = 36;
pub(crate) const WGPUVertexFormat_Sint32x2: WGPUVertexFormat = 37;
pub(crate) const WGPUVertexFormat_Sint32x3: WGPUVertexFormat = 38;
pub(crate) const WGPUVertexFormat_Sint32x4: WGPUVertexFormat = 39;
pub(crate) const WGPUVertexFormat_Unorm10_10_10_2: WGPUVertexFormat = 40;
pub(crate) const WGPUVertexFormat_Unorm8x4BGRA: WGPUVertexFormat = 41;
pub(crate) const WGPUVertexFormat_Force32: WGPUVertexFormat = 2147483647;

pub(crate) type WGPUVertexStepMode = i32;
pub(crate) const WGPUVertexStepMode_Undefined: WGPUVertexStepMode = 0;
pub(crate) const WGPUVertexStepMode_Vertex: WGPUVertexStepMode = 1;
pub(crate) const WGPUVertexStepMode_Instance: WGPUVertexStepMode = 2;
pub(crate) const WGPUVertexStepMode_Force32: WGPUVertexStepMode = 2147483647;

pub(crate) type WGPUWGSLLanguageFeatureName = i32;
pub(crate) const WGPUWGSLLanguageFeatureName_ReadonlyAndReadwriteStorageTextures: WGPUWGSLLanguageFeatureName = 1;
pub(crate) const WGPUWGSLLanguageFeatureName_Packed4x8IntegerDotProduct: WGPUWGSLLanguageFeatureName = 2;
pub(crate) const WGPUWGSLLanguageFeatureName_UnrestrictedPointerParameters: WGPUWGSLLanguageFeatureName = 3;
pub(crate) const WGPUWGSLLanguageFeatureName_PointerCompositeAccess: WGPUWGSLLanguageFeatureName = 4;
pub(crate) const WGPUWGSLLanguageFeatureName_Force32: WGPUWGSLLanguageFeatureName = 2147483647;

pub(crate) type WGPUWaitStatus = i32;
pub(crate) const WGPUWaitStatus_Success: WGPUWaitStatus = 1;
pub(crate) const WGPUWaitStatus_TimedOut: WGPUWaitStatus = 2;
pub(crate) const WGPUWaitStatus_Error: WGPUWaitStatus = 3;
pub(crate) const WGPUWaitStatus_Force32: WGPUWaitStatus = 2147483647;

pub(crate) type WGPUBool = u32;
pub(crate) type WGPUBufferUsage = WGPUFlags;
pub(crate) type WGPUColorWriteMask = WGPUFlags;
pub(crate) type WGPUFlags = u64;
pub(crate) type WGPUMapMode = WGPUFlags;
pub(crate) type WGPUShaderStage = WGPUFlags;
pub(crate) type WGPUTextureUsage = WGPUFlags;

pub(crate) type WGPUBufferMapCallback = Option<unsafe extern "C" fn(WGPUMapAsyncStatus, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPUCompilationInfoCallback = Option<unsafe extern "C" fn(WGPUCompilationInfoRequestStatus, *const WGPUCompilationInfo, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPUCreateComputePipelineAsyncCallback = Option<unsafe extern "C" fn(WGPUCreatePipelineAsyncStatus, WGPUComputePipeline, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPUCreateRenderPipelineAsyncCallback = Option<unsafe extern "C" fn(WGPUCreatePipelineAsyncStatus, WGPURenderPipeline, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPUDeviceLostCallback = Option<unsafe extern "C" fn(*const WGPUDevice, WGPUDeviceLostReason, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPUPopErrorScopeCallback = Option<unsafe extern "C" fn(WGPUPopErrorScopeStatus, WGPUErrorType, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPUProc = Option<unsafe extern "C" fn()>;
pub(crate) type WGPUProcAdapterAddRef = Option<unsafe extern "C" fn(WGPUAdapter)>;
pub(crate) type WGPUProcAdapterGetFeatures = Option<unsafe extern "C" fn(WGPUAdapter, *mut WGPUSupportedFeatures)>;
pub(crate) type WGPUProcAdapterGetInfo = Option<unsafe extern "C" fn(WGPUAdapter, *mut WGPUAdapterInfo) -> WGPUStatus>;
pub(crate) type WGPUProcAdapterGetLimits = Option<unsafe extern "C" fn(WGPUAdapter, *mut WGPULimits) -> WGPUStatus>;
pub(crate) type WGPUProcAdapterHasFeature = Option<unsafe extern "C" fn(WGPUAdapter, WGPUFeatureName) -> WGPUBool>;
pub(crate) type WGPUProcAdapterInfoFreeMembers = Option<unsafe extern "C" fn(WGPUAdapterInfo)>;
pub(crate) type WGPUProcAdapterRelease = Option<unsafe extern "C" fn(WGPUAdapter)>;
pub(crate) type WGPUProcAdapterRequestDevice = Option<unsafe extern "C" fn(WGPUAdapter, *const WGPUDeviceDescriptor, WGPURequestDeviceCallbackInfo) -> WGPUFuture>;
pub(crate) type WGPUProcBindGroupAddRef = Option<unsafe extern "C" fn(WGPUBindGroup)>;
pub(crate) type WGPUProcBindGroupLayoutAddRef = Option<unsafe extern "C" fn(WGPUBindGroupLayout)>;
pub(crate) type WGPUProcBindGroupLayoutRelease = Option<unsafe extern "C" fn(WGPUBindGroupLayout)>;
pub(crate) type WGPUProcBindGroupLayoutSetLabel = Option<unsafe extern "C" fn(WGPUBindGroupLayout, WGPUStringView)>;
pub(crate) type WGPUProcBindGroupRelease = Option<unsafe extern "C" fn(WGPUBindGroup)>;
pub(crate) type WGPUProcBindGroupSetLabel = Option<unsafe extern "C" fn(WGPUBindGroup, WGPUStringView)>;
pub(crate) type WGPUProcBufferAddRef = Option<unsafe extern "C" fn(WGPUBuffer)>;
pub(crate) type WGPUProcBufferDestroy = Option<unsafe extern "C" fn(WGPUBuffer)>;
pub(crate) type WGPUProcBufferGetConstMappedRange = Option<unsafe extern "C" fn(WGPUBuffer, usize, usize) -> *const std::ffi::c_void>;
pub(crate) type WGPUProcBufferGetMapState = Option<unsafe extern "C" fn(WGPUBuffer) -> WGPUBufferMapState>;
pub(crate) type WGPUProcBufferGetMappedRange = Option<unsafe extern "C" fn(WGPUBuffer, usize, usize) -> *mut std::ffi::c_void>;
pub(crate) type WGPUProcBufferGetSize = Option<unsafe extern "C" fn(WGPUBuffer) -> u64>;
pub(crate) type WGPUProcBufferGetUsage = Option<unsafe extern "C" fn(WGPUBuffer) -> WGPUBufferUsage>;
pub(crate) type WGPUProcBufferMapAsync = Option<unsafe extern "C" fn(WGPUBuffer, WGPUMapMode, usize, usize, WGPUBufferMapCallbackInfo) -> WGPUFuture>;
pub(crate) type WGPUProcBufferReadMappedRange = Option<unsafe extern "C" fn(WGPUBuffer, usize, *mut std::ffi::c_void, usize) -> WGPUStatus>;
pub(crate) type WGPUProcBufferRelease = Option<unsafe extern "C" fn(WGPUBuffer)>;
pub(crate) type WGPUProcBufferSetLabel = Option<unsafe extern "C" fn(WGPUBuffer, WGPUStringView)>;
pub(crate) type WGPUProcBufferUnmap = Option<unsafe extern "C" fn(WGPUBuffer)>;
pub(crate) type WGPUProcBufferWriteMappedRange = Option<unsafe extern "C" fn(WGPUBuffer, usize, *const std::ffi::c_void, usize) -> WGPUStatus>;
pub(crate) type WGPUProcCommandBufferAddRef = Option<unsafe extern "C" fn(WGPUCommandBuffer)>;
pub(crate) type WGPUProcCommandBufferRelease = Option<unsafe extern "C" fn(WGPUCommandBuffer)>;
pub(crate) type WGPUProcCommandBufferSetLabel = Option<unsafe extern "C" fn(WGPUCommandBuffer, WGPUStringView)>;
pub(crate) type WGPUProcCommandEncoderAddRef = Option<unsafe extern "C" fn(WGPUCommandEncoder)>;
pub(crate) type WGPUProcCommandEncoderBeginComputePass = Option<unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUComputePassDescriptor) -> WGPUComputePassEncoder>;
pub(crate) type WGPUProcCommandEncoderBeginRenderPass = Option<unsafe extern "C" fn(WGPUCommandEncoder, *const WGPURenderPassDescriptor) -> WGPURenderPassEncoder>;
pub(crate) type WGPUProcCommandEncoderClearBuffer = Option<unsafe extern "C" fn(WGPUCommandEncoder, WGPUBuffer, u64, u64)>;
pub(crate) type WGPUProcCommandEncoderCopyBufferToBuffer = Option<unsafe extern "C" fn(WGPUCommandEncoder, WGPUBuffer, u64, WGPUBuffer, u64, u64)>;
pub(crate) type WGPUProcCommandEncoderCopyBufferToTexture = Option<unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUTexelCopyBufferInfo, *const WGPUTexelCopyTextureInfo, *const WGPUExtent3D)>;
pub(crate) type WGPUProcCommandEncoderCopyTextureToBuffer = Option<unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUTexelCopyTextureInfo, *const WGPUTexelCopyBufferInfo, *const WGPUExtent3D)>;
pub(crate) type WGPUProcCommandEncoderCopyTextureToTexture = Option<unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUTexelCopyTextureInfo, *const WGPUTexelCopyTextureInfo, *const WGPUExtent3D)>;
pub(crate) type WGPUProcCommandEncoderFinish = Option<unsafe extern "C" fn(WGPUCommandEncoder, *const WGPUCommandBufferDescriptor) -> WGPUCommandBuffer>;
pub(crate) type WGPUProcCommandEncoderInsertDebugMarker = Option<unsafe extern "C" fn(WGPUCommandEncoder, WGPUStringView)>;
pub(crate) type WGPUProcCommandEncoderPopDebugGroup = Option<unsafe extern "C" fn(WGPUCommandEncoder)>;
pub(crate) type WGPUProcCommandEncoderPushDebugGroup = Option<unsafe extern "C" fn(WGPUCommandEncoder, WGPUStringView)>;
pub(crate) type WGPUProcCommandEncoderRelease = Option<unsafe extern "C" fn(WGPUCommandEncoder)>;
pub(crate) type WGPUProcCommandEncoderResolveQuerySet = Option<unsafe extern "C" fn(WGPUCommandEncoder, WGPUQuerySet, u32, u32, WGPUBuffer, u64)>;
pub(crate) type WGPUProcCommandEncoderSetLabel = Option<unsafe extern "C" fn(WGPUCommandEncoder, WGPUStringView)>;
pub(crate) type WGPUProcCommandEncoderWriteTimestamp = Option<unsafe extern "C" fn(WGPUCommandEncoder, WGPUQuerySet, u32)>;
pub(crate) type WGPUProcComputePassEncoderAddRef = Option<unsafe extern "C" fn(WGPUComputePassEncoder)>;
pub(crate) type WGPUProcComputePassEncoderDispatchWorkgroups = Option<unsafe extern "C" fn(WGPUComputePassEncoder, u32, u32, u32)>;
pub(crate) type WGPUProcComputePassEncoderDispatchWorkgroupsIndirect = Option<unsafe extern "C" fn(WGPUComputePassEncoder, WGPUBuffer, u64)>;
pub(crate) type WGPUProcComputePassEncoderEnd = Option<unsafe extern "C" fn(WGPUComputePassEncoder)>;
pub(crate) type WGPUProcComputePassEncoderInsertDebugMarker = Option<unsafe extern "C" fn(WGPUComputePassEncoder, WGPUStringView)>;
pub(crate) type WGPUProcComputePassEncoderPopDebugGroup = Option<unsafe extern "C" fn(WGPUComputePassEncoder)>;
pub(crate) type WGPUProcComputePassEncoderPushDebugGroup = Option<unsafe extern "C" fn(WGPUComputePassEncoder, WGPUStringView)>;
pub(crate) type WGPUProcComputePassEncoderRelease = Option<unsafe extern "C" fn(WGPUComputePassEncoder)>;
pub(crate) type WGPUProcComputePassEncoderSetBindGroup = Option<unsafe extern "C" fn(WGPUComputePassEncoder, u32, WGPUBindGroup, usize, *const u32)>;
pub(crate) type WGPUProcComputePassEncoderSetLabel = Option<unsafe extern "C" fn(WGPUComputePassEncoder, WGPUStringView)>;
pub(crate) type WGPUProcComputePassEncoderSetPipeline = Option<unsafe extern "C" fn(WGPUComputePassEncoder, WGPUComputePipeline)>;
pub(crate) type WGPUProcComputePassEncoderWriteTimestamp = Option<unsafe extern "C" fn(WGPUComputePassEncoder, WGPUQuerySet, u32)>;
pub(crate) type WGPUProcComputePipelineAddRef = Option<unsafe extern "C" fn(WGPUComputePipeline)>;
pub(crate) type WGPUProcComputePipelineGetBindGroupLayout = Option<unsafe extern "C" fn(WGPUComputePipeline, u32) -> WGPUBindGroupLayout>;
pub(crate) type WGPUProcComputePipelineRelease = Option<unsafe extern "C" fn(WGPUComputePipeline)>;
pub(crate) type WGPUProcComputePipelineSetLabel = Option<unsafe extern "C" fn(WGPUComputePipeline, WGPUStringView)>;
pub(crate) type WGPUProcCreateInstance = Option<unsafe extern "C" fn(*const WGPUInstanceDescriptor) -> WGPUInstance>;
pub(crate) type WGPUProcDeviceAddRef = Option<unsafe extern "C" fn(WGPUDevice)>;
pub(crate) type WGPUProcDeviceCreateBindGroup = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUBindGroupDescriptor) -> WGPUBindGroup>;
pub(crate) type WGPUProcDeviceCreateBindGroupLayout = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUBindGroupLayoutDescriptor) -> WGPUBindGroupLayout>;
pub(crate) type WGPUProcDeviceCreateBuffer = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUBufferDescriptor) -> WGPUBuffer>;
pub(crate) type WGPUProcDeviceCreateCommandEncoder = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUCommandEncoderDescriptor) -> WGPUCommandEncoder>;
pub(crate) type WGPUProcDeviceCreateComputePipeline = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUComputePipelineDescriptor) -> WGPUComputePipeline>;
pub(crate) type WGPUProcDeviceCreateComputePipelineAsync = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUComputePipelineDescriptor, WGPUCreateComputePipelineAsyncCallbackInfo) -> WGPUFuture>;
pub(crate) type WGPUProcDeviceCreatePipelineLayout = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUPipelineLayoutDescriptor) -> WGPUPipelineLayout>;
pub(crate) type WGPUProcDeviceCreateQuerySet = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUQuerySetDescriptor) -> WGPUQuerySet>;
pub(crate) type WGPUProcDeviceCreateRenderBundleEncoder = Option<unsafe extern "C" fn(WGPUDevice, *const WGPURenderBundleEncoderDescriptor) -> WGPURenderBundleEncoder>;
pub(crate) type WGPUProcDeviceCreateRenderPipeline = Option<unsafe extern "C" fn(WGPUDevice, *const WGPURenderPipelineDescriptor) -> WGPURenderPipeline>;
pub(crate) type WGPUProcDeviceCreateRenderPipelineAsync = Option<unsafe extern "C" fn(WGPUDevice, *const WGPURenderPipelineDescriptor, WGPUCreateRenderPipelineAsyncCallbackInfo) -> WGPUFuture>;
pub(crate) type WGPUProcDeviceCreateSampler = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUSamplerDescriptor) -> WGPUSampler>;
pub(crate) type WGPUProcDeviceCreateShaderModule = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUShaderModuleDescriptor) -> WGPUShaderModule>;
pub(crate) type WGPUProcDeviceCreateTexture = Option<unsafe extern "C" fn(WGPUDevice, *const WGPUTextureDescriptor) -> WGPUTexture>;
pub(crate) type WGPUProcDeviceDestroy = Option<unsafe extern "C" fn(WGPUDevice)>;
pub(crate) type WGPUProcDeviceGetAdapterInfo = Option<unsafe extern "C" fn(WGPUDevice, *mut WGPUAdapterInfo) -> WGPUStatus>;
pub(crate) type WGPUProcDeviceGetFeatures = Option<unsafe extern "C" fn(WGPUDevice, *mut WGPUSupportedFeatures)>;
pub(crate) type WGPUProcDeviceGetLimits = Option<unsafe extern "C" fn(WGPUDevice, *mut WGPULimits) -> WGPUStatus>;
pub(crate) type WGPUProcDeviceGetLostFuture = Option<unsafe extern "C" fn(WGPUDevice) -> WGPUFuture>;
pub(crate) type WGPUProcDeviceGetQueue = Option<unsafe extern "C" fn(WGPUDevice) -> WGPUQueue>;
pub(crate) type WGPUProcDeviceHasFeature = Option<unsafe extern "C" fn(WGPUDevice, WGPUFeatureName) -> WGPUBool>;
pub(crate) type WGPUProcDevicePopErrorScope = Option<unsafe extern "C" fn(WGPUDevice, WGPUPopErrorScopeCallbackInfo) -> WGPUFuture>;
pub(crate) type WGPUProcDevicePushErrorScope = Option<unsafe extern "C" fn(WGPUDevice, WGPUErrorFilter)>;
pub(crate) type WGPUProcDeviceRelease = Option<unsafe extern "C" fn(WGPUDevice)>;
pub(crate) type WGPUProcDeviceSetLabel = Option<unsafe extern "C" fn(WGPUDevice, WGPUStringView)>;
pub(crate) type WGPUProcGetInstanceFeatures = Option<unsafe extern "C" fn(*mut WGPUSupportedInstanceFeatures)>;
pub(crate) type WGPUProcGetInstanceLimits = Option<unsafe extern "C" fn(*mut WGPUInstanceLimits) -> WGPUStatus>;
pub(crate) type WGPUProcGetProcAddress = Option<unsafe extern "C" fn(WGPUStringView) -> WGPUProc>;
pub(crate) type WGPUProcHasInstanceFeature = Option<unsafe extern "C" fn(WGPUInstanceFeatureName) -> WGPUBool>;
pub(crate) type WGPUProcInstanceAddRef = Option<unsafe extern "C" fn(WGPUInstance)>;
pub(crate) type WGPUProcInstanceCreateSurface = Option<unsafe extern "C" fn(WGPUInstance, *const WGPUSurfaceDescriptor) -> WGPUSurface>;
pub(crate) type WGPUProcInstanceGetWGSLLanguageFeatures = Option<unsafe extern "C" fn(WGPUInstance, *mut WGPUSupportedWGSLLanguageFeatures)>;
pub(crate) type WGPUProcInstanceHasWGSLLanguageFeature = Option<unsafe extern "C" fn(WGPUInstance, WGPUWGSLLanguageFeatureName) -> WGPUBool>;
pub(crate) type WGPUProcInstanceProcessEvents = Option<unsafe extern "C" fn(WGPUInstance)>;
pub(crate) type WGPUProcInstanceRelease = Option<unsafe extern "C" fn(WGPUInstance)>;
pub(crate) type WGPUProcInstanceRequestAdapter = Option<unsafe extern "C" fn(WGPUInstance, *const WGPURequestAdapterOptions, WGPURequestAdapterCallbackInfo) -> WGPUFuture>;
pub(crate) type WGPUProcInstanceWaitAny = Option<unsafe extern "C" fn(WGPUInstance, usize, *mut WGPUFutureWaitInfo, u64) -> WGPUWaitStatus>;
pub(crate) type WGPUProcPipelineLayoutAddRef = Option<unsafe extern "C" fn(WGPUPipelineLayout)>;
pub(crate) type WGPUProcPipelineLayoutRelease = Option<unsafe extern "C" fn(WGPUPipelineLayout)>;
pub(crate) type WGPUProcPipelineLayoutSetLabel = Option<unsafe extern "C" fn(WGPUPipelineLayout, WGPUStringView)>;
pub(crate) type WGPUProcQuerySetAddRef = Option<unsafe extern "C" fn(WGPUQuerySet)>;
pub(crate) type WGPUProcQuerySetDestroy = Option<unsafe extern "C" fn(WGPUQuerySet)>;
pub(crate) type WGPUProcQuerySetGetCount = Option<unsafe extern "C" fn(WGPUQuerySet) -> u32>;
pub(crate) type WGPUProcQuerySetGetType = Option<unsafe extern "C" fn(WGPUQuerySet) -> WGPUQueryType>;
pub(crate) type WGPUProcQuerySetRelease = Option<unsafe extern "C" fn(WGPUQuerySet)>;
pub(crate) type WGPUProcQuerySetSetLabel = Option<unsafe extern "C" fn(WGPUQuerySet, WGPUStringView)>;
pub(crate) type WGPUProcQueueAddRef = Option<unsafe extern "C" fn(WGPUQueue)>;
pub(crate) type WGPUProcQueueOnSubmittedWorkDone = Option<unsafe extern "C" fn(WGPUQueue, WGPUQueueWorkDoneCallbackInfo) -> WGPUFuture>;
pub(crate) type WGPUProcQueueRelease = Option<unsafe extern "C" fn(WGPUQueue)>;
pub(crate) type WGPUProcQueueSetLabel = Option<unsafe extern "C" fn(WGPUQueue, WGPUStringView)>;
pub(crate) type WGPUProcQueueSubmit = Option<unsafe extern "C" fn(WGPUQueue, usize, *const WGPUCommandBuffer)>;
pub(crate) type WGPUProcQueueWriteBuffer = Option<unsafe extern "C" fn(WGPUQueue, WGPUBuffer, u64, *const std::ffi::c_void, usize)>;
pub(crate) type WGPUProcQueueWriteTexture = Option<unsafe extern "C" fn(WGPUQueue, *const WGPUTexelCopyTextureInfo, *const std::ffi::c_void, usize, *const WGPUTexelCopyBufferLayout, *const WGPUExtent3D)>;
pub(crate) type WGPUProcRenderBundleAddRef = Option<unsafe extern "C" fn(WGPURenderBundle)>;
pub(crate) type WGPUProcRenderBundleEncoderAddRef = Option<unsafe extern "C" fn(WGPURenderBundleEncoder)>;
pub(crate) type WGPUProcRenderBundleEncoderDraw = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, u32, u32, u32, u32)>;
pub(crate) type WGPUProcRenderBundleEncoderDrawIndexed = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, u32, u32, u32, i32, u32)>;
pub(crate) type WGPUProcRenderBundleEncoderDrawIndexedIndirect = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUBuffer, u64)>;
pub(crate) type WGPUProcRenderBundleEncoderDrawIndirect = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUBuffer, u64)>;
pub(crate) type WGPUProcRenderBundleEncoderFinish = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, *const WGPURenderBundleDescriptor) -> WGPURenderBundle>;
pub(crate) type WGPUProcRenderBundleEncoderInsertDebugMarker = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUStringView)>;
pub(crate) type WGPUProcRenderBundleEncoderPopDebugGroup = Option<unsafe extern "C" fn(WGPURenderBundleEncoder)>;
pub(crate) type WGPUProcRenderBundleEncoderPushDebugGroup = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUStringView)>;
pub(crate) type WGPUProcRenderBundleEncoderRelease = Option<unsafe extern "C" fn(WGPURenderBundleEncoder)>;
pub(crate) type WGPUProcRenderBundleEncoderSetBindGroup = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, u32, WGPUBindGroup, usize, *const u32)>;
pub(crate) type WGPUProcRenderBundleEncoderSetIndexBuffer = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUBuffer, WGPUIndexFormat, u64, u64)>;
pub(crate) type WGPUProcRenderBundleEncoderSetLabel = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, WGPUStringView)>;
pub(crate) type WGPUProcRenderBundleEncoderSetPipeline = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, WGPURenderPipeline)>;
pub(crate) type WGPUProcRenderBundleEncoderSetVertexBuffer = Option<unsafe extern "C" fn(WGPURenderBundleEncoder, u32, WGPUBuffer, u64, u64)>;
pub(crate) type WGPUProcRenderBundleRelease = Option<unsafe extern "C" fn(WGPURenderBundle)>;
pub(crate) type WGPUProcRenderBundleSetLabel = Option<unsafe extern "C" fn(WGPURenderBundle, WGPUStringView)>;
pub(crate) type WGPUProcRenderPassEncoderAddRef = Option<unsafe extern "C" fn(WGPURenderPassEncoder)>;
pub(crate) type WGPUProcRenderPassEncoderBeginOcclusionQuery = Option<unsafe extern "C" fn(WGPURenderPassEncoder, u32)>;
pub(crate) type WGPUProcRenderPassEncoderDraw = Option<unsafe extern "C" fn(WGPURenderPassEncoder, u32, u32, u32, u32)>;
pub(crate) type WGPUProcRenderPassEncoderDrawIndexed = Option<unsafe extern "C" fn(WGPURenderPassEncoder, u32, u32, u32, i32, u32)>;
pub(crate) type WGPUProcRenderPassEncoderDrawIndexedIndirect = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUBuffer, u64)>;
pub(crate) type WGPUProcRenderPassEncoderDrawIndirect = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUBuffer, u64)>;
pub(crate) type WGPUProcRenderPassEncoderEnd = Option<unsafe extern "C" fn(WGPURenderPassEncoder)>;
pub(crate) type WGPUProcRenderPassEncoderEndOcclusionQuery = Option<unsafe extern "C" fn(WGPURenderPassEncoder)>;
pub(crate) type WGPUProcRenderPassEncoderExecuteBundles = Option<unsafe extern "C" fn(WGPURenderPassEncoder, usize, *const WGPURenderBundle)>;
pub(crate) type WGPUProcRenderPassEncoderInsertDebugMarker = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUStringView)>;
pub(crate) type WGPUProcRenderPassEncoderMultiDrawIndexedIndirect = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUBuffer, u64, u32, WGPUBuffer, u64)>;
pub(crate) type WGPUProcRenderPassEncoderMultiDrawIndirect = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUBuffer, u64, u32, WGPUBuffer, u64)>;
pub(crate) type WGPUProcRenderPassEncoderPopDebugGroup = Option<unsafe extern "C" fn(WGPURenderPassEncoder)>;
pub(crate) type WGPUProcRenderPassEncoderPushDebugGroup = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUStringView)>;
pub(crate) type WGPUProcRenderPassEncoderRelease = Option<unsafe extern "C" fn(WGPURenderPassEncoder)>;
pub(crate) type WGPUProcRenderPassEncoderSetBindGroup = Option<unsafe extern "C" fn(WGPURenderPassEncoder, u32, WGPUBindGroup, usize, *const u32)>;
pub(crate) type WGPUProcRenderPassEncoderSetBlendConstant = Option<unsafe extern "C" fn(WGPURenderPassEncoder, *const WGPUColor)>;
pub(crate) type WGPUProcRenderPassEncoderSetIndexBuffer = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUBuffer, WGPUIndexFormat, u64, u64)>;
pub(crate) type WGPUProcRenderPassEncoderSetLabel = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUStringView)>;
pub(crate) type WGPUProcRenderPassEncoderSetPipeline = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPURenderPipeline)>;
pub(crate) type WGPUProcRenderPassEncoderSetScissorRect = Option<unsafe extern "C" fn(WGPURenderPassEncoder, u32, u32, u32, u32)>;
pub(crate) type WGPUProcRenderPassEncoderSetStencilReference = Option<unsafe extern "C" fn(WGPURenderPassEncoder, u32)>;
pub(crate) type WGPUProcRenderPassEncoderSetVertexBuffer = Option<unsafe extern "C" fn(WGPURenderPassEncoder, u32, WGPUBuffer, u64, u64)>;
pub(crate) type WGPUProcRenderPassEncoderSetViewport = Option<unsafe extern "C" fn(WGPURenderPassEncoder, f32, f32, f32, f32, f32, f32)>;
pub(crate) type WGPUProcRenderPassEncoderWriteTimestamp = Option<unsafe extern "C" fn(WGPURenderPassEncoder, WGPUQuerySet, u32)>;
pub(crate) type WGPUProcRenderPipelineAddRef = Option<unsafe extern "C" fn(WGPURenderPipeline)>;
pub(crate) type WGPUProcRenderPipelineGetBindGroupLayout = Option<unsafe extern "C" fn(WGPURenderPipeline, u32) -> WGPUBindGroupLayout>;
pub(crate) type WGPUProcRenderPipelineRelease = Option<unsafe extern "C" fn(WGPURenderPipeline)>;
pub(crate) type WGPUProcRenderPipelineSetLabel = Option<unsafe extern "C" fn(WGPURenderPipeline, WGPUStringView)>;
pub(crate) type WGPUProcSamplerAddRef = Option<unsafe extern "C" fn(WGPUSampler)>;
pub(crate) type WGPUProcSamplerRelease = Option<unsafe extern "C" fn(WGPUSampler)>;
pub(crate) type WGPUProcSamplerSetLabel = Option<unsafe extern "C" fn(WGPUSampler, WGPUStringView)>;
pub(crate) type WGPUProcShaderModuleAddRef = Option<unsafe extern "C" fn(WGPUShaderModule)>;
pub(crate) type WGPUProcShaderModuleGetCompilationInfo = Option<unsafe extern "C" fn(WGPUShaderModule, WGPUCompilationInfoCallbackInfo) -> WGPUFuture>;
pub(crate) type WGPUProcShaderModuleRelease = Option<unsafe extern "C" fn(WGPUShaderModule)>;
pub(crate) type WGPUProcShaderModuleSetLabel = Option<unsafe extern "C" fn(WGPUShaderModule, WGPUStringView)>;
pub(crate) type WGPUProcSupportedFeaturesFreeMembers = Option<unsafe extern "C" fn(WGPUSupportedFeatures)>;
pub(crate) type WGPUProcSupportedInstanceFeaturesFreeMembers = Option<unsafe extern "C" fn(WGPUSupportedInstanceFeatures)>;
pub(crate) type WGPUProcSupportedWGSLLanguageFeaturesFreeMembers = Option<unsafe extern "C" fn(WGPUSupportedWGSLLanguageFeatures)>;
pub(crate) type WGPUProcSurfaceAddRef = Option<unsafe extern "C" fn(WGPUSurface)>;
pub(crate) type WGPUProcSurfaceCapabilitiesFreeMembers = Option<unsafe extern "C" fn(WGPUSurfaceCapabilities)>;
pub(crate) type WGPUProcSurfaceConfigure = Option<unsafe extern "C" fn(WGPUSurface, *const WGPUSurfaceConfiguration)>;
pub(crate) type WGPUProcSurfaceGetCapabilities = Option<unsafe extern "C" fn(WGPUSurface, WGPUAdapter, *mut WGPUSurfaceCapabilities) -> WGPUStatus>;
pub(crate) type WGPUProcSurfaceGetCurrentTexture = Option<unsafe extern "C" fn(WGPUSurface, *mut WGPUSurfaceTexture)>;
pub(crate) type WGPUProcSurfacePresent = Option<unsafe extern "C" fn(WGPUSurface) -> WGPUStatus>;
pub(crate) type WGPUProcSurfaceRelease = Option<unsafe extern "C" fn(WGPUSurface)>;
pub(crate) type WGPUProcSurfaceSetLabel = Option<unsafe extern "C" fn(WGPUSurface, WGPUStringView)>;
pub(crate) type WGPUProcSurfaceUnconfigure = Option<unsafe extern "C" fn(WGPUSurface)>;
pub(crate) type WGPUProcTextureAddRef = Option<unsafe extern "C" fn(WGPUTexture)>;
pub(crate) type WGPUProcTextureCreateView = Option<unsafe extern "C" fn(WGPUTexture, *const WGPUTextureViewDescriptor) -> WGPUTextureView>;
pub(crate) type WGPUProcTextureDestroy = Option<unsafe extern "C" fn(WGPUTexture)>;
pub(crate) type WGPUProcTextureGetDepthOrArrayLayers = Option<unsafe extern "C" fn(WGPUTexture) -> u32>;
pub(crate) type WGPUProcTextureGetDimension = Option<unsafe extern "C" fn(WGPUTexture) -> WGPUTextureDimension>;
pub(crate) type WGPUProcTextureGetFormat = Option<unsafe extern "C" fn(WGPUTexture) -> WGPUTextureFormat>;
pub(crate) type WGPUProcTextureGetHeight = Option<unsafe extern "C" fn(WGPUTexture) -> u32>;
pub(crate) type WGPUProcTextureGetMipLevelCount = Option<unsafe extern "C" fn(WGPUTexture) -> u32>;
pub(crate) type WGPUProcTextureGetSampleCount = Option<unsafe extern "C" fn(WGPUTexture) -> u32>;
pub(crate) type WGPUProcTextureGetUsage = Option<unsafe extern "C" fn(WGPUTexture) -> WGPUTextureUsage>;
pub(crate) type WGPUProcTextureGetWidth = Option<unsafe extern "C" fn(WGPUTexture) -> u32>;
pub(crate) type WGPUProcTextureRelease = Option<unsafe extern "C" fn(WGPUTexture)>;
pub(crate) type WGPUProcTextureSetLabel = Option<unsafe extern "C" fn(WGPUTexture, WGPUStringView)>;
pub(crate) type WGPUProcTextureViewAddRef = Option<unsafe extern "C" fn(WGPUTextureView)>;
pub(crate) type WGPUProcTextureViewRelease = Option<unsafe extern "C" fn(WGPUTextureView)>;
pub(crate) type WGPUProcTextureViewSetLabel = Option<unsafe extern "C" fn(WGPUTextureView, WGPUStringView)>;
pub(crate) type WGPUQueueWorkDoneCallback = Option<unsafe extern "C" fn(WGPUQueueWorkDoneStatus, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPURequestAdapterCallback = Option<unsafe extern "C" fn(WGPURequestAdapterStatus, WGPUAdapter, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPURequestDeviceCallback = Option<unsafe extern "C" fn(WGPURequestDeviceStatus, WGPUDevice, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;
pub(crate) type WGPUUncapturedErrorCallback = Option<unsafe extern "C" fn(*const WGPUDevice, WGPUErrorType, WGPUStringView, *mut std::ffi::c_void, *mut std::ffi::c_void)>;

pub(crate) const WGPUBufferUsage_None: WGPUBufferUsage = 0 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_MapRead: WGPUBufferUsage = 1 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_MapWrite: WGPUBufferUsage = 2 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_CopySrc: WGPUBufferUsage = 4 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_CopyDst: WGPUBufferUsage = 8 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_Index: WGPUBufferUsage = 16 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_Vertex: WGPUBufferUsage = 32 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_Uniform: WGPUBufferUsage = 64 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_Storage: WGPUBufferUsage = 128 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_Indirect: WGPUBufferUsage = 256 as WGPUBufferUsage;
pub(crate) const WGPUBufferUsage_QueryResolve: WGPUBufferUsage = 512 as WGPUBufferUsage;
pub(crate) const WGPUColorWriteMask_None: WGPUColorWriteMask = 0 as WGPUColorWriteMask;
pub(crate) const WGPUColorWriteMask_Red: WGPUColorWriteMask = 1 as WGPUColorWriteMask;
pub(crate) const WGPUColorWriteMask_Green: WGPUColorWriteMask = 2 as WGPUColorWriteMask;
pub(crate) const WGPUColorWriteMask_Blue: WGPUColorWriteMask = 4 as WGPUColorWriteMask;
pub(crate) const WGPUColorWriteMask_Alpha: WGPUColorWriteMask = 8 as WGPUColorWriteMask;
pub(crate) const WGPUColorWriteMask_All: WGPUColorWriteMask = 15 as WGPUColorWriteMask;
pub(crate) const WGPUMapMode_None: WGPUMapMode = 0 as WGPUMapMode;
pub(crate) const WGPUMapMode_Read: WGPUMapMode = 1 as WGPUMapMode;
pub(crate) const WGPUMapMode_Write: WGPUMapMode = 2 as WGPUMapMode;
pub(crate) const WGPUShaderStage_None: WGPUShaderStage = 0 as WGPUShaderStage;
pub(crate) const WGPUShaderStage_Vertex: WGPUShaderStage = 1 as WGPUShaderStage;
pub(crate) const WGPUShaderStage_Fragment: WGPUShaderStage = 2 as WGPUShaderStage;
pub(crate) const WGPUShaderStage_Compute: WGPUShaderStage = 4 as WGPUShaderStage;
pub(crate) const WGPUTextureUsage_None: WGPUTextureUsage = 0 as WGPUTextureUsage;
pub(crate) const WGPUTextureUsage_CopySrc: WGPUTextureUsage = 1 as WGPUTextureUsage;
pub(crate) const WGPUTextureUsage_CopyDst: WGPUTextureUsage = 2 as WGPUTextureUsage;
pub(crate) const WGPUTextureUsage_TextureBinding: WGPUTextureUsage = 4 as WGPUTextureUsage;
pub(crate) const WGPUTextureUsage_StorageBinding: WGPUTextureUsage = 8 as WGPUTextureUsage;
pub(crate) const WGPUTextureUsage_RenderAttachment: WGPUTextureUsage = 16 as WGPUTextureUsage;

#[repr(C)]
pub(crate) struct WGPUStringView {
    pub(crate) data: *const std::ffi::c_char,
    pub(crate) length: usize,
}

#[repr(C)]
pub(crate) struct WGPUChainedStruct {
    pub(crate) next: *mut WGPUChainedStruct,
    pub(crate) sType: WGPUSType,
}

#[repr(C)]
pub(crate) struct WGPUBufferMapCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUBufferMapCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUCompilationInfoCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUCompilationInfoCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUCreateComputePipelineAsyncCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUCreateComputePipelineAsyncCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUCreateRenderPipelineAsyncCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUCreateRenderPipelineAsyncCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUDeviceLostCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUDeviceLostCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUPopErrorScopeCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUPopErrorScopeCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUQueueWorkDoneCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPUQueueWorkDoneCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPURequestAdapterCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPURequestAdapterCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPURequestDeviceCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) mode: WGPUCallbackMode,
    pub(crate) callback: WGPURequestDeviceCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUUncapturedErrorCallbackInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) callback: WGPUUncapturedErrorCallback,
    pub(crate) userdata1: *mut std::ffi::c_void,
    pub(crate) userdata2: *mut std::ffi::c_void,
}

#[repr(C)]
pub(crate) struct WGPUAdapterInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) vendor: WGPUStringView,
    pub(crate) architecture: WGPUStringView,
    pub(crate) device: WGPUStringView,
    pub(crate) description: WGPUStringView,
    pub(crate) backendType: WGPUBackendType,
    pub(crate) adapterType: WGPUAdapterType,
    pub(crate) vendorID: u32,
    pub(crate) deviceID: u32,
    pub(crate) subgroupMinSize: u32,
    pub(crate) subgroupMaxSize: u32,
}

#[repr(C)]
pub(crate) struct WGPUBindGroupEntry {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) binding: u32,
    pub(crate) buffer: WGPUBuffer,
    pub(crate) offset: u64,
    pub(crate) size: u64,
    pub(crate) sampler: WGPUSampler,
    pub(crate) textureView: WGPUTextureView,
}

#[repr(C)]
pub(crate) struct WGPUBlendComponent {
    pub(crate) operation: WGPUBlendOperation,
    pub(crate) srcFactor: WGPUBlendFactor,
    pub(crate) dstFactor: WGPUBlendFactor,
}

#[repr(C)]
pub(crate) struct WGPUBufferBindingLayout {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) r#type: WGPUBufferBindingType,
    pub(crate) hasDynamicOffset: WGPUBool,
    pub(crate) minBindingSize: u64,
}

#[repr(C)]
pub(crate) struct WGPUBufferDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) usage: WGPUBufferUsage,
    pub(crate) size: u64,
    pub(crate) mappedAtCreation: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUColor {
    pub(crate) r: f64,
    pub(crate) g: f64,
    pub(crate) b: f64,
    pub(crate) a: f64,
}

#[repr(C)]
pub(crate) struct WGPUCommandBufferDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUCommandEncoderDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUCompatibilityModeLimits {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) maxStorageBuffersInVertexStage: u32,
    pub(crate) maxStorageTexturesInVertexStage: u32,
    pub(crate) maxStorageBuffersInFragmentStage: u32,
    pub(crate) maxStorageTexturesInFragmentStage: u32,
}

#[repr(C)]
pub(crate) struct WGPUConstantEntry {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) key: WGPUStringView,
    pub(crate) value: f64,
}

#[repr(C)]
pub(crate) struct WGPUDawnCompilationMessageUtf16 {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) linePos: u64,
    pub(crate) offset: u64,
    pub(crate) length: u64,
}

#[repr(C)]
pub(crate) struct WGPUEmscriptenSurfaceSourceCanvasHTMLSelector {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) selector: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUExtent3D {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) depthOrArrayLayers: u32,
}

#[repr(C)]
pub(crate) struct WGPUFuture {
    pub(crate) id: u64,
}

#[repr(C)]
pub(crate) struct WGPUInstanceLimits {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) timedWaitAnyMaxCount: usize,
}

#[repr(C)]
pub(crate) struct WGPUINTERNAL_HAVE_EMDAWNWEBGPU_HEADER {
    pub(crate) unused: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUMultisampleState {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) count: u32,
    pub(crate) mask: u32,
    pub(crate) alphaToCoverageEnabled: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUOrigin3D {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) z: u32,
}

#[repr(C)]
pub(crate) struct WGPUPassTimestampWrites {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) querySet: WGPUQuerySet,
    pub(crate) beginningOfPassWriteIndex: u32,
    pub(crate) endOfPassWriteIndex: u32,
}

#[repr(C)]
pub(crate) struct WGPUPipelineLayoutDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) bindGroupLayoutCount: usize,
    pub(crate) bindGroupLayouts: *const WGPUBindGroupLayout,
    pub(crate) immediateSize: u32,
}

#[repr(C)]
pub(crate) struct WGPUPrimitiveState {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) topology: WGPUPrimitiveTopology,
    pub(crate) stripIndexFormat: WGPUIndexFormat,
    pub(crate) frontFace: WGPUFrontFace,
    pub(crate) cullMode: WGPUCullMode,
    pub(crate) unclippedDepth: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUQuerySetDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) r#type: WGPUQueryType,
    pub(crate) count: u32,
}

#[repr(C)]
pub(crate) struct WGPUQueueDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPURenderBundleDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPURenderBundleEncoderDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) colorFormatCount: usize,
    pub(crate) colorFormats: *const WGPUTextureFormat,
    pub(crate) depthStencilFormat: WGPUTextureFormat,
    pub(crate) sampleCount: u32,
    pub(crate) depthReadOnly: WGPUBool,
    pub(crate) stencilReadOnly: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPURenderPassDepthStencilAttachment {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) view: WGPUTextureView,
    pub(crate) depthLoadOp: WGPULoadOp,
    pub(crate) depthStoreOp: WGPUStoreOp,
    pub(crate) depthClearValue: f32,
    pub(crate) depthReadOnly: WGPUBool,
    pub(crate) stencilLoadOp: WGPULoadOp,
    pub(crate) stencilStoreOp: WGPUStoreOp,
    pub(crate) stencilClearValue: u32,
    pub(crate) stencilReadOnly: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPURenderPassMaxDrawCount {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) maxDrawCount: u64,
}

#[repr(C)]
pub(crate) struct WGPURequestAdapterWebXROptions {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) xrCompatible: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUSamplerBindingLayout {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) r#type: WGPUSamplerBindingType,
}

#[repr(C)]
pub(crate) struct WGPUSamplerDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) addressModeU: WGPUAddressMode,
    pub(crate) addressModeV: WGPUAddressMode,
    pub(crate) addressModeW: WGPUAddressMode,
    pub(crate) magFilter: WGPUFilterMode,
    pub(crate) minFilter: WGPUFilterMode,
    pub(crate) mipmapFilter: WGPUMipmapFilterMode,
    pub(crate) lodMinClamp: f32,
    pub(crate) lodMaxClamp: f32,
    pub(crate) compare: WGPUCompareFunction,
    pub(crate) maxAnisotropy: u16,
}

#[repr(C)]
pub(crate) struct WGPUShaderSourceSPIRV {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) codeSize: u32,
    pub(crate) code: *const u32,
}

#[repr(C)]
pub(crate) struct WGPUShaderSourceWGSL {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) code: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUStencilFaceState {
    pub(crate) compare: WGPUCompareFunction,
    pub(crate) failOp: WGPUStencilOperation,
    pub(crate) depthFailOp: WGPUStencilOperation,
    pub(crate) passOp: WGPUStencilOperation,
}

#[repr(C)]
pub(crate) struct WGPUStorageTextureBindingLayout {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) access: WGPUStorageTextureAccess,
    pub(crate) format: WGPUTextureFormat,
    pub(crate) viewDimension: WGPUTextureViewDimension,
}

#[repr(C)]
pub(crate) struct WGPUSupportedFeatures {
    pub(crate) featureCount: usize,
    pub(crate) features: *const WGPUFeatureName,
}

#[repr(C)]
pub(crate) struct WGPUSupportedInstanceFeatures {
    pub(crate) featureCount: usize,
    pub(crate) features: *const WGPUInstanceFeatureName,
}

#[repr(C)]
pub(crate) struct WGPUSupportedWGSLLanguageFeatures {
    pub(crate) featureCount: usize,
    pub(crate) features: *const WGPUWGSLLanguageFeatureName,
}

#[repr(C)]
pub(crate) struct WGPUSurfaceCapabilities {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) usages: WGPUTextureUsage,
    pub(crate) formatCount: usize,
    pub(crate) formats: *const WGPUTextureFormat,
    pub(crate) presentModeCount: usize,
    pub(crate) presentModes: *const WGPUPresentMode,
    pub(crate) alphaModeCount: usize,
    pub(crate) alphaModes: *const WGPUCompositeAlphaMode,
}

#[repr(C)]
pub(crate) struct WGPUSurfaceColorManagement {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) colorSpace: WGPUPredefinedColorSpace,
    pub(crate) toneMappingMode: WGPUToneMappingMode,
}

#[repr(C)]
pub(crate) struct WGPUSurfaceConfiguration {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) device: WGPUDevice,
    pub(crate) format: WGPUTextureFormat,
    pub(crate) usage: WGPUTextureUsage,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) viewFormatCount: usize,
    pub(crate) viewFormats: *const WGPUTextureFormat,
    pub(crate) alphaMode: WGPUCompositeAlphaMode,
    pub(crate) presentMode: WGPUPresentMode,
}

#[repr(C)]
pub(crate) struct WGPUSurfaceTexture {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) texture: WGPUTexture,
    pub(crate) status: WGPUSurfaceGetCurrentTextureStatus,
}

#[repr(C)]
pub(crate) struct WGPUTexelCopyBufferLayout {
    pub(crate) offset: u64,
    pub(crate) bytesPerRow: u32,
    pub(crate) rowsPerImage: u32,
}

#[repr(C)]
pub(crate) struct WGPUTextureBindingLayout {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) sampleType: WGPUTextureSampleType,
    pub(crate) viewDimension: WGPUTextureViewDimension,
    pub(crate) multisampled: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUTextureBindingViewDimensionDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) textureBindingViewDimension: WGPUTextureViewDimension,
}

#[repr(C)]
pub(crate) struct WGPUTextureComponentSwizzle {
    pub(crate) r: WGPUComponentSwizzle,
    pub(crate) g: WGPUComponentSwizzle,
    pub(crate) b: WGPUComponentSwizzle,
    pub(crate) a: WGPUComponentSwizzle,
}

#[repr(C)]
pub(crate) struct WGPUVertexAttribute {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) format: WGPUVertexFormat,
    pub(crate) offset: u64,
    pub(crate) shaderLocation: u32,
}

#[repr(C)]
pub(crate) struct WGPUBindGroupDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) layout: WGPUBindGroupLayout,
    pub(crate) entryCount: usize,
    pub(crate) entries: *const WGPUBindGroupEntry,
}

#[repr(C)]
pub(crate) struct WGPUBindGroupLayoutEntry {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) binding: u32,
    pub(crate) visibility: WGPUShaderStage,
    pub(crate) bindingArraySize: u32,
    pub(crate) buffer: WGPUBufferBindingLayout,
    pub(crate) sampler: WGPUSamplerBindingLayout,
    pub(crate) texture: WGPUTextureBindingLayout,
    pub(crate) storageTexture: WGPUStorageTextureBindingLayout,
}

#[repr(C)]
pub(crate) struct WGPUBlendState {
    pub(crate) color: WGPUBlendComponent,
    pub(crate) alpha: WGPUBlendComponent,
}

#[repr(C)]
pub(crate) struct WGPUCompilationMessage {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) message: WGPUStringView,
    pub(crate) r#type: WGPUCompilationMessageType,
    pub(crate) lineNum: u64,
    pub(crate) linePos: u64,
    pub(crate) offset: u64,
    pub(crate) length: u64,
}

#[repr(C)]
pub(crate) struct WGPUComputePassDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) timestampWrites: *const WGPUPassTimestampWrites,
}

#[repr(C)]
pub(crate) struct WGPUComputeState {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) module: WGPUShaderModule,
    pub(crate) entryPoint: WGPUStringView,
    pub(crate) constantCount: usize,
    pub(crate) constants: *const WGPUConstantEntry,
}

#[repr(C)]
pub(crate) struct WGPUDepthStencilState {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) format: WGPUTextureFormat,
    pub(crate) depthWriteEnabled: WGPUOptionalBool,
    pub(crate) depthCompare: WGPUCompareFunction,
    pub(crate) stencilFront: WGPUStencilFaceState,
    pub(crate) stencilBack: WGPUStencilFaceState,
    pub(crate) stencilReadMask: u32,
    pub(crate) stencilWriteMask: u32,
    pub(crate) depthBias: i32,
    pub(crate) depthBiasSlopeScale: f32,
    pub(crate) depthBiasClamp: f32,
}

#[repr(C)]
pub(crate) struct WGPUFutureWaitInfo {
    pub(crate) future: WGPUFuture,
    pub(crate) completed: WGPUBool,
}

#[repr(C)]
pub(crate) struct WGPUInstanceDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) requiredFeatureCount: usize,
    pub(crate) requiredFeatures: *const WGPUInstanceFeatureName,
    pub(crate) requiredLimits: *const WGPUInstanceLimits,
}

#[repr(C)]
pub(crate) struct WGPULimits {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) maxTextureDimension1D: u32,
    pub(crate) maxTextureDimension2D: u32,
    pub(crate) maxTextureDimension3D: u32,
    pub(crate) maxTextureArrayLayers: u32,
    pub(crate) maxBindGroups: u32,
    pub(crate) maxBindGroupsPlusVertexBuffers: u32,
    pub(crate) maxBindingsPerBindGroup: u32,
    pub(crate) maxDynamicUniformBuffersPerPipelineLayout: u32,
    pub(crate) maxDynamicStorageBuffersPerPipelineLayout: u32,
    pub(crate) maxSampledTexturesPerShaderStage: u32,
    pub(crate) maxSamplersPerShaderStage: u32,
    pub(crate) maxStorageBuffersPerShaderStage: u32,
    pub(crate) maxStorageTexturesPerShaderStage: u32,
    pub(crate) maxUniformBuffersPerShaderStage: u32,
    pub(crate) maxUniformBufferBindingSize: u64,
    pub(crate) maxStorageBufferBindingSize: u64,
    pub(crate) minUniformBufferOffsetAlignment: u32,
    pub(crate) minStorageBufferOffsetAlignment: u32,
    pub(crate) maxVertexBuffers: u32,
    pub(crate) maxBufferSize: u64,
    pub(crate) maxVertexAttributes: u32,
    pub(crate) maxVertexBufferArrayStride: u32,
    pub(crate) maxInterStageShaderVariables: u32,
    pub(crate) maxColorAttachments: u32,
    pub(crate) maxColorAttachmentBytesPerSample: u32,
    pub(crate) maxComputeWorkgroupStorageSize: u32,
    pub(crate) maxComputeInvocationsPerWorkgroup: u32,
    pub(crate) maxComputeWorkgroupSizeX: u32,
    pub(crate) maxComputeWorkgroupSizeY: u32,
    pub(crate) maxComputeWorkgroupSizeZ: u32,
    pub(crate) maxComputeWorkgroupsPerDimension: u32,
    pub(crate) maxImmediateSize: u32,
}

#[repr(C)]
pub(crate) struct WGPURenderPassColorAttachment {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) view: WGPUTextureView,
    pub(crate) depthSlice: u32,
    pub(crate) resolveTarget: WGPUTextureView,
    pub(crate) loadOp: WGPULoadOp,
    pub(crate) storeOp: WGPUStoreOp,
    pub(crate) clearValue: WGPUColor,
}

#[repr(C)]
pub(crate) struct WGPURequestAdapterOptions {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) featureLevel: WGPUFeatureLevel,
    pub(crate) powerPreference: WGPUPowerPreference,
    pub(crate) forceFallbackAdapter: WGPUBool,
    pub(crate) backendType: WGPUBackendType,
    pub(crate) compatibleSurface: WGPUSurface,
}

#[repr(C)]
pub(crate) struct WGPUShaderModuleDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUSurfaceDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
}

#[repr(C)]
pub(crate) struct WGPUTexelCopyBufferInfo {
    pub(crate) layout: WGPUTexelCopyBufferLayout,
    pub(crate) buffer: WGPUBuffer,
}

#[repr(C)]
pub(crate) struct WGPUTexelCopyTextureInfo {
    pub(crate) texture: WGPUTexture,
    pub(crate) mipLevel: u32,
    pub(crate) origin: WGPUOrigin3D,
    pub(crate) aspect: WGPUTextureAspect,
}

#[repr(C)]
pub(crate) struct WGPUTextureComponentSwizzleDescriptor {
    pub(crate) chain: WGPUChainedStruct,
    pub(crate) swizzle: WGPUTextureComponentSwizzle,
}

#[repr(C)]
pub(crate) struct WGPUTextureDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) usage: WGPUTextureUsage,
    pub(crate) dimension: WGPUTextureDimension,
    pub(crate) size: WGPUExtent3D,
    pub(crate) format: WGPUTextureFormat,
    pub(crate) mipLevelCount: u32,
    pub(crate) sampleCount: u32,
    pub(crate) viewFormatCount: usize,
    pub(crate) viewFormats: *const WGPUTextureFormat,
}

#[repr(C)]
pub(crate) struct WGPUVertexBufferLayout {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) stepMode: WGPUVertexStepMode,
    pub(crate) arrayStride: u64,
    pub(crate) attributeCount: usize,
    pub(crate) attributes: *const WGPUVertexAttribute,
}

#[repr(C)]
pub(crate) struct WGPUBindGroupLayoutDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) entryCount: usize,
    pub(crate) entries: *const WGPUBindGroupLayoutEntry,
}

#[repr(C)]
pub(crate) struct WGPUColorTargetState {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) format: WGPUTextureFormat,
    pub(crate) blend: *const WGPUBlendState,
    pub(crate) writeMask: WGPUColorWriteMask,
}

#[repr(C)]
pub(crate) struct WGPUCompilationInfo {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) messageCount: usize,
    pub(crate) messages: *const WGPUCompilationMessage,
}

#[repr(C)]
pub(crate) struct WGPUComputePipelineDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) layout: WGPUPipelineLayout,
    pub(crate) compute: WGPUComputeState,
}

#[repr(C)]
pub(crate) struct WGPUDeviceDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) requiredFeatureCount: usize,
    pub(crate) requiredFeatures: *const WGPUFeatureName,
    pub(crate) requiredLimits: *const WGPULimits,
    pub(crate) defaultQueue: WGPUQueueDescriptor,
    pub(crate) deviceLostCallbackInfo: WGPUDeviceLostCallbackInfo,
    pub(crate) uncapturedErrorCallbackInfo: WGPUUncapturedErrorCallbackInfo,
}

#[repr(C)]
pub(crate) struct WGPURenderPassDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) colorAttachmentCount: usize,
    pub(crate) colorAttachments: *const WGPURenderPassColorAttachment,
    pub(crate) depthStencilAttachment: *const WGPURenderPassDepthStencilAttachment,
    pub(crate) occlusionQuerySet: WGPUQuerySet,
    pub(crate) timestampWrites: *const WGPUPassTimestampWrites,
}

#[repr(C)]
pub(crate) struct WGPUTextureViewDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) format: WGPUTextureFormat,
    pub(crate) dimension: WGPUTextureViewDimension,
    pub(crate) baseMipLevel: u32,
    pub(crate) mipLevelCount: u32,
    pub(crate) baseArrayLayer: u32,
    pub(crate) arrayLayerCount: u32,
    pub(crate) aspect: WGPUTextureAspect,
    pub(crate) usage: WGPUTextureUsage,
}

#[repr(C)]
pub(crate) struct WGPUVertexState {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) module: WGPUShaderModule,
    pub(crate) entryPoint: WGPUStringView,
    pub(crate) constantCount: usize,
    pub(crate) constants: *const WGPUConstantEntry,
    pub(crate) bufferCount: usize,
    pub(crate) buffers: *const WGPUVertexBufferLayout,
}

#[repr(C)]
pub(crate) struct WGPUFragmentState {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) module: WGPUShaderModule,
    pub(crate) entryPoint: WGPUStringView,
    pub(crate) constantCount: usize,
    pub(crate) constants: *const WGPUConstantEntry,
    pub(crate) targetCount: usize,
    pub(crate) targets: *const WGPUColorTargetState,
}

#[repr(C)]
pub(crate) struct WGPURenderPipelineDescriptor {
    pub(crate) nextInChain: *mut WGPUChainedStruct,
    pub(crate) label: WGPUStringView,
    pub(crate) layout: WGPUPipelineLayout,
    pub(crate) vertex: WGPUVertexState,
    pub(crate) primitive: WGPUPrimitiveState,
    pub(crate) depthStencil: *const WGPUDepthStencilState,
    pub(crate) multisample: WGPUMultisampleState,
    pub(crate) fragment: *const WGPUFragmentState,
}

unsafe extern "C" {
    pub(crate) fn wgpuCreateInstance(arg0: *const WGPUInstanceDescriptor) -> WGPUInstance;
    pub(crate) fn wgpuGetInstanceFeatures(arg0: *mut WGPUSupportedInstanceFeatures);
    pub(crate) fn wgpuGetInstanceLimits(arg0: *mut WGPUInstanceLimits) -> WGPUStatus;
    pub(crate) fn wgpuHasInstanceFeature(arg0: WGPUInstanceFeatureName) -> WGPUBool;
    pub(crate) fn wgpuGetProcAddress(arg0: WGPUStringView) -> WGPUProc;
    pub(crate) fn wgpuAdapterGetFeatures(arg0: WGPUAdapter, arg1: *mut WGPUSupportedFeatures);
    pub(crate) fn wgpuAdapterGetInfo(arg0: WGPUAdapter, arg1: *mut WGPUAdapterInfo) -> WGPUStatus;
    pub(crate) fn wgpuAdapterGetLimits(arg0: WGPUAdapter, arg1: *mut WGPULimits) -> WGPUStatus;
    pub(crate) fn wgpuAdapterHasFeature(arg0: WGPUAdapter, arg1: WGPUFeatureName) -> WGPUBool;
    pub(crate) fn wgpuAdapterRequestDevice(arg0: WGPUAdapter, arg1: *const WGPUDeviceDescriptor, arg2: WGPURequestDeviceCallbackInfo) -> WGPUFuture;
    pub(crate) fn wgpuAdapterAddRef(arg0: WGPUAdapter);
    pub(crate) fn wgpuAdapterRelease(arg0: WGPUAdapter);
    pub(crate) fn wgpuAdapterInfoFreeMembers(arg0: WGPUAdapterInfo);
    pub(crate) fn wgpuBindGroupSetLabel(arg0: WGPUBindGroup, arg1: WGPUStringView);
    pub(crate) fn wgpuBindGroupAddRef(arg0: WGPUBindGroup);
    pub(crate) fn wgpuBindGroupRelease(arg0: WGPUBindGroup);
    pub(crate) fn wgpuBindGroupLayoutSetLabel(arg0: WGPUBindGroupLayout, arg1: WGPUStringView);
    pub(crate) fn wgpuBindGroupLayoutAddRef(arg0: WGPUBindGroupLayout);
    pub(crate) fn wgpuBindGroupLayoutRelease(arg0: WGPUBindGroupLayout);
    pub(crate) fn wgpuBufferDestroy(arg0: WGPUBuffer);
    pub(crate) fn wgpuBufferGetConstMappedRange(arg0: WGPUBuffer, arg1: usize, arg2: usize) -> *const std::ffi::c_void;
    pub(crate) fn wgpuBufferGetMappedRange(arg0: WGPUBuffer, arg1: usize, arg2: usize) -> *mut std::ffi::c_void;
    pub(crate) fn wgpuBufferGetMapState(arg0: WGPUBuffer) -> WGPUBufferMapState;
    pub(crate) fn wgpuBufferGetSize(arg0: WGPUBuffer) -> u64;
    pub(crate) fn wgpuBufferGetUsage(arg0: WGPUBuffer) -> WGPUBufferUsage;
    pub(crate) fn wgpuBufferMapAsync(arg0: WGPUBuffer, arg1: WGPUMapMode, arg2: usize, arg3: usize, arg4: WGPUBufferMapCallbackInfo) -> WGPUFuture;
    pub(crate) fn wgpuBufferReadMappedRange(arg0: WGPUBuffer, arg1: usize, arg2: *mut std::ffi::c_void, arg3: usize) -> WGPUStatus;
    pub(crate) fn wgpuBufferSetLabel(arg0: WGPUBuffer, arg1: WGPUStringView);
    pub(crate) fn wgpuBufferUnmap(arg0: WGPUBuffer);
    pub(crate) fn wgpuBufferWriteMappedRange(arg0: WGPUBuffer, arg1: usize, arg2: *const std::ffi::c_void, arg3: usize) -> WGPUStatus;
    pub(crate) fn wgpuBufferAddRef(arg0: WGPUBuffer);
    pub(crate) fn wgpuBufferRelease(arg0: WGPUBuffer);
    pub(crate) fn wgpuCommandBufferSetLabel(arg0: WGPUCommandBuffer, arg1: WGPUStringView);
    pub(crate) fn wgpuCommandBufferAddRef(arg0: WGPUCommandBuffer);
    pub(crate) fn wgpuCommandBufferRelease(arg0: WGPUCommandBuffer);
    pub(crate) fn wgpuCommandEncoderBeginComputePass(arg0: WGPUCommandEncoder, arg1: *const WGPUComputePassDescriptor) -> WGPUComputePassEncoder;
    pub(crate) fn wgpuCommandEncoderBeginRenderPass(arg0: WGPUCommandEncoder, arg1: *const WGPURenderPassDescriptor) -> WGPURenderPassEncoder;
    pub(crate) fn wgpuCommandEncoderClearBuffer(arg0: WGPUCommandEncoder, arg1: WGPUBuffer, arg2: u64, arg3: u64);
    pub(crate) fn wgpuCommandEncoderCopyBufferToBuffer(arg0: WGPUCommandEncoder, arg1: WGPUBuffer, arg2: u64, arg3: WGPUBuffer, arg4: u64, arg5: u64);
    pub(crate) fn wgpuCommandEncoderCopyBufferToTexture(arg0: WGPUCommandEncoder, arg1: *const WGPUTexelCopyBufferInfo, arg2: *const WGPUTexelCopyTextureInfo, arg3: *const WGPUExtent3D);
    pub(crate) fn wgpuCommandEncoderCopyTextureToBuffer(arg0: WGPUCommandEncoder, arg1: *const WGPUTexelCopyTextureInfo, arg2: *const WGPUTexelCopyBufferInfo, arg3: *const WGPUExtent3D);
    pub(crate) fn wgpuCommandEncoderCopyTextureToTexture(arg0: WGPUCommandEncoder, arg1: *const WGPUTexelCopyTextureInfo, arg2: *const WGPUTexelCopyTextureInfo, arg3: *const WGPUExtent3D);
    pub(crate) fn wgpuCommandEncoderFinish(arg0: WGPUCommandEncoder, arg1: *const WGPUCommandBufferDescriptor) -> WGPUCommandBuffer;
    pub(crate) fn wgpuCommandEncoderInsertDebugMarker(arg0: WGPUCommandEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuCommandEncoderPopDebugGroup(arg0: WGPUCommandEncoder);
    pub(crate) fn wgpuCommandEncoderPushDebugGroup(arg0: WGPUCommandEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuCommandEncoderResolveQuerySet(arg0: WGPUCommandEncoder, arg1: WGPUQuerySet, arg2: u32, arg3: u32, arg4: WGPUBuffer, arg5: u64);
    pub(crate) fn wgpuCommandEncoderSetLabel(arg0: WGPUCommandEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuCommandEncoderWriteTimestamp(arg0: WGPUCommandEncoder, arg1: WGPUQuerySet, arg2: u32);
    pub(crate) fn wgpuCommandEncoderAddRef(arg0: WGPUCommandEncoder);
    pub(crate) fn wgpuCommandEncoderRelease(arg0: WGPUCommandEncoder);
    pub(crate) fn wgpuComputePassEncoderDispatchWorkgroups(arg0: WGPUComputePassEncoder, arg1: u32, arg2: u32, arg3: u32);
    pub(crate) fn wgpuComputePassEncoderDispatchWorkgroupsIndirect(arg0: WGPUComputePassEncoder, arg1: WGPUBuffer, arg2: u64);
    pub(crate) fn wgpuComputePassEncoderEnd(arg0: WGPUComputePassEncoder);
    pub(crate) fn wgpuComputePassEncoderInsertDebugMarker(arg0: WGPUComputePassEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuComputePassEncoderPopDebugGroup(arg0: WGPUComputePassEncoder);
    pub(crate) fn wgpuComputePassEncoderPushDebugGroup(arg0: WGPUComputePassEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuComputePassEncoderSetBindGroup(arg0: WGPUComputePassEncoder, arg1: u32, arg2: WGPUBindGroup, arg3: usize, arg4: *const u32);
    pub(crate) fn wgpuComputePassEncoderSetLabel(arg0: WGPUComputePassEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuComputePassEncoderSetPipeline(arg0: WGPUComputePassEncoder, arg1: WGPUComputePipeline);
    pub(crate) fn wgpuComputePassEncoderWriteTimestamp(arg0: WGPUComputePassEncoder, arg1: WGPUQuerySet, arg2: u32);
    pub(crate) fn wgpuComputePassEncoderAddRef(arg0: WGPUComputePassEncoder);
    pub(crate) fn wgpuComputePassEncoderRelease(arg0: WGPUComputePassEncoder);
    pub(crate) fn wgpuComputePipelineGetBindGroupLayout(arg0: WGPUComputePipeline, arg1: u32) -> WGPUBindGroupLayout;
    pub(crate) fn wgpuComputePipelineSetLabel(arg0: WGPUComputePipeline, arg1: WGPUStringView);
    pub(crate) fn wgpuComputePipelineAddRef(arg0: WGPUComputePipeline);
    pub(crate) fn wgpuComputePipelineRelease(arg0: WGPUComputePipeline);
    pub(crate) fn wgpuDeviceCreateBindGroup(arg0: WGPUDevice, arg1: *const WGPUBindGroupDescriptor) -> WGPUBindGroup;
    pub(crate) fn wgpuDeviceCreateBindGroupLayout(arg0: WGPUDevice, arg1: *const WGPUBindGroupLayoutDescriptor) -> WGPUBindGroupLayout;
    pub(crate) fn wgpuDeviceCreateBuffer(arg0: WGPUDevice, arg1: *const WGPUBufferDescriptor) -> WGPUBuffer;
    pub(crate) fn wgpuDeviceCreateCommandEncoder(arg0: WGPUDevice, arg1: *const WGPUCommandEncoderDescriptor) -> WGPUCommandEncoder;
    pub(crate) fn wgpuDeviceCreateComputePipeline(arg0: WGPUDevice, arg1: *const WGPUComputePipelineDescriptor) -> WGPUComputePipeline;
    pub(crate) fn wgpuDeviceCreateComputePipelineAsync(arg0: WGPUDevice, arg1: *const WGPUComputePipelineDescriptor, arg2: WGPUCreateComputePipelineAsyncCallbackInfo) -> WGPUFuture;
    pub(crate) fn wgpuDeviceCreatePipelineLayout(arg0: WGPUDevice, arg1: *const WGPUPipelineLayoutDescriptor) -> WGPUPipelineLayout;
    pub(crate) fn wgpuDeviceCreateQuerySet(arg0: WGPUDevice, arg1: *const WGPUQuerySetDescriptor) -> WGPUQuerySet;
    pub(crate) fn wgpuDeviceCreateRenderBundleEncoder(arg0: WGPUDevice, arg1: *const WGPURenderBundleEncoderDescriptor) -> WGPURenderBundleEncoder;
    pub(crate) fn wgpuDeviceCreateRenderPipeline(arg0: WGPUDevice, arg1: *const WGPURenderPipelineDescriptor) -> WGPURenderPipeline;
    pub(crate) fn wgpuDeviceCreateRenderPipelineAsync(arg0: WGPUDevice, arg1: *const WGPURenderPipelineDescriptor, arg2: WGPUCreateRenderPipelineAsyncCallbackInfo) -> WGPUFuture;
    pub(crate) fn wgpuDeviceCreateSampler(arg0: WGPUDevice, arg1: *const WGPUSamplerDescriptor) -> WGPUSampler;
    pub(crate) fn wgpuDeviceCreateShaderModule(arg0: WGPUDevice, arg1: *const WGPUShaderModuleDescriptor) -> WGPUShaderModule;
    pub(crate) fn wgpuDeviceCreateTexture(arg0: WGPUDevice, arg1: *const WGPUTextureDescriptor) -> WGPUTexture;
    pub(crate) fn wgpuDeviceDestroy(arg0: WGPUDevice);
    pub(crate) fn wgpuDeviceGetAdapterInfo(arg0: WGPUDevice, arg1: *mut WGPUAdapterInfo) -> WGPUStatus;
    pub(crate) fn wgpuDeviceGetFeatures(arg0: WGPUDevice, arg1: *mut WGPUSupportedFeatures);
    pub(crate) fn wgpuDeviceGetLimits(arg0: WGPUDevice, arg1: *mut WGPULimits) -> WGPUStatus;
    pub(crate) fn wgpuDeviceGetLostFuture(arg0: WGPUDevice) -> WGPUFuture;
    pub(crate) fn wgpuDeviceGetQueue(arg0: WGPUDevice) -> WGPUQueue;
    pub(crate) fn wgpuDeviceHasFeature(arg0: WGPUDevice, arg1: WGPUFeatureName) -> WGPUBool;
    pub(crate) fn wgpuDevicePopErrorScope(arg0: WGPUDevice, arg1: WGPUPopErrorScopeCallbackInfo) -> WGPUFuture;
    pub(crate) fn wgpuDevicePushErrorScope(arg0: WGPUDevice, arg1: WGPUErrorFilter);
    pub(crate) fn wgpuDeviceSetLabel(arg0: WGPUDevice, arg1: WGPUStringView);
    pub(crate) fn wgpuDeviceAddRef(arg0: WGPUDevice);
    pub(crate) fn wgpuDeviceRelease(arg0: WGPUDevice);
    pub(crate) fn wgpuInstanceCreateSurface(arg0: WGPUInstance, arg1: *const WGPUSurfaceDescriptor) -> WGPUSurface;
    pub(crate) fn wgpuInstanceGetWGSLLanguageFeatures(arg0: WGPUInstance, arg1: *mut WGPUSupportedWGSLLanguageFeatures);
    pub(crate) fn wgpuInstanceHasWGSLLanguageFeature(arg0: WGPUInstance, arg1: WGPUWGSLLanguageFeatureName) -> WGPUBool;
    pub(crate) fn wgpuInstanceProcessEvents(arg0: WGPUInstance);
    pub(crate) fn wgpuInstanceRequestAdapter(arg0: WGPUInstance, arg1: *const WGPURequestAdapterOptions, arg2: WGPURequestAdapterCallbackInfo) -> WGPUFuture;
    pub(crate) fn wgpuInstanceWaitAny(arg0: WGPUInstance, arg1: usize, arg2: *mut WGPUFutureWaitInfo, arg3: u64) -> WGPUWaitStatus;
    pub(crate) fn wgpuInstanceAddRef(arg0: WGPUInstance);
    pub(crate) fn wgpuInstanceRelease(arg0: WGPUInstance);
    pub(crate) fn wgpuPipelineLayoutSetLabel(arg0: WGPUPipelineLayout, arg1: WGPUStringView);
    pub(crate) fn wgpuPipelineLayoutAddRef(arg0: WGPUPipelineLayout);
    pub(crate) fn wgpuPipelineLayoutRelease(arg0: WGPUPipelineLayout);
    pub(crate) fn wgpuQuerySetDestroy(arg0: WGPUQuerySet);
    pub(crate) fn wgpuQuerySetGetCount(arg0: WGPUQuerySet) -> u32;
    pub(crate) fn wgpuQuerySetGetType(arg0: WGPUQuerySet) -> WGPUQueryType;
    pub(crate) fn wgpuQuerySetSetLabel(arg0: WGPUQuerySet, arg1: WGPUStringView);
    pub(crate) fn wgpuQuerySetAddRef(arg0: WGPUQuerySet);
    pub(crate) fn wgpuQuerySetRelease(arg0: WGPUQuerySet);
    pub(crate) fn wgpuQueueOnSubmittedWorkDone(arg0: WGPUQueue, arg1: WGPUQueueWorkDoneCallbackInfo) -> WGPUFuture;
    pub(crate) fn wgpuQueueSetLabel(arg0: WGPUQueue, arg1: WGPUStringView);
    pub(crate) fn wgpuQueueSubmit(arg0: WGPUQueue, arg1: usize, arg2: *const WGPUCommandBuffer);
    pub(crate) fn wgpuQueueWriteBuffer(arg0: WGPUQueue, arg1: WGPUBuffer, arg2: u64, arg3: *const std::ffi::c_void, arg4: usize);
    pub(crate) fn wgpuQueueWriteTexture(arg0: WGPUQueue, arg1: *const WGPUTexelCopyTextureInfo, arg2: *const std::ffi::c_void, arg3: usize, arg4: *const WGPUTexelCopyBufferLayout, arg5: *const WGPUExtent3D);
    pub(crate) fn wgpuQueueAddRef(arg0: WGPUQueue);
    pub(crate) fn wgpuQueueRelease(arg0: WGPUQueue);
    pub(crate) fn wgpuRenderBundleSetLabel(arg0: WGPURenderBundle, arg1: WGPUStringView);
    pub(crate) fn wgpuRenderBundleAddRef(arg0: WGPURenderBundle);
    pub(crate) fn wgpuRenderBundleRelease(arg0: WGPURenderBundle);
    pub(crate) fn wgpuRenderBundleEncoderDraw(arg0: WGPURenderBundleEncoder, arg1: u32, arg2: u32, arg3: u32, arg4: u32);
    pub(crate) fn wgpuRenderBundleEncoderDrawIndexed(arg0: WGPURenderBundleEncoder, arg1: u32, arg2: u32, arg3: u32, arg4: i32, arg5: u32);
    pub(crate) fn wgpuRenderBundleEncoderDrawIndexedIndirect(arg0: WGPURenderBundleEncoder, arg1: WGPUBuffer, arg2: u64);
    pub(crate) fn wgpuRenderBundleEncoderDrawIndirect(arg0: WGPURenderBundleEncoder, arg1: WGPUBuffer, arg2: u64);
    pub(crate) fn wgpuRenderBundleEncoderFinish(arg0: WGPURenderBundleEncoder, arg1: *const WGPURenderBundleDescriptor) -> WGPURenderBundle;
    pub(crate) fn wgpuRenderBundleEncoderInsertDebugMarker(arg0: WGPURenderBundleEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuRenderBundleEncoderPopDebugGroup(arg0: WGPURenderBundleEncoder);
    pub(crate) fn wgpuRenderBundleEncoderPushDebugGroup(arg0: WGPURenderBundleEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuRenderBundleEncoderSetBindGroup(arg0: WGPURenderBundleEncoder, arg1: u32, arg2: WGPUBindGroup, arg3: usize, arg4: *const u32);
    pub(crate) fn wgpuRenderBundleEncoderSetIndexBuffer(arg0: WGPURenderBundleEncoder, arg1: WGPUBuffer, arg2: WGPUIndexFormat, arg3: u64, arg4: u64);
    pub(crate) fn wgpuRenderBundleEncoderSetLabel(arg0: WGPURenderBundleEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuRenderBundleEncoderSetPipeline(arg0: WGPURenderBundleEncoder, arg1: WGPURenderPipeline);
    pub(crate) fn wgpuRenderBundleEncoderSetVertexBuffer(arg0: WGPURenderBundleEncoder, arg1: u32, arg2: WGPUBuffer, arg3: u64, arg4: u64);
    pub(crate) fn wgpuRenderBundleEncoderAddRef(arg0: WGPURenderBundleEncoder);
    pub(crate) fn wgpuRenderBundleEncoderRelease(arg0: WGPURenderBundleEncoder);
    pub(crate) fn wgpuRenderPassEncoderBeginOcclusionQuery(arg0: WGPURenderPassEncoder, arg1: u32);
    pub(crate) fn wgpuRenderPassEncoderDraw(arg0: WGPURenderPassEncoder, arg1: u32, arg2: u32, arg3: u32, arg4: u32);
    pub(crate) fn wgpuRenderPassEncoderDrawIndexed(arg0: WGPURenderPassEncoder, arg1: u32, arg2: u32, arg3: u32, arg4: i32, arg5: u32);
    pub(crate) fn wgpuRenderPassEncoderDrawIndexedIndirect(arg0: WGPURenderPassEncoder, arg1: WGPUBuffer, arg2: u64);
    pub(crate) fn wgpuRenderPassEncoderDrawIndirect(arg0: WGPURenderPassEncoder, arg1: WGPUBuffer, arg2: u64);
    pub(crate) fn wgpuRenderPassEncoderEnd(arg0: WGPURenderPassEncoder);
    pub(crate) fn wgpuRenderPassEncoderEndOcclusionQuery(arg0: WGPURenderPassEncoder);
    pub(crate) fn wgpuRenderPassEncoderExecuteBundles(arg0: WGPURenderPassEncoder, arg1: usize, arg2: *const WGPURenderBundle);
    pub(crate) fn wgpuRenderPassEncoderInsertDebugMarker(arg0: WGPURenderPassEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuRenderPassEncoderMultiDrawIndexedIndirect(arg0: WGPURenderPassEncoder, arg1: WGPUBuffer, arg2: u64, arg3: u32, arg4: WGPUBuffer, arg5: u64);
    pub(crate) fn wgpuRenderPassEncoderMultiDrawIndirect(arg0: WGPURenderPassEncoder, arg1: WGPUBuffer, arg2: u64, arg3: u32, arg4: WGPUBuffer, arg5: u64);
    pub(crate) fn wgpuRenderPassEncoderPopDebugGroup(arg0: WGPURenderPassEncoder);
    pub(crate) fn wgpuRenderPassEncoderPushDebugGroup(arg0: WGPURenderPassEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuRenderPassEncoderSetBindGroup(arg0: WGPURenderPassEncoder, arg1: u32, arg2: WGPUBindGroup, arg3: usize, arg4: *const u32);
    pub(crate) fn wgpuRenderPassEncoderSetBlendConstant(arg0: WGPURenderPassEncoder, arg1: *const WGPUColor);
    pub(crate) fn wgpuRenderPassEncoderSetIndexBuffer(arg0: WGPURenderPassEncoder, arg1: WGPUBuffer, arg2: WGPUIndexFormat, arg3: u64, arg4: u64);
    pub(crate) fn wgpuRenderPassEncoderSetLabel(arg0: WGPURenderPassEncoder, arg1: WGPUStringView);
    pub(crate) fn wgpuRenderPassEncoderSetPipeline(arg0: WGPURenderPassEncoder, arg1: WGPURenderPipeline);
    pub(crate) fn wgpuRenderPassEncoderSetScissorRect(arg0: WGPURenderPassEncoder, arg1: u32, arg2: u32, arg3: u32, arg4: u32);
    pub(crate) fn wgpuRenderPassEncoderSetStencilReference(arg0: WGPURenderPassEncoder, arg1: u32);
    pub(crate) fn wgpuRenderPassEncoderSetVertexBuffer(arg0: WGPURenderPassEncoder, arg1: u32, arg2: WGPUBuffer, arg3: u64, arg4: u64);
    pub(crate) fn wgpuRenderPassEncoderSetViewport(arg0: WGPURenderPassEncoder, arg1: f32, arg2: f32, arg3: f32, arg4: f32, arg5: f32, arg6: f32);
    pub(crate) fn wgpuRenderPassEncoderWriteTimestamp(arg0: WGPURenderPassEncoder, arg1: WGPUQuerySet, arg2: u32);
    pub(crate) fn wgpuRenderPassEncoderAddRef(arg0: WGPURenderPassEncoder);
    pub(crate) fn wgpuRenderPassEncoderRelease(arg0: WGPURenderPassEncoder);
    pub(crate) fn wgpuRenderPipelineGetBindGroupLayout(arg0: WGPURenderPipeline, arg1: u32) -> WGPUBindGroupLayout;
    pub(crate) fn wgpuRenderPipelineSetLabel(arg0: WGPURenderPipeline, arg1: WGPUStringView);
    pub(crate) fn wgpuRenderPipelineAddRef(arg0: WGPURenderPipeline);
    pub(crate) fn wgpuRenderPipelineRelease(arg0: WGPURenderPipeline);
    pub(crate) fn wgpuSamplerSetLabel(arg0: WGPUSampler, arg1: WGPUStringView);
    pub(crate) fn wgpuSamplerAddRef(arg0: WGPUSampler);
    pub(crate) fn wgpuSamplerRelease(arg0: WGPUSampler);
    pub(crate) fn wgpuShaderModuleGetCompilationInfo(arg0: WGPUShaderModule, arg1: WGPUCompilationInfoCallbackInfo) -> WGPUFuture;
    pub(crate) fn wgpuShaderModuleSetLabel(arg0: WGPUShaderModule, arg1: WGPUStringView);
    pub(crate) fn wgpuShaderModuleAddRef(arg0: WGPUShaderModule);
    pub(crate) fn wgpuShaderModuleRelease(arg0: WGPUShaderModule);
    pub(crate) fn wgpuSupportedFeaturesFreeMembers(arg0: WGPUSupportedFeatures);
    pub(crate) fn wgpuSupportedInstanceFeaturesFreeMembers(arg0: WGPUSupportedInstanceFeatures);
    pub(crate) fn wgpuSupportedWGSLLanguageFeaturesFreeMembers(arg0: WGPUSupportedWGSLLanguageFeatures);
    pub(crate) fn wgpuSurfaceConfigure(arg0: WGPUSurface, arg1: *const WGPUSurfaceConfiguration);
    pub(crate) fn wgpuSurfaceGetCapabilities(arg0: WGPUSurface, arg1: WGPUAdapter, arg2: *mut WGPUSurfaceCapabilities) -> WGPUStatus;
    pub(crate) fn wgpuSurfaceGetCurrentTexture(arg0: WGPUSurface, arg1: *mut WGPUSurfaceTexture);
    pub(crate) fn wgpuSurfacePresent(arg0: WGPUSurface) -> WGPUStatus;
    pub(crate) fn wgpuSurfaceSetLabel(arg0: WGPUSurface, arg1: WGPUStringView);
    pub(crate) fn wgpuSurfaceUnconfigure(arg0: WGPUSurface);
    pub(crate) fn wgpuSurfaceAddRef(arg0: WGPUSurface);
    pub(crate) fn wgpuSurfaceRelease(arg0: WGPUSurface);
    pub(crate) fn wgpuSurfaceCapabilitiesFreeMembers(arg0: WGPUSurfaceCapabilities);
    pub(crate) fn wgpuTextureCreateView(arg0: WGPUTexture, arg1: *const WGPUTextureViewDescriptor) -> WGPUTextureView;
    pub(crate) fn wgpuTextureDestroy(arg0: WGPUTexture);
    pub(crate) fn wgpuTextureGetDepthOrArrayLayers(arg0: WGPUTexture) -> u32;
    pub(crate) fn wgpuTextureGetDimension(arg0: WGPUTexture) -> WGPUTextureDimension;
    pub(crate) fn wgpuTextureGetFormat(arg0: WGPUTexture) -> WGPUTextureFormat;
    pub(crate) fn wgpuTextureGetHeight(arg0: WGPUTexture) -> u32;
    pub(crate) fn wgpuTextureGetMipLevelCount(arg0: WGPUTexture) -> u32;
    pub(crate) fn wgpuTextureGetSampleCount(arg0: WGPUTexture) -> u32;
    pub(crate) fn wgpuTextureGetUsage(arg0: WGPUTexture) -> WGPUTextureUsage;
    pub(crate) fn wgpuTextureGetWidth(arg0: WGPUTexture) -> u32;
    pub(crate) fn wgpuTextureSetLabel(arg0: WGPUTexture, arg1: WGPUStringView);
    pub(crate) fn wgpuTextureAddRef(arg0: WGPUTexture);
    pub(crate) fn wgpuTextureRelease(arg0: WGPUTexture);
    pub(crate) fn wgpuTextureViewSetLabel(arg0: WGPUTextureView, arg1: WGPUStringView);
    pub(crate) fn wgpuTextureViewAddRef(arg0: WGPUTextureView);
    pub(crate) fn wgpuTextureViewRelease(arg0: WGPUTextureView);
}

pub(crate) const ABI_ENUM_COUNT: usize = 54;
pub(crate) const ABI_STRUCT_COUNT: usize = 87;
pub(crate) const ABI_FIELD_COUNT: usize = 420;
pub(crate) const ABI_HANDLE_COUNT: usize = 22;
pub(crate) const ABI_FUNCTION_POINTER_COUNT: usize = 210;
pub(crate) const ABI_STATIC_CONSTANT_COUNT: usize = 30;
pub(crate) const ABI_FUNCTION_COUNT: usize = 199;
pub(crate) const PREPROCESSOR_DEFINITION_COUNT: usize = 119;
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
                if name.starts_with("wgpu") { names.insert(name); }
            }
        }
        assert!(names.len() >= ABI_FUNCTION_COUNT);
    }
}
