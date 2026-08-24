//! Generated complete Rust translation of frozen Wagyu webgpu_cpp.h; do not hand edit.

#![allow(non_snake_case, non_upper_case_globals)]

use super::webgpu_cpp_chained_struct_decl::{ChainedStruct, ChainedStructOut, SType};
use super::webgpu_decl::*;
use super::webgpu_enum_class_bitmasks_decl::{impl_wgpu_bitmask_operators, IsWGPUBitmask};

pub(crate) const PINNED_SOURCE: &str = include_str!("source/renderer_src_webgpu_wagyu-port_include_webgpu_webgpu_cpp.h");
pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = 5833;
pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = 287_261;
pub(crate) const PINNED_SOURCE_SHA256: &str = "d58a0553dd853995594f2ccac691c7e02228263582ce5202f0fd931d48fc5f13";

pub(crate) const kArrayLayerCountUndefined: u32 = WGPU_ARRAY_LAYER_COUNT_UNDEFINED;
pub(crate) const kCopyStrideUndefined: u32 = WGPU_COPY_STRIDE_UNDEFINED;
pub(crate) const kDepthClearValueUndefined: f32 = f32::NAN;
pub(crate) const kDepthSliceUndefined: u32 = WGPU_DEPTH_SLICE_UNDEFINED;
pub(crate) const kLimitU32Undefined: u32 = WGPU_LIMIT_U32_UNDEFINED;
pub(crate) const kLimitU64Undefined: u64 = WGPU_LIMIT_U64_UNDEFINED;
pub(crate) const kMipLevelCountUndefined: u32 = WGPU_MIP_LEVEL_COUNT_UNDEFINED;
pub(crate) const kQuerySetIndexUndefined: u32 = WGPU_QUERY_SET_INDEX_UNDEFINED;
pub(crate) const kStrlen: usize = WGPU_STRLEN;
pub(crate) const kWholeMapSize: usize = WGPU_WHOLE_MAP_SIZE;
pub(crate) const kWholeSize: u64 = WGPU_WHOLE_SIZE;

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct AdapterType(pub(crate) u32);
impl AdapterType {
    pub(crate) const DiscreteGPU: Self = Self(WGPUAdapterType_DiscreteGPU as u32);
    pub(crate) const IntegratedGPU: Self = Self(WGPUAdapterType_IntegratedGPU as u32);
    pub(crate) const CPU: Self = Self(WGPUAdapterType_CPU as u32);
    pub(crate) const Unknown: Self = Self(WGPUAdapterType_Unknown as u32);
}

impl From<WGPUAdapterType> for AdapterType {
    fn from(value: WGPUAdapterType) -> Self { Self(value as u32) }
}
impl From<AdapterType> for WGPUAdapterType {
    fn from(value: AdapterType) -> Self { value.0 as WGPUAdapterType }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct AddressMode(pub(crate) u32);
impl AddressMode {
    pub(crate) const Undefined: Self = Self(WGPUAddressMode_Undefined as u32);
    pub(crate) const ClampToEdge: Self = Self(WGPUAddressMode_ClampToEdge as u32);
    pub(crate) const Repeat: Self = Self(WGPUAddressMode_Repeat as u32);
    pub(crate) const MirrorRepeat: Self = Self(WGPUAddressMode_MirrorRepeat as u32);
}

impl From<WGPUAddressMode> for AddressMode {
    fn from(value: WGPUAddressMode) -> Self { Self(value as u32) }
}
impl From<AddressMode> for WGPUAddressMode {
    fn from(value: AddressMode) -> Self { value.0 as WGPUAddressMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct BackendType(pub(crate) u32);
impl BackendType {
    pub(crate) const Undefined: Self = Self(WGPUBackendType_Undefined as u32);
    pub(crate) const Null: Self = Self(WGPUBackendType_Null as u32);
    pub(crate) const WebGPU: Self = Self(WGPUBackendType_WebGPU as u32);
    pub(crate) const D3D11: Self = Self(WGPUBackendType_D3D11 as u32);
    pub(crate) const D3D12: Self = Self(WGPUBackendType_D3D12 as u32);
    pub(crate) const Metal: Self = Self(WGPUBackendType_Metal as u32);
    pub(crate) const Vulkan: Self = Self(WGPUBackendType_Vulkan as u32);
    pub(crate) const OpenGL: Self = Self(WGPUBackendType_OpenGL as u32);
    pub(crate) const OpenGLES: Self = Self(WGPUBackendType_OpenGLES as u32);
}

impl From<WGPUBackendType> for BackendType {
    fn from(value: WGPUBackendType) -> Self { Self(value as u32) }
}
impl From<BackendType> for WGPUBackendType {
    fn from(value: BackendType) -> Self { value.0 as WGPUBackendType }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct BlendFactor(pub(crate) u32);
impl BlendFactor {
    pub(crate) const Undefined: Self = Self(WGPUBlendFactor_Undefined as u32);
    pub(crate) const Zero: Self = Self(WGPUBlendFactor_Zero as u32);
    pub(crate) const One: Self = Self(WGPUBlendFactor_One as u32);
    pub(crate) const Src: Self = Self(WGPUBlendFactor_Src as u32);
    pub(crate) const OneMinusSrc: Self = Self(WGPUBlendFactor_OneMinusSrc as u32);
    pub(crate) const SrcAlpha: Self = Self(WGPUBlendFactor_SrcAlpha as u32);
    pub(crate) const OneMinusSrcAlpha: Self = Self(WGPUBlendFactor_OneMinusSrcAlpha as u32);
    pub(crate) const Dst: Self = Self(WGPUBlendFactor_Dst as u32);
    pub(crate) const OneMinusDst: Self = Self(WGPUBlendFactor_OneMinusDst as u32);
    pub(crate) const DstAlpha: Self = Self(WGPUBlendFactor_DstAlpha as u32);
    pub(crate) const OneMinusDstAlpha: Self = Self(WGPUBlendFactor_OneMinusDstAlpha as u32);
    pub(crate) const SrcAlphaSaturated: Self = Self(WGPUBlendFactor_SrcAlphaSaturated as u32);
    pub(crate) const Constant: Self = Self(WGPUBlendFactor_Constant as u32);
    pub(crate) const OneMinusConstant: Self = Self(WGPUBlendFactor_OneMinusConstant as u32);
    pub(crate) const Src1: Self = Self(WGPUBlendFactor_Src1 as u32);
    pub(crate) const OneMinusSrc1: Self = Self(WGPUBlendFactor_OneMinusSrc1 as u32);
    pub(crate) const Src1Alpha: Self = Self(WGPUBlendFactor_Src1Alpha as u32);
    pub(crate) const OneMinusSrc1Alpha: Self = Self(WGPUBlendFactor_OneMinusSrc1Alpha as u32);
}

impl From<WGPUBlendFactor> for BlendFactor {
    fn from(value: WGPUBlendFactor) -> Self { Self(value as u32) }
}
impl From<BlendFactor> for WGPUBlendFactor {
    fn from(value: BlendFactor) -> Self { value.0 as WGPUBlendFactor }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct BlendOperation(pub(crate) u32);
impl BlendOperation {
    pub(crate) const Undefined: Self = Self(WGPUBlendOperation_Undefined as u32);
    pub(crate) const Add: Self = Self(WGPUBlendOperation_Add as u32);
    pub(crate) const Subtract: Self = Self(WGPUBlendOperation_Subtract as u32);
    pub(crate) const ReverseSubtract: Self = Self(WGPUBlendOperation_ReverseSubtract as u32);
    pub(crate) const Min: Self = Self(WGPUBlendOperation_Min as u32);
    pub(crate) const Max: Self = Self(WGPUBlendOperation_Max as u32);
}

impl From<WGPUBlendOperation> for BlendOperation {
    fn from(value: WGPUBlendOperation) -> Self { Self(value as u32) }
}
impl From<BlendOperation> for WGPUBlendOperation {
    fn from(value: BlendOperation) -> Self { value.0 as WGPUBlendOperation }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct BufferBindingType(pub(crate) u32);
impl BufferBindingType {
    pub(crate) const BindingNotUsed: Self = Self(WGPUBufferBindingType_BindingNotUsed as u32);
    pub(crate) const Undefined: Self = Self(WGPUBufferBindingType_Undefined as u32);
    pub(crate) const Uniform: Self = Self(WGPUBufferBindingType_Uniform as u32);
    pub(crate) const Storage: Self = Self(WGPUBufferBindingType_Storage as u32);
    pub(crate) const ReadOnlyStorage: Self = Self(WGPUBufferBindingType_ReadOnlyStorage as u32);
}

impl From<WGPUBufferBindingType> for BufferBindingType {
    fn from(value: WGPUBufferBindingType) -> Self { Self(value as u32) }
}
impl From<BufferBindingType> for WGPUBufferBindingType {
    fn from(value: BufferBindingType) -> Self { value.0 as WGPUBufferBindingType }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct BufferMapState(pub(crate) u32);
impl BufferMapState {
    pub(crate) const Unmapped: Self = Self(WGPUBufferMapState_Unmapped as u32);
    pub(crate) const Pending: Self = Self(WGPUBufferMapState_Pending as u32);
    pub(crate) const Mapped: Self = Self(WGPUBufferMapState_Mapped as u32);
}

impl From<WGPUBufferMapState> for BufferMapState {
    fn from(value: WGPUBufferMapState) -> Self { Self(value as u32) }
}
impl From<BufferMapState> for WGPUBufferMapState {
    fn from(value: BufferMapState) -> Self { value.0 as WGPUBufferMapState }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct CallbackMode(pub(crate) u32);
impl CallbackMode {
    pub(crate) const WaitAnyOnly: Self = Self(WGPUCallbackMode_WaitAnyOnly as u32);
    pub(crate) const AllowProcessEvents: Self = Self(WGPUCallbackMode_AllowProcessEvents as u32);
    pub(crate) const AllowSpontaneous: Self = Self(WGPUCallbackMode_AllowSpontaneous as u32);
}

impl From<WGPUCallbackMode> for CallbackMode {
    fn from(value: WGPUCallbackMode) -> Self { Self(value as u32) }
}
impl From<CallbackMode> for WGPUCallbackMode {
    fn from(value: CallbackMode) -> Self { value.0 as WGPUCallbackMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct CompareFunction(pub(crate) u32);
impl CompareFunction {
    pub(crate) const Undefined: Self = Self(WGPUCompareFunction_Undefined as u32);
    pub(crate) const Never: Self = Self(WGPUCompareFunction_Never as u32);
    pub(crate) const Less: Self = Self(WGPUCompareFunction_Less as u32);
    pub(crate) const Equal: Self = Self(WGPUCompareFunction_Equal as u32);
    pub(crate) const LessEqual: Self = Self(WGPUCompareFunction_LessEqual as u32);
    pub(crate) const Greater: Self = Self(WGPUCompareFunction_Greater as u32);
    pub(crate) const NotEqual: Self = Self(WGPUCompareFunction_NotEqual as u32);
    pub(crate) const GreaterEqual: Self = Self(WGPUCompareFunction_GreaterEqual as u32);
    pub(crate) const Always: Self = Self(WGPUCompareFunction_Always as u32);
}

impl From<WGPUCompareFunction> for CompareFunction {
    fn from(value: WGPUCompareFunction) -> Self { Self(value as u32) }
}
impl From<CompareFunction> for WGPUCompareFunction {
    fn from(value: CompareFunction) -> Self { value.0 as WGPUCompareFunction }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct CompilationInfoRequestStatus(pub(crate) u32);
impl CompilationInfoRequestStatus {
    pub(crate) const Success: Self = Self(WGPUCompilationInfoRequestStatus_Success as u32);
    pub(crate) const CallbackCancelled: Self = Self(WGPUCompilationInfoRequestStatus_CallbackCancelled as u32);
}

impl From<WGPUCompilationInfoRequestStatus> for CompilationInfoRequestStatus {
    fn from(value: WGPUCompilationInfoRequestStatus) -> Self { Self(value as u32) }
}
impl From<CompilationInfoRequestStatus> for WGPUCompilationInfoRequestStatus {
    fn from(value: CompilationInfoRequestStatus) -> Self { value.0 as WGPUCompilationInfoRequestStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct CompilationMessageType(pub(crate) u32);
impl CompilationMessageType {
    pub(crate) const Error: Self = Self(WGPUCompilationMessageType_Error as u32);
    pub(crate) const Warning: Self = Self(WGPUCompilationMessageType_Warning as u32);
    pub(crate) const Info: Self = Self(WGPUCompilationMessageType_Info as u32);
}

impl From<WGPUCompilationMessageType> for CompilationMessageType {
    fn from(value: WGPUCompilationMessageType) -> Self { Self(value as u32) }
}
impl From<CompilationMessageType> for WGPUCompilationMessageType {
    fn from(value: CompilationMessageType) -> Self { value.0 as WGPUCompilationMessageType }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ComponentSwizzle(pub(crate) u32);
impl ComponentSwizzle {
    pub(crate) const Undefined: Self = Self(WGPUComponentSwizzle_Undefined as u32);
    pub(crate) const Zero: Self = Self(WGPUComponentSwizzle_Zero as u32);
    pub(crate) const One: Self = Self(WGPUComponentSwizzle_One as u32);
    pub(crate) const R: Self = Self(WGPUComponentSwizzle_R as u32);
    pub(crate) const G: Self = Self(WGPUComponentSwizzle_G as u32);
    pub(crate) const B: Self = Self(WGPUComponentSwizzle_B as u32);
    pub(crate) const A: Self = Self(WGPUComponentSwizzle_A as u32);
}

impl From<WGPUComponentSwizzle> for ComponentSwizzle {
    fn from(value: WGPUComponentSwizzle) -> Self { Self(value as u32) }
}
impl From<ComponentSwizzle> for WGPUComponentSwizzle {
    fn from(value: ComponentSwizzle) -> Self { value.0 as WGPUComponentSwizzle }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct CompositeAlphaMode(pub(crate) u32);
impl CompositeAlphaMode {
    pub(crate) const Auto: Self = Self(WGPUCompositeAlphaMode_Auto as u32);
    pub(crate) const Opaque: Self = Self(WGPUCompositeAlphaMode_Opaque as u32);
    pub(crate) const Premultiplied: Self = Self(WGPUCompositeAlphaMode_Premultiplied as u32);
    pub(crate) const Unpremultiplied: Self = Self(WGPUCompositeAlphaMode_Unpremultiplied as u32);
    pub(crate) const Inherit: Self = Self(WGPUCompositeAlphaMode_Inherit as u32);
}

impl From<WGPUCompositeAlphaMode> for CompositeAlphaMode {
    fn from(value: WGPUCompositeAlphaMode) -> Self { Self(value as u32) }
}
impl From<CompositeAlphaMode> for WGPUCompositeAlphaMode {
    fn from(value: CompositeAlphaMode) -> Self { value.0 as WGPUCompositeAlphaMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct CreatePipelineAsyncStatus(pub(crate) u32);
impl CreatePipelineAsyncStatus {
    pub(crate) const Success: Self = Self(WGPUCreatePipelineAsyncStatus_Success as u32);
    pub(crate) const CallbackCancelled: Self = Self(WGPUCreatePipelineAsyncStatus_CallbackCancelled as u32);
    pub(crate) const ValidationError: Self = Self(WGPUCreatePipelineAsyncStatus_ValidationError as u32);
    pub(crate) const InternalError: Self = Self(WGPUCreatePipelineAsyncStatus_InternalError as u32);
}

impl From<WGPUCreatePipelineAsyncStatus> for CreatePipelineAsyncStatus {
    fn from(value: WGPUCreatePipelineAsyncStatus) -> Self { Self(value as u32) }
}
impl From<CreatePipelineAsyncStatus> for WGPUCreatePipelineAsyncStatus {
    fn from(value: CreatePipelineAsyncStatus) -> Self { value.0 as WGPUCreatePipelineAsyncStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct CullMode(pub(crate) u32);
impl CullMode {
    pub(crate) const Undefined: Self = Self(WGPUCullMode_Undefined as u32);
    pub(crate) const None: Self = Self(WGPUCullMode_None as u32);
    pub(crate) const Front: Self = Self(WGPUCullMode_Front as u32);
    pub(crate) const Back: Self = Self(WGPUCullMode_Back as u32);
}

impl From<WGPUCullMode> for CullMode {
    fn from(value: WGPUCullMode) -> Self { Self(value as u32) }
}
impl From<CullMode> for WGPUCullMode {
    fn from(value: CullMode) -> Self { value.0 as WGPUCullMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct DeviceLostReason(pub(crate) u32);
impl DeviceLostReason {
    pub(crate) const Unknown: Self = Self(WGPUDeviceLostReason_Unknown as u32);
    pub(crate) const Destroyed: Self = Self(WGPUDeviceLostReason_Destroyed as u32);
    pub(crate) const CallbackCancelled: Self = Self(WGPUDeviceLostReason_CallbackCancelled as u32);
    pub(crate) const FailedCreation: Self = Self(WGPUDeviceLostReason_FailedCreation as u32);
}

impl From<WGPUDeviceLostReason> for DeviceLostReason {
    fn from(value: WGPUDeviceLostReason) -> Self { Self(value as u32) }
}
impl From<DeviceLostReason> for WGPUDeviceLostReason {
    fn from(value: DeviceLostReason) -> Self { value.0 as WGPUDeviceLostReason }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ErrorFilter(pub(crate) u32);
impl ErrorFilter {
    pub(crate) const Validation: Self = Self(WGPUErrorFilter_Validation as u32);
    pub(crate) const OutOfMemory: Self = Self(WGPUErrorFilter_OutOfMemory as u32);
    pub(crate) const Internal: Self = Self(WGPUErrorFilter_Internal as u32);
}

impl From<WGPUErrorFilter> for ErrorFilter {
    fn from(value: WGPUErrorFilter) -> Self { Self(value as u32) }
}
impl From<ErrorFilter> for WGPUErrorFilter {
    fn from(value: ErrorFilter) -> Self { value.0 as WGPUErrorFilter }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ErrorType(pub(crate) u32);
impl ErrorType {
    pub(crate) const NoError: Self = Self(WGPUErrorType_NoError as u32);
    pub(crate) const Validation: Self = Self(WGPUErrorType_Validation as u32);
    pub(crate) const OutOfMemory: Self = Self(WGPUErrorType_OutOfMemory as u32);
    pub(crate) const Internal: Self = Self(WGPUErrorType_Internal as u32);
    pub(crate) const Unknown: Self = Self(WGPUErrorType_Unknown as u32);
}

impl From<WGPUErrorType> for ErrorType {
    fn from(value: WGPUErrorType) -> Self { Self(value as u32) }
}
impl From<ErrorType> for WGPUErrorType {
    fn from(value: ErrorType) -> Self { value.0 as WGPUErrorType }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct FeatureLevel(pub(crate) u32);
impl FeatureLevel {
    pub(crate) const Undefined: Self = Self(WGPUFeatureLevel_Undefined as u32);
    pub(crate) const Compatibility: Self = Self(WGPUFeatureLevel_Compatibility as u32);
    pub(crate) const Core: Self = Self(WGPUFeatureLevel_Core as u32);
}

impl From<WGPUFeatureLevel> for FeatureLevel {
    fn from(value: WGPUFeatureLevel) -> Self { Self(value as u32) }
}
impl From<FeatureLevel> for WGPUFeatureLevel {
    fn from(value: FeatureLevel) -> Self { value.0 as WGPUFeatureLevel }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct FeatureName(pub(crate) u32);
impl FeatureName {
    pub(crate) const CoreFeaturesAndLimits: Self = Self(WGPUFeatureName_CoreFeaturesAndLimits as u32);
    pub(crate) const DepthClipControl: Self = Self(WGPUFeatureName_DepthClipControl as u32);
    pub(crate) const Depth32FloatStencil8: Self = Self(WGPUFeatureName_Depth32FloatStencil8 as u32);
    pub(crate) const TextureCompressionBC: Self = Self(WGPUFeatureName_TextureCompressionBC as u32);
    pub(crate) const TextureCompressionBCSliced3D: Self = Self(WGPUFeatureName_TextureCompressionBCSliced3D as u32);
    pub(crate) const TextureCompressionETC2: Self = Self(WGPUFeatureName_TextureCompressionETC2 as u32);
    pub(crate) const TextureCompressionASTC: Self = Self(WGPUFeatureName_TextureCompressionASTC as u32);
    pub(crate) const TextureCompressionASTCSliced3D: Self = Self(WGPUFeatureName_TextureCompressionASTCSliced3D as u32);
    pub(crate) const TimestampQuery: Self = Self(WGPUFeatureName_TimestampQuery as u32);
    pub(crate) const IndirectFirstInstance: Self = Self(WGPUFeatureName_IndirectFirstInstance as u32);
    pub(crate) const ShaderF16: Self = Self(WGPUFeatureName_ShaderF16 as u32);
    pub(crate) const RG11B10UfloatRenderable: Self = Self(WGPUFeatureName_RG11B10UfloatRenderable as u32);
    pub(crate) const BGRA8UnormStorage: Self = Self(WGPUFeatureName_BGRA8UnormStorage as u32);
    pub(crate) const Float32Filterable: Self = Self(WGPUFeatureName_Float32Filterable as u32);
    pub(crate) const Float32Blendable: Self = Self(WGPUFeatureName_Float32Blendable as u32);
    pub(crate) const ClipDistances: Self = Self(WGPUFeatureName_ClipDistances as u32);
    pub(crate) const DualSourceBlending: Self = Self(WGPUFeatureName_DualSourceBlending as u32);
    pub(crate) const Subgroups: Self = Self(WGPUFeatureName_Subgroups as u32);
    pub(crate) const TextureFormatsTier1: Self = Self(WGPUFeatureName_TextureFormatsTier1 as u32);
    pub(crate) const TextureFormatsTier2: Self = Self(WGPUFeatureName_TextureFormatsTier2 as u32);
    pub(crate) const PrimitiveIndex: Self = Self(WGPUFeatureName_PrimitiveIndex as u32);
    pub(crate) const TextureComponentSwizzle: Self = Self(WGPUFeatureName_TextureComponentSwizzle as u32);
    pub(crate) const Unorm16TextureFormats: Self = Self(WGPUFeatureName_Unorm16TextureFormats as u32);
    pub(crate) const Snorm16TextureFormats: Self = Self(WGPUFeatureName_Snorm16TextureFormats as u32);
    pub(crate) const MultiDrawIndirect: Self = Self(WGPUFeatureName_MultiDrawIndirect as u32);
}

impl From<WGPUFeatureName> for FeatureName {
    fn from(value: WGPUFeatureName) -> Self { Self(value as u32) }
}
impl From<FeatureName> for WGPUFeatureName {
    fn from(value: FeatureName) -> Self { value.0 as WGPUFeatureName }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct FilterMode(pub(crate) u32);
impl FilterMode {
    pub(crate) const Undefined: Self = Self(WGPUFilterMode_Undefined as u32);
    pub(crate) const Nearest: Self = Self(WGPUFilterMode_Nearest as u32);
    pub(crate) const Linear: Self = Self(WGPUFilterMode_Linear as u32);
}

impl From<WGPUFilterMode> for FilterMode {
    fn from(value: WGPUFilterMode) -> Self { Self(value as u32) }
}
impl From<FilterMode> for WGPUFilterMode {
    fn from(value: FilterMode) -> Self { value.0 as WGPUFilterMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct FrontFace(pub(crate) u32);
impl FrontFace {
    pub(crate) const Undefined: Self = Self(WGPUFrontFace_Undefined as u32);
    pub(crate) const CCW: Self = Self(WGPUFrontFace_CCW as u32);
    pub(crate) const CW: Self = Self(WGPUFrontFace_CW as u32);
}

impl From<WGPUFrontFace> for FrontFace {
    fn from(value: WGPUFrontFace) -> Self { Self(value as u32) }
}
impl From<FrontFace> for WGPUFrontFace {
    fn from(value: FrontFace) -> Self { value.0 as WGPUFrontFace }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct IndexFormat(pub(crate) u32);
impl IndexFormat {
    pub(crate) const Undefined: Self = Self(WGPUIndexFormat_Undefined as u32);
    pub(crate) const Uint16: Self = Self(WGPUIndexFormat_Uint16 as u32);
    pub(crate) const Uint32: Self = Self(WGPUIndexFormat_Uint32 as u32);
}

impl From<WGPUIndexFormat> for IndexFormat {
    fn from(value: WGPUIndexFormat) -> Self { Self(value as u32) }
}
impl From<IndexFormat> for WGPUIndexFormat {
    fn from(value: IndexFormat) -> Self { value.0 as WGPUIndexFormat }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct InstanceFeatureName(pub(crate) u32);
impl InstanceFeatureName {
    pub(crate) const TimedWaitAny: Self = Self(WGPUInstanceFeatureName_TimedWaitAny as u32);
    pub(crate) const ShaderSourceSPIRV: Self = Self(WGPUInstanceFeatureName_ShaderSourceSPIRV as u32);
    pub(crate) const MultipleDevicesPerAdapter: Self = Self(WGPUInstanceFeatureName_MultipleDevicesPerAdapter as u32);
}

impl From<WGPUInstanceFeatureName> for InstanceFeatureName {
    fn from(value: WGPUInstanceFeatureName) -> Self { Self(value as u32) }
}
impl From<InstanceFeatureName> for WGPUInstanceFeatureName {
    fn from(value: InstanceFeatureName) -> Self { value.0 as WGPUInstanceFeatureName }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct LoadOp(pub(crate) u32);
impl LoadOp {
    pub(crate) const Undefined: Self = Self(WGPULoadOp_Undefined as u32);
    pub(crate) const Load: Self = Self(WGPULoadOp_Load as u32);
    pub(crate) const Clear: Self = Self(WGPULoadOp_Clear as u32);
}

impl From<WGPULoadOp> for LoadOp {
    fn from(value: WGPULoadOp) -> Self { Self(value as u32) }
}
impl From<LoadOp> for WGPULoadOp {
    fn from(value: LoadOp) -> Self { value.0 as WGPULoadOp }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct MapAsyncStatus(pub(crate) u32);
impl MapAsyncStatus {
    pub(crate) const Success: Self = Self(WGPUMapAsyncStatus_Success as u32);
    pub(crate) const CallbackCancelled: Self = Self(WGPUMapAsyncStatus_CallbackCancelled as u32);
    pub(crate) const Error: Self = Self(WGPUMapAsyncStatus_Error as u32);
    pub(crate) const Aborted: Self = Self(WGPUMapAsyncStatus_Aborted as u32);
}

impl From<WGPUMapAsyncStatus> for MapAsyncStatus {
    fn from(value: WGPUMapAsyncStatus) -> Self { Self(value as u32) }
}
impl From<MapAsyncStatus> for WGPUMapAsyncStatus {
    fn from(value: MapAsyncStatus) -> Self { value.0 as WGPUMapAsyncStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct MipmapFilterMode(pub(crate) u32);
impl MipmapFilterMode {
    pub(crate) const Undefined: Self = Self(WGPUMipmapFilterMode_Undefined as u32);
    pub(crate) const Nearest: Self = Self(WGPUMipmapFilterMode_Nearest as u32);
    pub(crate) const Linear: Self = Self(WGPUMipmapFilterMode_Linear as u32);
}

impl From<WGPUMipmapFilterMode> for MipmapFilterMode {
    fn from(value: WGPUMipmapFilterMode) -> Self { Self(value as u32) }
}
impl From<MipmapFilterMode> for WGPUMipmapFilterMode {
    fn from(value: MipmapFilterMode) -> Self { value.0 as WGPUMipmapFilterMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct PopErrorScopeStatus(pub(crate) u32);
impl PopErrorScopeStatus {
    pub(crate) const Success: Self = Self(WGPUPopErrorScopeStatus_Success as u32);
    pub(crate) const CallbackCancelled: Self = Self(WGPUPopErrorScopeStatus_CallbackCancelled as u32);
    pub(crate) const Error: Self = Self(WGPUPopErrorScopeStatus_Error as u32);
}

impl From<WGPUPopErrorScopeStatus> for PopErrorScopeStatus {
    fn from(value: WGPUPopErrorScopeStatus) -> Self { Self(value as u32) }
}
impl From<PopErrorScopeStatus> for WGPUPopErrorScopeStatus {
    fn from(value: PopErrorScopeStatus) -> Self { value.0 as WGPUPopErrorScopeStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct PowerPreference(pub(crate) u32);
impl PowerPreference {
    pub(crate) const Undefined: Self = Self(WGPUPowerPreference_Undefined as u32);
    pub(crate) const LowPower: Self = Self(WGPUPowerPreference_LowPower as u32);
    pub(crate) const HighPerformance: Self = Self(WGPUPowerPreference_HighPerformance as u32);
}

impl From<WGPUPowerPreference> for PowerPreference {
    fn from(value: WGPUPowerPreference) -> Self { Self(value as u32) }
}
impl From<PowerPreference> for WGPUPowerPreference {
    fn from(value: PowerPreference) -> Self { value.0 as WGPUPowerPreference }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct PredefinedColorSpace(pub(crate) u32);
impl PredefinedColorSpace {
    pub(crate) const SRGB: Self = Self(WGPUPredefinedColorSpace_SRGB as u32);
    pub(crate) const DisplayP3: Self = Self(WGPUPredefinedColorSpace_DisplayP3 as u32);
}

impl From<WGPUPredefinedColorSpace> for PredefinedColorSpace {
    fn from(value: WGPUPredefinedColorSpace) -> Self { Self(value as u32) }
}
impl From<PredefinedColorSpace> for WGPUPredefinedColorSpace {
    fn from(value: PredefinedColorSpace) -> Self { value.0 as WGPUPredefinedColorSpace }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct PresentMode(pub(crate) u32);
impl PresentMode {
    pub(crate) const Undefined: Self = Self(WGPUPresentMode_Undefined as u32);
    pub(crate) const Fifo: Self = Self(WGPUPresentMode_Fifo as u32);
    pub(crate) const FifoRelaxed: Self = Self(WGPUPresentMode_FifoRelaxed as u32);
    pub(crate) const Immediate: Self = Self(WGPUPresentMode_Immediate as u32);
    pub(crate) const Mailbox: Self = Self(WGPUPresentMode_Mailbox as u32);
}

impl From<WGPUPresentMode> for PresentMode {
    fn from(value: WGPUPresentMode) -> Self { Self(value as u32) }
}
impl From<PresentMode> for WGPUPresentMode {
    fn from(value: PresentMode) -> Self { value.0 as WGPUPresentMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct PrimitiveTopology(pub(crate) u32);
impl PrimitiveTopology {
    pub(crate) const Undefined: Self = Self(WGPUPrimitiveTopology_Undefined as u32);
    pub(crate) const PointList: Self = Self(WGPUPrimitiveTopology_PointList as u32);
    pub(crate) const LineList: Self = Self(WGPUPrimitiveTopology_LineList as u32);
    pub(crate) const LineStrip: Self = Self(WGPUPrimitiveTopology_LineStrip as u32);
    pub(crate) const TriangleList: Self = Self(WGPUPrimitiveTopology_TriangleList as u32);
    pub(crate) const TriangleStrip: Self = Self(WGPUPrimitiveTopology_TriangleStrip as u32);
}

impl From<WGPUPrimitiveTopology> for PrimitiveTopology {
    fn from(value: WGPUPrimitiveTopology) -> Self { Self(value as u32) }
}
impl From<PrimitiveTopology> for WGPUPrimitiveTopology {
    fn from(value: PrimitiveTopology) -> Self { value.0 as WGPUPrimitiveTopology }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct QueryType(pub(crate) u32);
impl QueryType {
    pub(crate) const Occlusion: Self = Self(WGPUQueryType_Occlusion as u32);
    pub(crate) const Timestamp: Self = Self(WGPUQueryType_Timestamp as u32);
}

impl From<WGPUQueryType> for QueryType {
    fn from(value: WGPUQueryType) -> Self { Self(value as u32) }
}
impl From<QueryType> for WGPUQueryType {
    fn from(value: QueryType) -> Self { value.0 as WGPUQueryType }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct QueueWorkDoneStatus(pub(crate) u32);
impl QueueWorkDoneStatus {
    pub(crate) const Success: Self = Self(WGPUQueueWorkDoneStatus_Success as u32);
    pub(crate) const CallbackCancelled: Self = Self(WGPUQueueWorkDoneStatus_CallbackCancelled as u32);
    pub(crate) const Error: Self = Self(WGPUQueueWorkDoneStatus_Error as u32);
}

impl From<WGPUQueueWorkDoneStatus> for QueueWorkDoneStatus {
    fn from(value: WGPUQueueWorkDoneStatus) -> Self { Self(value as u32) }
}
impl From<QueueWorkDoneStatus> for WGPUQueueWorkDoneStatus {
    fn from(value: QueueWorkDoneStatus) -> Self { value.0 as WGPUQueueWorkDoneStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct RequestAdapterStatus(pub(crate) u32);
impl RequestAdapterStatus {
    pub(crate) const Success: Self = Self(WGPURequestAdapterStatus_Success as u32);
    pub(crate) const CallbackCancelled: Self = Self(WGPURequestAdapterStatus_CallbackCancelled as u32);
    pub(crate) const Unavailable: Self = Self(WGPURequestAdapterStatus_Unavailable as u32);
    pub(crate) const Error: Self = Self(WGPURequestAdapterStatus_Error as u32);
}

impl From<WGPURequestAdapterStatus> for RequestAdapterStatus {
    fn from(value: WGPURequestAdapterStatus) -> Self { Self(value as u32) }
}
impl From<RequestAdapterStatus> for WGPURequestAdapterStatus {
    fn from(value: RequestAdapterStatus) -> Self { value.0 as WGPURequestAdapterStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct RequestDeviceStatus(pub(crate) u32);
impl RequestDeviceStatus {
    pub(crate) const Success: Self = Self(WGPURequestDeviceStatus_Success as u32);
    pub(crate) const CallbackCancelled: Self = Self(WGPURequestDeviceStatus_CallbackCancelled as u32);
    pub(crate) const Error: Self = Self(WGPURequestDeviceStatus_Error as u32);
}

impl From<WGPURequestDeviceStatus> for RequestDeviceStatus {
    fn from(value: WGPURequestDeviceStatus) -> Self { Self(value as u32) }
}
impl From<RequestDeviceStatus> for WGPURequestDeviceStatus {
    fn from(value: RequestDeviceStatus) -> Self { value.0 as WGPURequestDeviceStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct SamplerBindingType(pub(crate) u32);
impl SamplerBindingType {
    pub(crate) const BindingNotUsed: Self = Self(WGPUSamplerBindingType_BindingNotUsed as u32);
    pub(crate) const Undefined: Self = Self(WGPUSamplerBindingType_Undefined as u32);
    pub(crate) const Filtering: Self = Self(WGPUSamplerBindingType_Filtering as u32);
    pub(crate) const NonFiltering: Self = Self(WGPUSamplerBindingType_NonFiltering as u32);
    pub(crate) const Comparison: Self = Self(WGPUSamplerBindingType_Comparison as u32);
}

impl From<WGPUSamplerBindingType> for SamplerBindingType {
    fn from(value: WGPUSamplerBindingType) -> Self { Self(value as u32) }
}
impl From<SamplerBindingType> for WGPUSamplerBindingType {
    fn from(value: SamplerBindingType) -> Self { value.0 as WGPUSamplerBindingType }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Status(pub(crate) u32);
impl Status {
    pub(crate) const Success: Self = Self(WGPUStatus_Success as u32);
    pub(crate) const Error: Self = Self(WGPUStatus_Error as u32);
}

impl From<WGPUStatus> for Status {
    fn from(value: WGPUStatus) -> Self { Self(value as u32) }
}
impl From<Status> for WGPUStatus {
    fn from(value: Status) -> Self { value.0 as WGPUStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct StencilOperation(pub(crate) u32);
impl StencilOperation {
    pub(crate) const Undefined: Self = Self(WGPUStencilOperation_Undefined as u32);
    pub(crate) const Keep: Self = Self(WGPUStencilOperation_Keep as u32);
    pub(crate) const Zero: Self = Self(WGPUStencilOperation_Zero as u32);
    pub(crate) const Replace: Self = Self(WGPUStencilOperation_Replace as u32);
    pub(crate) const Invert: Self = Self(WGPUStencilOperation_Invert as u32);
    pub(crate) const IncrementClamp: Self = Self(WGPUStencilOperation_IncrementClamp as u32);
    pub(crate) const DecrementClamp: Self = Self(WGPUStencilOperation_DecrementClamp as u32);
    pub(crate) const IncrementWrap: Self = Self(WGPUStencilOperation_IncrementWrap as u32);
    pub(crate) const DecrementWrap: Self = Self(WGPUStencilOperation_DecrementWrap as u32);
}

impl From<WGPUStencilOperation> for StencilOperation {
    fn from(value: WGPUStencilOperation) -> Self { Self(value as u32) }
}
impl From<StencilOperation> for WGPUStencilOperation {
    fn from(value: StencilOperation) -> Self { value.0 as WGPUStencilOperation }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct StorageTextureAccess(pub(crate) u32);
impl StorageTextureAccess {
    pub(crate) const BindingNotUsed: Self = Self(WGPUStorageTextureAccess_BindingNotUsed as u32);
    pub(crate) const Undefined: Self = Self(WGPUStorageTextureAccess_Undefined as u32);
    pub(crate) const WriteOnly: Self = Self(WGPUStorageTextureAccess_WriteOnly as u32);
    pub(crate) const ReadOnly: Self = Self(WGPUStorageTextureAccess_ReadOnly as u32);
    pub(crate) const ReadWrite: Self = Self(WGPUStorageTextureAccess_ReadWrite as u32);
}

impl From<WGPUStorageTextureAccess> for StorageTextureAccess {
    fn from(value: WGPUStorageTextureAccess) -> Self { Self(value as u32) }
}
impl From<StorageTextureAccess> for WGPUStorageTextureAccess {
    fn from(value: StorageTextureAccess) -> Self { value.0 as WGPUStorageTextureAccess }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct StoreOp(pub(crate) u32);
impl StoreOp {
    pub(crate) const Undefined: Self = Self(WGPUStoreOp_Undefined as u32);
    pub(crate) const Store: Self = Self(WGPUStoreOp_Store as u32);
    pub(crate) const Discard: Self = Self(WGPUStoreOp_Discard as u32);
}

impl From<WGPUStoreOp> for StoreOp {
    fn from(value: WGPUStoreOp) -> Self { Self(value as u32) }
}
impl From<StoreOp> for WGPUStoreOp {
    fn from(value: StoreOp) -> Self { value.0 as WGPUStoreOp }
}

impl SType {
    pub(crate) const ShaderSourceSPIRV: Self = Self(WGPUSType_ShaderSourceSPIRV as u32);
    pub(crate) const ShaderSourceWGSL: Self = Self(WGPUSType_ShaderSourceWGSL as u32);
    pub(crate) const RenderPassMaxDrawCount: Self = Self(WGPUSType_RenderPassMaxDrawCount as u32);
    pub(crate) const SurfaceSourceMetalLayer: Self = Self(WGPUSType_SurfaceSourceMetalLayer as u32);
    pub(crate) const SurfaceSourceWindowsHWND: Self = Self(WGPUSType_SurfaceSourceWindowsHWND as u32);
    pub(crate) const SurfaceSourceXlibWindow: Self = Self(WGPUSType_SurfaceSourceXlibWindow as u32);
    pub(crate) const SurfaceSourceWaylandSurface: Self = Self(WGPUSType_SurfaceSourceWaylandSurface as u32);
    pub(crate) const SurfaceSourceAndroidNativeWindow: Self = Self(WGPUSType_SurfaceSourceAndroidNativeWindow as u32);
    pub(crate) const SurfaceSourceXCBWindow: Self = Self(WGPUSType_SurfaceSourceXCBWindow as u32);
    pub(crate) const SurfaceColorManagement: Self = Self(WGPUSType_SurfaceColorManagement as u32);
    pub(crate) const RequestAdapterWebXROptions: Self = Self(WGPUSType_RequestAdapterWebXROptions as u32);
    pub(crate) const TextureComponentSwizzleDescriptor: Self = Self(WGPUSType_TextureComponentSwizzleDescriptor as u32);
    pub(crate) const CompatibilityModeLimits: Self = Self(WGPUSType_CompatibilityModeLimits as u32);
    pub(crate) const TextureBindingViewDimensionDescriptor: Self = Self(WGPUSType_TextureBindingViewDimensionDescriptor as u32);
    pub(crate) const EmscriptenSurfaceSourceCanvasHTMLSelector: Self = Self(WGPUSType_EmscriptenSurfaceSourceCanvasHTMLSelector as u32);
    pub(crate) const DawnCompilationMessageUtf16: Self = Self(WGPUSType_DawnCompilationMessageUtf16 as u32);
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct SurfaceGetCurrentTextureStatus(pub(crate) u32);
impl SurfaceGetCurrentTextureStatus {
    pub(crate) const SuccessOptimal: Self = Self(WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal as u32);
    pub(crate) const SuccessSuboptimal: Self = Self(WGPUSurfaceGetCurrentTextureStatus_SuccessSuboptimal as u32);
    pub(crate) const Timeout: Self = Self(WGPUSurfaceGetCurrentTextureStatus_Timeout as u32);
    pub(crate) const Outdated: Self = Self(WGPUSurfaceGetCurrentTextureStatus_Outdated as u32);
    pub(crate) const Lost: Self = Self(WGPUSurfaceGetCurrentTextureStatus_Lost as u32);
    pub(crate) const Error: Self = Self(WGPUSurfaceGetCurrentTextureStatus_Error as u32);
}

impl From<WGPUSurfaceGetCurrentTextureStatus> for SurfaceGetCurrentTextureStatus {
    fn from(value: WGPUSurfaceGetCurrentTextureStatus) -> Self { Self(value as u32) }
}
impl From<SurfaceGetCurrentTextureStatus> for WGPUSurfaceGetCurrentTextureStatus {
    fn from(value: SurfaceGetCurrentTextureStatus) -> Self { value.0 as WGPUSurfaceGetCurrentTextureStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct TextureAspect(pub(crate) u32);
impl TextureAspect {
    pub(crate) const Undefined: Self = Self(WGPUTextureAspect_Undefined as u32);
    pub(crate) const All: Self = Self(WGPUTextureAspect_All as u32);
    pub(crate) const StencilOnly: Self = Self(WGPUTextureAspect_StencilOnly as u32);
    pub(crate) const DepthOnly: Self = Self(WGPUTextureAspect_DepthOnly as u32);
}

impl From<WGPUTextureAspect> for TextureAspect {
    fn from(value: WGPUTextureAspect) -> Self { Self(value as u32) }
}
impl From<TextureAspect> for WGPUTextureAspect {
    fn from(value: TextureAspect) -> Self { value.0 as WGPUTextureAspect }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct TextureDimension(pub(crate) u32);
impl TextureDimension {
    pub(crate) const Undefined: Self = Self(WGPUTextureDimension_Undefined as u32);
    pub(crate) const e1D: Self = Self(WGPUTextureDimension_1D as u32);
    pub(crate) const e2D: Self = Self(WGPUTextureDimension_2D as u32);
    pub(crate) const e3D: Self = Self(WGPUTextureDimension_3D as u32);
}

impl From<WGPUTextureDimension> for TextureDimension {
    fn from(value: WGPUTextureDimension) -> Self { Self(value as u32) }
}
impl From<TextureDimension> for WGPUTextureDimension {
    fn from(value: TextureDimension) -> Self { value.0 as WGPUTextureDimension }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct TextureFormat(pub(crate) u32);
impl TextureFormat {
    pub(crate) const Undefined: Self = Self(WGPUTextureFormat_Undefined as u32);
    pub(crate) const R8Unorm: Self = Self(WGPUTextureFormat_R8Unorm as u32);
    pub(crate) const R8Snorm: Self = Self(WGPUTextureFormat_R8Snorm as u32);
    pub(crate) const R8Uint: Self = Self(WGPUTextureFormat_R8Uint as u32);
    pub(crate) const R8Sint: Self = Self(WGPUTextureFormat_R8Sint as u32);
    pub(crate) const R16Unorm: Self = Self(WGPUTextureFormat_R16Unorm as u32);
    pub(crate) const R16Snorm: Self = Self(WGPUTextureFormat_R16Snorm as u32);
    pub(crate) const R16Uint: Self = Self(WGPUTextureFormat_R16Uint as u32);
    pub(crate) const R16Sint: Self = Self(WGPUTextureFormat_R16Sint as u32);
    pub(crate) const R16Float: Self = Self(WGPUTextureFormat_R16Float as u32);
    pub(crate) const RG8Unorm: Self = Self(WGPUTextureFormat_RG8Unorm as u32);
    pub(crate) const RG8Snorm: Self = Self(WGPUTextureFormat_RG8Snorm as u32);
    pub(crate) const RG8Uint: Self = Self(WGPUTextureFormat_RG8Uint as u32);
    pub(crate) const RG8Sint: Self = Self(WGPUTextureFormat_RG8Sint as u32);
    pub(crate) const R32Float: Self = Self(WGPUTextureFormat_R32Float as u32);
    pub(crate) const R32Uint: Self = Self(WGPUTextureFormat_R32Uint as u32);
    pub(crate) const R32Sint: Self = Self(WGPUTextureFormat_R32Sint as u32);
    pub(crate) const RG16Unorm: Self = Self(WGPUTextureFormat_RG16Unorm as u32);
    pub(crate) const RG16Snorm: Self = Self(WGPUTextureFormat_RG16Snorm as u32);
    pub(crate) const RG16Uint: Self = Self(WGPUTextureFormat_RG16Uint as u32);
    pub(crate) const RG16Sint: Self = Self(WGPUTextureFormat_RG16Sint as u32);
    pub(crate) const RG16Float: Self = Self(WGPUTextureFormat_RG16Float as u32);
    pub(crate) const RGBA8Unorm: Self = Self(WGPUTextureFormat_RGBA8Unorm as u32);
    pub(crate) const RGBA8UnormSrgb: Self = Self(WGPUTextureFormat_RGBA8UnormSrgb as u32);
    pub(crate) const RGBA8Snorm: Self = Self(WGPUTextureFormat_RGBA8Snorm as u32);
    pub(crate) const RGBA8Uint: Self = Self(WGPUTextureFormat_RGBA8Uint as u32);
    pub(crate) const RGBA8Sint: Self = Self(WGPUTextureFormat_RGBA8Sint as u32);
    pub(crate) const BGRA8Unorm: Self = Self(WGPUTextureFormat_BGRA8Unorm as u32);
    pub(crate) const BGRA8UnormSrgb: Self = Self(WGPUTextureFormat_BGRA8UnormSrgb as u32);
    pub(crate) const RGB10A2Uint: Self = Self(WGPUTextureFormat_RGB10A2Uint as u32);
    pub(crate) const RGB10A2Unorm: Self = Self(WGPUTextureFormat_RGB10A2Unorm as u32);
    pub(crate) const RG11B10Ufloat: Self = Self(WGPUTextureFormat_RG11B10Ufloat as u32);
    pub(crate) const RGB9E5Ufloat: Self = Self(WGPUTextureFormat_RGB9E5Ufloat as u32);
    pub(crate) const RG32Float: Self = Self(WGPUTextureFormat_RG32Float as u32);
    pub(crate) const RG32Uint: Self = Self(WGPUTextureFormat_RG32Uint as u32);
    pub(crate) const RG32Sint: Self = Self(WGPUTextureFormat_RG32Sint as u32);
    pub(crate) const RGBA16Unorm: Self = Self(WGPUTextureFormat_RGBA16Unorm as u32);
    pub(crate) const RGBA16Snorm: Self = Self(WGPUTextureFormat_RGBA16Snorm as u32);
    pub(crate) const RGBA16Uint: Self = Self(WGPUTextureFormat_RGBA16Uint as u32);
    pub(crate) const RGBA16Sint: Self = Self(WGPUTextureFormat_RGBA16Sint as u32);
    pub(crate) const RGBA16Float: Self = Self(WGPUTextureFormat_RGBA16Float as u32);
    pub(crate) const RGBA32Float: Self = Self(WGPUTextureFormat_RGBA32Float as u32);
    pub(crate) const RGBA32Uint: Self = Self(WGPUTextureFormat_RGBA32Uint as u32);
    pub(crate) const RGBA32Sint: Self = Self(WGPUTextureFormat_RGBA32Sint as u32);
    pub(crate) const Stencil8: Self = Self(WGPUTextureFormat_Stencil8 as u32);
    pub(crate) const Depth16Unorm: Self = Self(WGPUTextureFormat_Depth16Unorm as u32);
    pub(crate) const Depth24Plus: Self = Self(WGPUTextureFormat_Depth24Plus as u32);
    pub(crate) const Depth24PlusStencil8: Self = Self(WGPUTextureFormat_Depth24PlusStencil8 as u32);
    pub(crate) const Depth32Float: Self = Self(WGPUTextureFormat_Depth32Float as u32);
    pub(crate) const Depth32FloatStencil8: Self = Self(WGPUTextureFormat_Depth32FloatStencil8 as u32);
    pub(crate) const BC1RGBAUnorm: Self = Self(WGPUTextureFormat_BC1RGBAUnorm as u32);
    pub(crate) const BC1RGBAUnormSrgb: Self = Self(WGPUTextureFormat_BC1RGBAUnormSrgb as u32);
    pub(crate) const BC2RGBAUnorm: Self = Self(WGPUTextureFormat_BC2RGBAUnorm as u32);
    pub(crate) const BC2RGBAUnormSrgb: Self = Self(WGPUTextureFormat_BC2RGBAUnormSrgb as u32);
    pub(crate) const BC3RGBAUnorm: Self = Self(WGPUTextureFormat_BC3RGBAUnorm as u32);
    pub(crate) const BC3RGBAUnormSrgb: Self = Self(WGPUTextureFormat_BC3RGBAUnormSrgb as u32);
    pub(crate) const BC4RUnorm: Self = Self(WGPUTextureFormat_BC4RUnorm as u32);
    pub(crate) const BC4RSnorm: Self = Self(WGPUTextureFormat_BC4RSnorm as u32);
    pub(crate) const BC5RGUnorm: Self = Self(WGPUTextureFormat_BC5RGUnorm as u32);
    pub(crate) const BC5RGSnorm: Self = Self(WGPUTextureFormat_BC5RGSnorm as u32);
    pub(crate) const BC6HRGBUfloat: Self = Self(WGPUTextureFormat_BC6HRGBUfloat as u32);
    pub(crate) const BC6HRGBFloat: Self = Self(WGPUTextureFormat_BC6HRGBFloat as u32);
    pub(crate) const BC7RGBAUnorm: Self = Self(WGPUTextureFormat_BC7RGBAUnorm as u32);
    pub(crate) const BC7RGBAUnormSrgb: Self = Self(WGPUTextureFormat_BC7RGBAUnormSrgb as u32);
    pub(crate) const ETC2RGB8Unorm: Self = Self(WGPUTextureFormat_ETC2RGB8Unorm as u32);
    pub(crate) const ETC2RGB8UnormSrgb: Self = Self(WGPUTextureFormat_ETC2RGB8UnormSrgb as u32);
    pub(crate) const ETC2RGB8A1Unorm: Self = Self(WGPUTextureFormat_ETC2RGB8A1Unorm as u32);
    pub(crate) const ETC2RGB8A1UnormSrgb: Self = Self(WGPUTextureFormat_ETC2RGB8A1UnormSrgb as u32);
    pub(crate) const ETC2RGBA8Unorm: Self = Self(WGPUTextureFormat_ETC2RGBA8Unorm as u32);
    pub(crate) const ETC2RGBA8UnormSrgb: Self = Self(WGPUTextureFormat_ETC2RGBA8UnormSrgb as u32);
    pub(crate) const EACR11Unorm: Self = Self(WGPUTextureFormat_EACR11Unorm as u32);
    pub(crate) const EACR11Snorm: Self = Self(WGPUTextureFormat_EACR11Snorm as u32);
    pub(crate) const EACRG11Unorm: Self = Self(WGPUTextureFormat_EACRG11Unorm as u32);
    pub(crate) const EACRG11Snorm: Self = Self(WGPUTextureFormat_EACRG11Snorm as u32);
    pub(crate) const ASTC4x4Unorm: Self = Self(WGPUTextureFormat_ASTC4x4Unorm as u32);
    pub(crate) const ASTC4x4UnormSrgb: Self = Self(WGPUTextureFormat_ASTC4x4UnormSrgb as u32);
    pub(crate) const ASTC5x4Unorm: Self = Self(WGPUTextureFormat_ASTC5x4Unorm as u32);
    pub(crate) const ASTC5x4UnormSrgb: Self = Self(WGPUTextureFormat_ASTC5x4UnormSrgb as u32);
    pub(crate) const ASTC5x5Unorm: Self = Self(WGPUTextureFormat_ASTC5x5Unorm as u32);
    pub(crate) const ASTC5x5UnormSrgb: Self = Self(WGPUTextureFormat_ASTC5x5UnormSrgb as u32);
    pub(crate) const ASTC6x5Unorm: Self = Self(WGPUTextureFormat_ASTC6x5Unorm as u32);
    pub(crate) const ASTC6x5UnormSrgb: Self = Self(WGPUTextureFormat_ASTC6x5UnormSrgb as u32);
    pub(crate) const ASTC6x6Unorm: Self = Self(WGPUTextureFormat_ASTC6x6Unorm as u32);
    pub(crate) const ASTC6x6UnormSrgb: Self = Self(WGPUTextureFormat_ASTC6x6UnormSrgb as u32);
    pub(crate) const ASTC8x5Unorm: Self = Self(WGPUTextureFormat_ASTC8x5Unorm as u32);
    pub(crate) const ASTC8x5UnormSrgb: Self = Self(WGPUTextureFormat_ASTC8x5UnormSrgb as u32);
    pub(crate) const ASTC8x6Unorm: Self = Self(WGPUTextureFormat_ASTC8x6Unorm as u32);
    pub(crate) const ASTC8x6UnormSrgb: Self = Self(WGPUTextureFormat_ASTC8x6UnormSrgb as u32);
    pub(crate) const ASTC8x8Unorm: Self = Self(WGPUTextureFormat_ASTC8x8Unorm as u32);
    pub(crate) const ASTC8x8UnormSrgb: Self = Self(WGPUTextureFormat_ASTC8x8UnormSrgb as u32);
    pub(crate) const ASTC10x5Unorm: Self = Self(WGPUTextureFormat_ASTC10x5Unorm as u32);
    pub(crate) const ASTC10x5UnormSrgb: Self = Self(WGPUTextureFormat_ASTC10x5UnormSrgb as u32);
    pub(crate) const ASTC10x6Unorm: Self = Self(WGPUTextureFormat_ASTC10x6Unorm as u32);
    pub(crate) const ASTC10x6UnormSrgb: Self = Self(WGPUTextureFormat_ASTC10x6UnormSrgb as u32);
    pub(crate) const ASTC10x8Unorm: Self = Self(WGPUTextureFormat_ASTC10x8Unorm as u32);
    pub(crate) const ASTC10x8UnormSrgb: Self = Self(WGPUTextureFormat_ASTC10x8UnormSrgb as u32);
    pub(crate) const ASTC10x10Unorm: Self = Self(WGPUTextureFormat_ASTC10x10Unorm as u32);
    pub(crate) const ASTC10x10UnormSrgb: Self = Self(WGPUTextureFormat_ASTC10x10UnormSrgb as u32);
    pub(crate) const ASTC12x10Unorm: Self = Self(WGPUTextureFormat_ASTC12x10Unorm as u32);
    pub(crate) const ASTC12x10UnormSrgb: Self = Self(WGPUTextureFormat_ASTC12x10UnormSrgb as u32);
    pub(crate) const ASTC12x12Unorm: Self = Self(WGPUTextureFormat_ASTC12x12Unorm as u32);
    pub(crate) const ASTC12x12UnormSrgb: Self = Self(WGPUTextureFormat_ASTC12x12UnormSrgb as u32);
}

impl From<WGPUTextureFormat> for TextureFormat {
    fn from(value: WGPUTextureFormat) -> Self { Self(value as u32) }
}
impl From<TextureFormat> for WGPUTextureFormat {
    fn from(value: TextureFormat) -> Self { value.0 as WGPUTextureFormat }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct TextureSampleType(pub(crate) u32);
impl TextureSampleType {
    pub(crate) const BindingNotUsed: Self = Self(WGPUTextureSampleType_BindingNotUsed as u32);
    pub(crate) const Undefined: Self = Self(WGPUTextureSampleType_Undefined as u32);
    pub(crate) const Float: Self = Self(WGPUTextureSampleType_Float as u32);
    pub(crate) const UnfilterableFloat: Self = Self(WGPUTextureSampleType_UnfilterableFloat as u32);
    pub(crate) const Depth: Self = Self(WGPUTextureSampleType_Depth as u32);
    pub(crate) const Sint: Self = Self(WGPUTextureSampleType_Sint as u32);
    pub(crate) const Uint: Self = Self(WGPUTextureSampleType_Uint as u32);
}

impl From<WGPUTextureSampleType> for TextureSampleType {
    fn from(value: WGPUTextureSampleType) -> Self { Self(value as u32) }
}
impl From<TextureSampleType> for WGPUTextureSampleType {
    fn from(value: TextureSampleType) -> Self { value.0 as WGPUTextureSampleType }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct TextureViewDimension(pub(crate) u32);
impl TextureViewDimension {
    pub(crate) const Undefined: Self = Self(WGPUTextureViewDimension_Undefined as u32);
    pub(crate) const e1D: Self = Self(WGPUTextureViewDimension_1D as u32);
    pub(crate) const e2D: Self = Self(WGPUTextureViewDimension_2D as u32);
    pub(crate) const e2DArray: Self = Self(WGPUTextureViewDimension_2DArray as u32);
    pub(crate) const Cube: Self = Self(WGPUTextureViewDimension_Cube as u32);
    pub(crate) const CubeArray: Self = Self(WGPUTextureViewDimension_CubeArray as u32);
    pub(crate) const e3D: Self = Self(WGPUTextureViewDimension_3D as u32);
}

impl From<WGPUTextureViewDimension> for TextureViewDimension {
    fn from(value: WGPUTextureViewDimension) -> Self { Self(value as u32) }
}
impl From<TextureViewDimension> for WGPUTextureViewDimension {
    fn from(value: TextureViewDimension) -> Self { value.0 as WGPUTextureViewDimension }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ToneMappingMode(pub(crate) u32);
impl ToneMappingMode {
    pub(crate) const Standard: Self = Self(WGPUToneMappingMode_Standard as u32);
    pub(crate) const Extended: Self = Self(WGPUToneMappingMode_Extended as u32);
}

impl From<WGPUToneMappingMode> for ToneMappingMode {
    fn from(value: WGPUToneMappingMode) -> Self { Self(value as u32) }
}
impl From<ToneMappingMode> for WGPUToneMappingMode {
    fn from(value: ToneMappingMode) -> Self { value.0 as WGPUToneMappingMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct VertexFormat(pub(crate) u32);
impl VertexFormat {
    pub(crate) const Uint8: Self = Self(WGPUVertexFormat_Uint8 as u32);
    pub(crate) const Uint8x2: Self = Self(WGPUVertexFormat_Uint8x2 as u32);
    pub(crate) const Uint8x4: Self = Self(WGPUVertexFormat_Uint8x4 as u32);
    pub(crate) const Sint8: Self = Self(WGPUVertexFormat_Sint8 as u32);
    pub(crate) const Sint8x2: Self = Self(WGPUVertexFormat_Sint8x2 as u32);
    pub(crate) const Sint8x4: Self = Self(WGPUVertexFormat_Sint8x4 as u32);
    pub(crate) const Unorm8: Self = Self(WGPUVertexFormat_Unorm8 as u32);
    pub(crate) const Unorm8x2: Self = Self(WGPUVertexFormat_Unorm8x2 as u32);
    pub(crate) const Unorm8x4: Self = Self(WGPUVertexFormat_Unorm8x4 as u32);
    pub(crate) const Snorm8: Self = Self(WGPUVertexFormat_Snorm8 as u32);
    pub(crate) const Snorm8x2: Self = Self(WGPUVertexFormat_Snorm8x2 as u32);
    pub(crate) const Snorm8x4: Self = Self(WGPUVertexFormat_Snorm8x4 as u32);
    pub(crate) const Uint16: Self = Self(WGPUVertexFormat_Uint16 as u32);
    pub(crate) const Uint16x2: Self = Self(WGPUVertexFormat_Uint16x2 as u32);
    pub(crate) const Uint16x4: Self = Self(WGPUVertexFormat_Uint16x4 as u32);
    pub(crate) const Sint16: Self = Self(WGPUVertexFormat_Sint16 as u32);
    pub(crate) const Sint16x2: Self = Self(WGPUVertexFormat_Sint16x2 as u32);
    pub(crate) const Sint16x4: Self = Self(WGPUVertexFormat_Sint16x4 as u32);
    pub(crate) const Unorm16: Self = Self(WGPUVertexFormat_Unorm16 as u32);
    pub(crate) const Unorm16x2: Self = Self(WGPUVertexFormat_Unorm16x2 as u32);
    pub(crate) const Unorm16x4: Self = Self(WGPUVertexFormat_Unorm16x4 as u32);
    pub(crate) const Snorm16: Self = Self(WGPUVertexFormat_Snorm16 as u32);
    pub(crate) const Snorm16x2: Self = Self(WGPUVertexFormat_Snorm16x2 as u32);
    pub(crate) const Snorm16x4: Self = Self(WGPUVertexFormat_Snorm16x4 as u32);
    pub(crate) const Float16: Self = Self(WGPUVertexFormat_Float16 as u32);
    pub(crate) const Float16x2: Self = Self(WGPUVertexFormat_Float16x2 as u32);
    pub(crate) const Float16x4: Self = Self(WGPUVertexFormat_Float16x4 as u32);
    pub(crate) const Float32: Self = Self(WGPUVertexFormat_Float32 as u32);
    pub(crate) const Float32x2: Self = Self(WGPUVertexFormat_Float32x2 as u32);
    pub(crate) const Float32x3: Self = Self(WGPUVertexFormat_Float32x3 as u32);
    pub(crate) const Float32x4: Self = Self(WGPUVertexFormat_Float32x4 as u32);
    pub(crate) const Uint32: Self = Self(WGPUVertexFormat_Uint32 as u32);
    pub(crate) const Uint32x2: Self = Self(WGPUVertexFormat_Uint32x2 as u32);
    pub(crate) const Uint32x3: Self = Self(WGPUVertexFormat_Uint32x3 as u32);
    pub(crate) const Uint32x4: Self = Self(WGPUVertexFormat_Uint32x4 as u32);
    pub(crate) const Sint32: Self = Self(WGPUVertexFormat_Sint32 as u32);
    pub(crate) const Sint32x2: Self = Self(WGPUVertexFormat_Sint32x2 as u32);
    pub(crate) const Sint32x3: Self = Self(WGPUVertexFormat_Sint32x3 as u32);
    pub(crate) const Sint32x4: Self = Self(WGPUVertexFormat_Sint32x4 as u32);
    pub(crate) const Unorm10_10_10_2: Self = Self(WGPUVertexFormat_Unorm10_10_10_2 as u32);
    pub(crate) const Unorm8x4BGRA: Self = Self(WGPUVertexFormat_Unorm8x4BGRA as u32);
}

impl From<WGPUVertexFormat> for VertexFormat {
    fn from(value: WGPUVertexFormat) -> Self { Self(value as u32) }
}
impl From<VertexFormat> for WGPUVertexFormat {
    fn from(value: VertexFormat) -> Self { value.0 as WGPUVertexFormat }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct VertexStepMode(pub(crate) u32);
impl VertexStepMode {
    pub(crate) const Undefined: Self = Self(WGPUVertexStepMode_Undefined as u32);
    pub(crate) const Vertex: Self = Self(WGPUVertexStepMode_Vertex as u32);
    pub(crate) const Instance: Self = Self(WGPUVertexStepMode_Instance as u32);
}

impl From<WGPUVertexStepMode> for VertexStepMode {
    fn from(value: WGPUVertexStepMode) -> Self { Self(value as u32) }
}
impl From<VertexStepMode> for WGPUVertexStepMode {
    fn from(value: VertexStepMode) -> Self { value.0 as WGPUVertexStepMode }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct WaitStatus(pub(crate) u32);
impl WaitStatus {
    pub(crate) const Success: Self = Self(WGPUWaitStatus_Success as u32);
    pub(crate) const TimedOut: Self = Self(WGPUWaitStatus_TimedOut as u32);
    pub(crate) const Error: Self = Self(WGPUWaitStatus_Error as u32);
}

impl From<WGPUWaitStatus> for WaitStatus {
    fn from(value: WGPUWaitStatus) -> Self { Self(value as u32) }
}
impl From<WaitStatus> for WGPUWaitStatus {
    fn from(value: WaitStatus) -> Self { value.0 as WGPUWaitStatus }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct WGSLLanguageFeatureName(pub(crate) u32);
impl WGSLLanguageFeatureName {
    pub(crate) const ReadonlyAndReadwriteStorageTextures: Self = Self(WGPUWGSLLanguageFeatureName_ReadonlyAndReadwriteStorageTextures as u32);
    pub(crate) const Packed4x8IntegerDotProduct: Self = Self(WGPUWGSLLanguageFeatureName_Packed4x8IntegerDotProduct as u32);
    pub(crate) const UnrestrictedPointerParameters: Self = Self(WGPUWGSLLanguageFeatureName_UnrestrictedPointerParameters as u32);
    pub(crate) const PointerCompositeAccess: Self = Self(WGPUWGSLLanguageFeatureName_PointerCompositeAccess as u32);
}

impl From<WGPUWGSLLanguageFeatureName> for WGSLLanguageFeatureName {
    fn from(value: WGPUWGSLLanguageFeatureName) -> Self { Self(value as u32) }
}
impl From<WGSLLanguageFeatureName> for WGPUWGSLLanguageFeatureName {
    fn from(value: WGSLLanguageFeatureName) -> Self { value.0 as WGPUWGSLLanguageFeatureName }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct BufferUsage(pub(crate) u64);
impl BufferUsage {
    pub(crate) const None: Self = Self(WGPUBufferUsage_None as u64);
    pub(crate) const MapRead: Self = Self(WGPUBufferUsage_MapRead as u64);
    pub(crate) const MapWrite: Self = Self(WGPUBufferUsage_MapWrite as u64);
    pub(crate) const CopySrc: Self = Self(WGPUBufferUsage_CopySrc as u64);
    pub(crate) const CopyDst: Self = Self(WGPUBufferUsage_CopyDst as u64);
    pub(crate) const Index: Self = Self(WGPUBufferUsage_Index as u64);
    pub(crate) const Vertex: Self = Self(WGPUBufferUsage_Vertex as u64);
    pub(crate) const Uniform: Self = Self(WGPUBufferUsage_Uniform as u64);
    pub(crate) const Storage: Self = Self(WGPUBufferUsage_Storage as u64);
    pub(crate) const Indirect: Self = Self(WGPUBufferUsage_Indirect as u64);
    pub(crate) const QueryResolve: Self = Self(WGPUBufferUsage_QueryResolve as u64);
}

impl From<WGPUBufferUsage> for BufferUsage {
    fn from(value: WGPUBufferUsage) -> Self { Self(value as u64) }
}
impl From<BufferUsage> for WGPUBufferUsage {
    fn from(value: BufferUsage) -> Self { value.0 as WGPUBufferUsage }
}

impl IsWGPUBitmask for BufferUsage {
    type Integral = u64;
    fn fromIntegral(value: Self::Integral) -> Self { Self(value) }
    fn intoIntegral(self) -> Self::Integral { self.0 }
    fn wrappingSubOne(value: Self::Integral) -> Self::Integral { value.wrapping_sub(1) }
}
impl_wgpu_bitmask_operators!(BufferUsage);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ColorWriteMask(pub(crate) u64);
impl ColorWriteMask {
    pub(crate) const None: Self = Self(WGPUColorWriteMask_None as u64);
    pub(crate) const Red: Self = Self(WGPUColorWriteMask_Red as u64);
    pub(crate) const Green: Self = Self(WGPUColorWriteMask_Green as u64);
    pub(crate) const Blue: Self = Self(WGPUColorWriteMask_Blue as u64);
    pub(crate) const Alpha: Self = Self(WGPUColorWriteMask_Alpha as u64);
    pub(crate) const All: Self = Self(WGPUColorWriteMask_All as u64);
}

impl From<WGPUColorWriteMask> for ColorWriteMask {
    fn from(value: WGPUColorWriteMask) -> Self { Self(value as u64) }
}
impl From<ColorWriteMask> for WGPUColorWriteMask {
    fn from(value: ColorWriteMask) -> Self { value.0 as WGPUColorWriteMask }
}

impl IsWGPUBitmask for ColorWriteMask {
    type Integral = u64;
    fn fromIntegral(value: Self::Integral) -> Self { Self(value) }
    fn intoIntegral(self) -> Self::Integral { self.0 }
    fn wrappingSubOne(value: Self::Integral) -> Self::Integral { value.wrapping_sub(1) }
}
impl_wgpu_bitmask_operators!(ColorWriteMask);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct MapMode(pub(crate) u64);
impl MapMode {
    pub(crate) const None: Self = Self(WGPUMapMode_None as u64);
    pub(crate) const Read: Self = Self(WGPUMapMode_Read as u64);
    pub(crate) const Write: Self = Self(WGPUMapMode_Write as u64);
}

impl From<WGPUMapMode> for MapMode {
    fn from(value: WGPUMapMode) -> Self { Self(value as u64) }
}
impl From<MapMode> for WGPUMapMode {
    fn from(value: MapMode) -> Self { value.0 as WGPUMapMode }
}

impl IsWGPUBitmask for MapMode {
    type Integral = u64;
    fn fromIntegral(value: Self::Integral) -> Self { Self(value) }
    fn intoIntegral(self) -> Self::Integral { self.0 }
    fn wrappingSubOne(value: Self::Integral) -> Self::Integral { value.wrapping_sub(1) }
}
impl_wgpu_bitmask_operators!(MapMode);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct ShaderStage(pub(crate) u64);
impl ShaderStage {
    pub(crate) const None: Self = Self(WGPUShaderStage_None as u64);
    pub(crate) const Vertex: Self = Self(WGPUShaderStage_Vertex as u64);
    pub(crate) const Fragment: Self = Self(WGPUShaderStage_Fragment as u64);
    pub(crate) const Compute: Self = Self(WGPUShaderStage_Compute as u64);
}

impl From<WGPUShaderStage> for ShaderStage {
    fn from(value: WGPUShaderStage) -> Self { Self(value as u64) }
}
impl From<ShaderStage> for WGPUShaderStage {
    fn from(value: ShaderStage) -> Self { value.0 as WGPUShaderStage }
}

impl IsWGPUBitmask for ShaderStage {
    type Integral = u64;
    fn fromIntegral(value: Self::Integral) -> Self { Self(value) }
    fn intoIntegral(self) -> Self::Integral { self.0 }
    fn wrappingSubOne(value: Self::Integral) -> Self::Integral { value.wrapping_sub(1) }
}
impl_wgpu_bitmask_operators!(ShaderStage);

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct TextureUsage(pub(crate) u64);
impl TextureUsage {
    pub(crate) const None: Self = Self(WGPUTextureUsage_None as u64);
    pub(crate) const CopySrc: Self = Self(WGPUTextureUsage_CopySrc as u64);
    pub(crate) const CopyDst: Self = Self(WGPUTextureUsage_CopyDst as u64);
    pub(crate) const TextureBinding: Self = Self(WGPUTextureUsage_TextureBinding as u64);
    pub(crate) const StorageBinding: Self = Self(WGPUTextureUsage_StorageBinding as u64);
    pub(crate) const RenderAttachment: Self = Self(WGPUTextureUsage_RenderAttachment as u64);
}

impl From<WGPUTextureUsage> for TextureUsage {
    fn from(value: WGPUTextureUsage) -> Self { Self(value as u64) }
}
impl From<TextureUsage> for WGPUTextureUsage {
    fn from(value: TextureUsage) -> Self { value.0 as WGPUTextureUsage }
}

impl IsWGPUBitmask for TextureUsage {
    type Integral = u64;
    fn fromIntegral(value: Self::Integral) -> Self { Self(value) }
    fn intoIntegral(self) -> Self::Integral { self.0 }
    fn wrappingSubOne(value: Self::Integral) -> Self::Integral { value.wrapping_sub(1) }
}
impl_wgpu_bitmask_operators!(TextureUsage);

impl Default for WGPUStringView {
    fn default() -> Self {
        Self {
            data: std::ptr::null(),
            length: WGPU_STRLEN,
        }
    }
}

impl Default for WGPUBufferMapCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUCompilationInfoCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUCreateComputePipelineAsyncCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUCreateRenderPipelineAsyncCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUDeviceLostCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUPopErrorScopeCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUQueueWorkDoneCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPURequestAdapterCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPURequestDeviceCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            mode: 0 as WGPUCallbackMode,
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUUncapturedErrorCallbackInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            callback: None,
            userdata1: std::ptr::null_mut(),
            userdata2: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUAdapterInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            vendor: WGPUStringView::default(),
            architecture: WGPUStringView::default(),
            device: WGPUStringView::default(),
            description: WGPUStringView::default(),
            backendType: WGPUBackendType_Undefined,
            adapterType: 0 as WGPUAdapterType,
            vendorID: 0,
            deviceID: 0,
            subgroupMinSize: 0,
            subgroupMaxSize: 0,
        }
    }
}

impl Default for WGPUBindGroupEntry {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            binding: 0,
            buffer: std::ptr::null_mut(),
            offset: 0,
            size: WGPU_WHOLE_SIZE,
            sampler: std::ptr::null_mut(),
            textureView: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUBlendComponent {
    fn default() -> Self {
        Self {
            operation: WGPUBlendOperation_Undefined,
            srcFactor: WGPUBlendFactor_Undefined,
            dstFactor: WGPUBlendFactor_Undefined,
        }
    }
}

impl Default for WGPUBufferBindingLayout {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            r#type: WGPUBufferBindingType_Undefined,
            hasDynamicOffset: WGPU_FALSE,
            minBindingSize: 0,
        }
    }
}

impl Default for WGPUBufferDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            usage: WGPUBufferUsage_None,
            size: 0,
            mappedAtCreation: WGPU_FALSE,
        }
    }
}

impl Default for WGPUColor {
    fn default() -> Self {
        Self {
            r: 0.0,
            g: 0.0,
            b: 0.0,
            a: 0.0,
        }
    }
}

impl Default for WGPUCommandBufferDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
        }
    }
}

impl Default for WGPUCommandEncoderDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
        }
    }
}

impl Default for WGPUCompatibilityModeLimits {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_CompatibilityModeLimits },
            maxStorageBuffersInVertexStage: WGPU_LIMIT_U32_UNDEFINED,
            maxStorageTexturesInVertexStage: WGPU_LIMIT_U32_UNDEFINED,
            maxStorageBuffersInFragmentStage: WGPU_LIMIT_U32_UNDEFINED,
            maxStorageTexturesInFragmentStage: WGPU_LIMIT_U32_UNDEFINED,
        }
    }
}

impl Default for WGPUConstantEntry {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            key: WGPUStringView::default(),
            value: 0.0,
        }
    }
}

impl Default for WGPUDawnCompilationMessageUtf16 {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_DawnCompilationMessageUtf16 },
            linePos: 0,
            offset: 0,
            length: 0,
        }
    }
}

impl Default for WGPUEmscriptenSurfaceSourceCanvasHTMLSelector {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_EmscriptenSurfaceSourceCanvasHTMLSelector },
            selector: WGPUStringView::default(),
        }
    }
}

impl Default for WGPUExtent3D {
    fn default() -> Self {
        Self {
            width: 0,
            height: 1,
            depthOrArrayLayers: 1,
        }
    }
}

impl Default for WGPUFuture {
    fn default() -> Self {
        Self {
            id: 0,
        }
    }
}

impl Default for WGPUInstanceLimits {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            timedWaitAnyMaxCount: 0,
        }
    }
}

impl Default for WGPUINTERNAL_HAVE_EMDAWNWEBGPU_HEADER {
    fn default() -> Self {
        Self {
            unused: WGPU_FALSE,
        }
    }
}

impl Default for WGPUMultisampleState {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            count: 1,
            mask: 0xFFFFFFFF,
            alphaToCoverageEnabled: WGPU_FALSE,
        }
    }
}

impl Default for WGPUOrigin3D {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
        }
    }
}

impl Default for WGPUPassTimestampWrites {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            querySet: std::ptr::null_mut(),
            beginningOfPassWriteIndex: WGPU_QUERY_SET_INDEX_UNDEFINED,
            endOfPassWriteIndex: WGPU_QUERY_SET_INDEX_UNDEFINED,
        }
    }
}

impl Default for WGPUPipelineLayoutDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            bindGroupLayoutCount: 0,
            bindGroupLayouts: std::ptr::null(),
            immediateSize: 0,
        }
    }
}

impl Default for WGPUPrimitiveState {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            topology: WGPUPrimitiveTopology_Undefined,
            stripIndexFormat: WGPUIndexFormat_Undefined,
            frontFace: WGPUFrontFace_Undefined,
            cullMode: WGPUCullMode_Undefined,
            unclippedDepth: WGPU_FALSE,
        }
    }
}

impl Default for WGPUQuerySetDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            r#type: 0 as WGPUQueryType,
            count: 0,
        }
    }
}

impl Default for WGPUQueueDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
        }
    }
}

impl Default for WGPURenderBundleDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
        }
    }
}

impl Default for WGPURenderBundleEncoderDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            colorFormatCount: 0,
            colorFormats: std::ptr::null(),
            depthStencilFormat: WGPUTextureFormat_Undefined,
            sampleCount: 1,
            depthReadOnly: WGPU_FALSE,
            stencilReadOnly: WGPU_FALSE,
        }
    }
}

impl Default for WGPURenderPassDepthStencilAttachment {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            view: std::ptr::null_mut(),
            depthLoadOp: WGPULoadOp_Undefined,
            depthStoreOp: WGPUStoreOp_Undefined,
            depthClearValue: f32::NAN,
            depthReadOnly: WGPU_FALSE,
            stencilLoadOp: WGPULoadOp_Undefined,
            stencilStoreOp: WGPUStoreOp_Undefined,
            stencilClearValue: 0,
            stencilReadOnly: WGPU_FALSE,
        }
    }
}

impl Default for WGPURenderPassMaxDrawCount {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_RenderPassMaxDrawCount },
            maxDrawCount: 50000000,
        }
    }
}

impl Default for WGPURequestAdapterWebXROptions {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_RequestAdapterWebXROptions },
            xrCompatible: WGPU_FALSE,
        }
    }
}

impl Default for WGPUSamplerBindingLayout {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            r#type: WGPUSamplerBindingType_Undefined,
        }
    }
}

impl Default for WGPUSamplerDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            addressModeU: WGPUAddressMode_Undefined,
            addressModeV: WGPUAddressMode_Undefined,
            addressModeW: WGPUAddressMode_Undefined,
            magFilter: WGPUFilterMode_Undefined,
            minFilter: WGPUFilterMode_Undefined,
            mipmapFilter: WGPUMipmapFilterMode_Undefined,
            lodMinClamp: 0.0,
            lodMaxClamp: 32.0,
            compare: WGPUCompareFunction_Undefined,
            maxAnisotropy: 1,
        }
    }
}

impl Default for WGPUShaderSourceSPIRV {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_ShaderSourceSPIRV },
            codeSize: 0,
            code: std::ptr::null(),
        }
    }
}

impl Default for WGPUShaderSourceWGSL {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_ShaderSourceWGSL },
            code: WGPUStringView::default(),
        }
    }
}

impl Default for WGPUStencilFaceState {
    fn default() -> Self {
        Self {
            compare: WGPUCompareFunction_Undefined,
            failOp: WGPUStencilOperation_Undefined,
            depthFailOp: WGPUStencilOperation_Undefined,
            passOp: WGPUStencilOperation_Undefined,
        }
    }
}

impl Default for WGPUStorageTextureBindingLayout {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            access: WGPUStorageTextureAccess_Undefined,
            format: WGPUTextureFormat_Undefined,
            viewDimension: WGPUTextureViewDimension_Undefined,
        }
    }
}

impl Default for WGPUSupportedFeatures {
    fn default() -> Self {
        Self {
            featureCount: 0,
            features: std::ptr::null(),
        }
    }
}

impl Default for WGPUSupportedInstanceFeatures {
    fn default() -> Self {
        Self {
            featureCount: 0,
            features: std::ptr::null(),
        }
    }
}

impl Default for WGPUSupportedWGSLLanguageFeatures {
    fn default() -> Self {
        Self {
            featureCount: 0,
            features: std::ptr::null(),
        }
    }
}

impl Default for WGPUSurfaceCapabilities {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            usages: WGPUTextureUsage_None,
            formatCount: 0,
            formats: std::ptr::null(),
            presentModeCount: 0,
            presentModes: std::ptr::null(),
            alphaModeCount: 0,
            alphaModes: std::ptr::null(),
        }
    }
}

impl Default for WGPUSurfaceColorManagement {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_SurfaceColorManagement },
            colorSpace: 0 as WGPUPredefinedColorSpace,
            toneMappingMode: 0 as WGPUToneMappingMode,
        }
    }
}

impl Default for WGPUSurfaceConfiguration {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            device: std::ptr::null_mut(),
            format: WGPUTextureFormat_Undefined,
            usage: WGPUTextureUsage_RenderAttachment,
            width: 0,
            height: 0,
            viewFormatCount: 0,
            viewFormats: std::ptr::null(),
            alphaMode: WGPUCompositeAlphaMode_Auto,
            presentMode: WGPUPresentMode_Undefined,
        }
    }
}

impl Default for WGPUSurfaceTexture {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            texture: std::ptr::null_mut(),
            status: 0 as WGPUSurfaceGetCurrentTextureStatus,
        }
    }
}

impl Default for WGPUTexelCopyBufferLayout {
    fn default() -> Self {
        Self {
            offset: 0,
            bytesPerRow: WGPU_COPY_STRIDE_UNDEFINED,
            rowsPerImage: WGPU_COPY_STRIDE_UNDEFINED,
        }
    }
}

impl Default for WGPUTextureBindingLayout {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            sampleType: WGPUTextureSampleType_Undefined,
            viewDimension: WGPUTextureViewDimension_Undefined,
            multisampled: WGPU_FALSE,
        }
    }
}

impl Default for WGPUTextureBindingViewDimensionDescriptor {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_TextureBindingViewDimensionDescriptor },
            textureBindingViewDimension: WGPUTextureViewDimension_Undefined,
        }
    }
}

impl Default for WGPUTextureComponentSwizzle {
    fn default() -> Self {
        Self {
            r: WGPUComponentSwizzle_Undefined,
            g: WGPUComponentSwizzle_Undefined,
            b: WGPUComponentSwizzle_Undefined,
            a: WGPUComponentSwizzle_Undefined,
        }
    }
}

impl Default for WGPUVertexAttribute {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            format: 0 as WGPUVertexFormat,
            offset: 0,
            shaderLocation: 0,
        }
    }
}

impl Default for WGPUBindGroupDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            layout: std::ptr::null_mut(),
            entryCount: 0,
            entries: std::ptr::null(),
        }
    }
}

impl Default for WGPUBindGroupLayoutEntry {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            binding: 0,
            visibility: WGPUShaderStage_None,
            bindingArraySize: 0,
            buffer: WGPUBufferBindingLayout::default(),
            sampler: WGPUSamplerBindingLayout::default(),
            texture: WGPUTextureBindingLayout::default(),
            storageTexture: WGPUStorageTextureBindingLayout::default(),
        }
    }
}

impl Default for WGPUBlendState {
    fn default() -> Self {
        Self {
            color: WGPUBlendComponent::default(),
            alpha: WGPUBlendComponent::default(),
        }
    }
}

impl Default for WGPUCompilationMessage {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            message: WGPUStringView::default(),
            r#type: 0 as WGPUCompilationMessageType,
            lineNum: 0,
            linePos: 0,
            offset: 0,
            length: 0,
        }
    }
}

impl Default for WGPUComputePassDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            timestampWrites: std::ptr::null(),
        }
    }
}

impl Default for WGPUComputeState {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            module: std::ptr::null_mut(),
            entryPoint: WGPUStringView::default(),
            constantCount: 0,
            constants: std::ptr::null(),
        }
    }
}

impl Default for WGPUDepthStencilState {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            format: WGPUTextureFormat_Undefined,
            depthWriteEnabled: WGPUOptionalBool_Undefined,
            depthCompare: WGPUCompareFunction_Undefined,
            stencilFront: WGPUStencilFaceState::default(),
            stencilBack: WGPUStencilFaceState::default(),
            stencilReadMask: 0xFFFFFFFF,
            stencilWriteMask: 0xFFFFFFFF,
            depthBias: 0,
            depthBiasSlopeScale: 0.0,
            depthBiasClamp: 0.0,
        }
    }
}

impl Default for WGPUFutureWaitInfo {
    fn default() -> Self {
        Self {
            future: WGPUFuture::default(),
            completed: WGPU_FALSE,
        }
    }
}

impl Default for WGPUInstanceDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            requiredFeatureCount: 0,
            requiredFeatures: std::ptr::null(),
            requiredLimits: std::ptr::null(),
        }
    }
}

impl Default for WGPULimits {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            maxTextureDimension1D: WGPU_LIMIT_U32_UNDEFINED,
            maxTextureDimension2D: WGPU_LIMIT_U32_UNDEFINED,
            maxTextureDimension3D: WGPU_LIMIT_U32_UNDEFINED,
            maxTextureArrayLayers: WGPU_LIMIT_U32_UNDEFINED,
            maxBindGroups: WGPU_LIMIT_U32_UNDEFINED,
            maxBindGroupsPlusVertexBuffers: WGPU_LIMIT_U32_UNDEFINED,
            maxBindingsPerBindGroup: WGPU_LIMIT_U32_UNDEFINED,
            maxDynamicUniformBuffersPerPipelineLayout: WGPU_LIMIT_U32_UNDEFINED,
            maxDynamicStorageBuffersPerPipelineLayout: WGPU_LIMIT_U32_UNDEFINED,
            maxSampledTexturesPerShaderStage: WGPU_LIMIT_U32_UNDEFINED,
            maxSamplersPerShaderStage: WGPU_LIMIT_U32_UNDEFINED,
            maxStorageBuffersPerShaderStage: WGPU_LIMIT_U32_UNDEFINED,
            maxStorageTexturesPerShaderStage: WGPU_LIMIT_U32_UNDEFINED,
            maxUniformBuffersPerShaderStage: WGPU_LIMIT_U32_UNDEFINED,
            maxUniformBufferBindingSize: WGPU_LIMIT_U64_UNDEFINED,
            maxStorageBufferBindingSize: WGPU_LIMIT_U64_UNDEFINED,
            minUniformBufferOffsetAlignment: WGPU_LIMIT_U32_UNDEFINED,
            minStorageBufferOffsetAlignment: WGPU_LIMIT_U32_UNDEFINED,
            maxVertexBuffers: WGPU_LIMIT_U32_UNDEFINED,
            maxBufferSize: WGPU_LIMIT_U64_UNDEFINED,
            maxVertexAttributes: WGPU_LIMIT_U32_UNDEFINED,
            maxVertexBufferArrayStride: WGPU_LIMIT_U32_UNDEFINED,
            maxInterStageShaderVariables: WGPU_LIMIT_U32_UNDEFINED,
            maxColorAttachments: WGPU_LIMIT_U32_UNDEFINED,
            maxColorAttachmentBytesPerSample: WGPU_LIMIT_U32_UNDEFINED,
            maxComputeWorkgroupStorageSize: WGPU_LIMIT_U32_UNDEFINED,
            maxComputeInvocationsPerWorkgroup: WGPU_LIMIT_U32_UNDEFINED,
            maxComputeWorkgroupSizeX: WGPU_LIMIT_U32_UNDEFINED,
            maxComputeWorkgroupSizeY: WGPU_LIMIT_U32_UNDEFINED,
            maxComputeWorkgroupSizeZ: WGPU_LIMIT_U32_UNDEFINED,
            maxComputeWorkgroupsPerDimension: WGPU_LIMIT_U32_UNDEFINED,
            maxImmediateSize: WGPU_LIMIT_U32_UNDEFINED,
        }
    }
}

impl Default for WGPURenderPassColorAttachment {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            view: std::ptr::null_mut(),
            depthSlice: WGPU_DEPTH_SLICE_UNDEFINED,
            resolveTarget: std::ptr::null_mut(),
            loadOp: WGPULoadOp_Undefined,
            storeOp: WGPUStoreOp_Undefined,
            clearValue: WGPUColor::default(),
        }
    }
}

impl Default for WGPURequestAdapterOptions {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            featureLevel: WGPUFeatureLevel_Undefined,
            powerPreference: WGPUPowerPreference_Undefined,
            forceFallbackAdapter: WGPU_FALSE,
            backendType: WGPUBackendType_Undefined,
            compatibleSurface: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUShaderModuleDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
        }
    }
}

impl Default for WGPUSurfaceDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
        }
    }
}

impl Default for WGPUTexelCopyBufferInfo {
    fn default() -> Self {
        Self {
            layout: WGPUTexelCopyBufferLayout::default(),
            buffer: std::ptr::null_mut(),
        }
    }
}

impl Default for WGPUTexelCopyTextureInfo {
    fn default() -> Self {
        Self {
            texture: std::ptr::null_mut(),
            mipLevel: 0,
            origin: WGPUOrigin3D::default(),
            aspect: WGPUTextureAspect_Undefined,
        }
    }
}

impl Default for WGPUTextureComponentSwizzleDescriptor {
    fn default() -> Self {
        Self {
            chain: WGPUChainedStruct { next: std::ptr::null_mut(), sType: WGPUSType_TextureComponentSwizzleDescriptor },
            swizzle: WGPUTextureComponentSwizzle::default(),
        }
    }
}

impl Default for WGPUTextureDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            usage: WGPUTextureUsage_None,
            dimension: WGPUTextureDimension_Undefined,
            size: WGPUExtent3D::default(),
            format: WGPUTextureFormat_Undefined,
            mipLevelCount: 1,
            sampleCount: 1,
            viewFormatCount: 0,
            viewFormats: std::ptr::null(),
        }
    }
}

impl Default for WGPUVertexBufferLayout {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            stepMode: WGPUVertexStepMode_Undefined,
            arrayStride: 0,
            attributeCount: 0,
            attributes: std::ptr::null(),
        }
    }
}

impl Default for WGPUBindGroupLayoutDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            entryCount: 0,
            entries: std::ptr::null(),
        }
    }
}

impl Default for WGPUColorTargetState {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            format: WGPUTextureFormat_Undefined,
            blend: std::ptr::null(),
            writeMask: WGPUColorWriteMask_All,
        }
    }
}

impl Default for WGPUCompilationInfo {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            messageCount: 0,
            messages: std::ptr::null(),
        }
    }
}

impl Default for WGPUComputePipelineDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            layout: std::ptr::null_mut(),
            compute: WGPUComputeState::default(),
        }
    }
}

impl Default for WGPUDeviceDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            requiredFeatureCount: 0,
            requiredFeatures: std::ptr::null(),
            requiredLimits: std::ptr::null(),
            defaultQueue: WGPUQueueDescriptor::default(),
            deviceLostCallbackInfo: WGPUDeviceLostCallbackInfo::default(),
            uncapturedErrorCallbackInfo: WGPUUncapturedErrorCallbackInfo::default(),
        }
    }
}

impl Default for WGPURenderPassDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            colorAttachmentCount: 0,
            colorAttachments: std::ptr::null(),
            depthStencilAttachment: std::ptr::null(),
            occlusionQuerySet: std::ptr::null_mut(),
            timestampWrites: std::ptr::null(),
        }
    }
}

impl Default for WGPUTextureViewDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            format: WGPUTextureFormat_Undefined,
            dimension: WGPUTextureViewDimension_Undefined,
            baseMipLevel: 0,
            mipLevelCount: WGPU_MIP_LEVEL_COUNT_UNDEFINED,
            baseArrayLayer: 0,
            arrayLayerCount: WGPU_ARRAY_LAYER_COUNT_UNDEFINED,
            aspect: WGPUTextureAspect_Undefined,
            usage: WGPUTextureUsage_None,
        }
    }
}

impl Default for WGPUVertexState {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            module: std::ptr::null_mut(),
            entryPoint: WGPUStringView::default(),
            constantCount: 0,
            constants: std::ptr::null(),
            bufferCount: 0,
            buffers: std::ptr::null(),
        }
    }
}

impl Default for WGPUFragmentState {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            module: std::ptr::null_mut(),
            entryPoint: WGPUStringView::default(),
            constantCount: 0,
            constants: std::ptr::null(),
            targetCount: 0,
            targets: std::ptr::null(),
        }
    }
}

impl Default for WGPURenderPipelineDescriptor {
    fn default() -> Self {
        Self {
            nextInChain: std::ptr::null_mut(),
            label: WGPUStringView::default(),
            layout: std::ptr::null_mut(),
            vertex: WGPUVertexState::default(),
            primitive: WGPUPrimitiveState::default(),
            depthStencil: std::ptr::null(),
            multisample: WGPUMultisampleState::default(),
            fragment: std::ptr::null(),
        }
    }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct Bool(pub(crate) WGPUBool);
impl Bool { pub(crate) const fn asBool(self) -> bool { self.0 != 0 } }
impl From<bool> for Bool { fn from(value: bool) -> Self { Self(value as WGPUBool) } }
impl From<WGPUBool> for Bool { fn from(value: WGPUBool) -> Self { Self(value) } }

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(crate) struct OptionalBool(pub(crate) WGPUOptionalBool);
impl OptionalBool {
    pub(crate) const False: Self = Self(WGPUOptionalBool_False);
    pub(crate) const True: Self = Self(WGPUOptionalBool_True);
    pub(crate) const Undefined: Self = Self(WGPUOptionalBool_Undefined);
    pub(crate) fn intoOption(self) -> Option<bool> {
        (self.0 != WGPUOptionalBool_Undefined).then_some(self.0 != WGPUOptionalBool_False)
    }
}
impl Default for OptionalBool { fn default() -> Self { Self::Undefined } }
impl From<bool> for OptionalBool { fn from(value: bool) -> Self { Self(value as WGPUOptionalBool) } }
impl From<Option<bool>> for OptionalBool {
    fn from(value: Option<bool>) -> Self { value.map(Self::from).unwrap_or(Self::Undefined) }
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ConvertibleStatus(pub(crate) Status);
impl ConvertibleStatus { pub(crate) fn asBool(self) -> bool { self.0 == Status::Success } }

pub(crate) type Proc = WGPUProc;
pub(crate) type StringView = WGPUStringView;

pub(crate) type BindGroupEntry = WGPUBindGroupEntry;
pub(crate) type BlendComponent = WGPUBlendComponent;
pub(crate) type BufferBindingLayout = WGPUBufferBindingLayout;
pub(crate) type BufferDescriptor = WGPUBufferDescriptor;
pub(crate) type Color = WGPUColor;
pub(crate) type CommandBufferDescriptor = WGPUCommandBufferDescriptor;
pub(crate) type CommandEncoderDescriptor = WGPUCommandEncoderDescriptor;
pub(crate) type CompatibilityModeLimits = WGPUCompatibilityModeLimits;
pub(crate) type ConstantEntry = WGPUConstantEntry;
pub(crate) type DawnCompilationMessageUtf16 = WGPUDawnCompilationMessageUtf16;
pub(crate) type EmscriptenSurfaceSourceCanvasHTMLSelector = WGPUEmscriptenSurfaceSourceCanvasHTMLSelector;
pub(crate) type Extent3D = WGPUExtent3D;
pub(crate) type Future = WGPUFuture;
pub(crate) type InstanceLimits = WGPUInstanceLimits;
pub(crate) type INTERNAL_HAVE_EMDAWNWEBGPU_HEADER = WGPUINTERNAL_HAVE_EMDAWNWEBGPU_HEADER;
pub(crate) type MultisampleState = WGPUMultisampleState;
pub(crate) type Origin3D = WGPUOrigin3D;
pub(crate) type PassTimestampWrites = WGPUPassTimestampWrites;
pub(crate) type PipelineLayoutDescriptor = WGPUPipelineLayoutDescriptor;
pub(crate) type PrimitiveState = WGPUPrimitiveState;
pub(crate) type QuerySetDescriptor = WGPUQuerySetDescriptor;
pub(crate) type QueueDescriptor = WGPUQueueDescriptor;
pub(crate) type RenderBundleDescriptor = WGPURenderBundleDescriptor;
pub(crate) type RenderBundleEncoderDescriptor = WGPURenderBundleEncoderDescriptor;
pub(crate) type RenderPassDepthStencilAttachment = WGPURenderPassDepthStencilAttachment;
pub(crate) type RenderPassMaxDrawCount = WGPURenderPassMaxDrawCount;
pub(crate) type RequestAdapterWebXROptions = WGPURequestAdapterWebXROptions;
pub(crate) type SamplerBindingLayout = WGPUSamplerBindingLayout;
pub(crate) type SamplerDescriptor = WGPUSamplerDescriptor;
pub(crate) type ShaderSourceSPIRV = WGPUShaderSourceSPIRV;
pub(crate) type ShaderSourceWGSL = WGPUShaderSourceWGSL;
pub(crate) type StencilFaceState = WGPUStencilFaceState;
pub(crate) type StorageTextureBindingLayout = WGPUStorageTextureBindingLayout;
pub(crate) type SurfaceColorManagement = WGPUSurfaceColorManagement;
pub(crate) type SurfaceConfiguration = WGPUSurfaceConfiguration;
pub(crate) type SurfaceTexture = WGPUSurfaceTexture;
pub(crate) type TexelCopyBufferLayout = WGPUTexelCopyBufferLayout;
pub(crate) type TextureBindingLayout = WGPUTextureBindingLayout;
pub(crate) type TextureBindingViewDimensionDescriptor = WGPUTextureBindingViewDimensionDescriptor;
pub(crate) type TextureComponentSwizzle = WGPUTextureComponentSwizzle;
pub(crate) type VertexAttribute = WGPUVertexAttribute;
pub(crate) type BindGroupDescriptor = WGPUBindGroupDescriptor;
pub(crate) type BindGroupLayoutEntry = WGPUBindGroupLayoutEntry;
pub(crate) type BlendState = WGPUBlendState;
pub(crate) type CompilationMessage = WGPUCompilationMessage;
pub(crate) type ComputePassDescriptor = WGPUComputePassDescriptor;
pub(crate) type ComputeState = WGPUComputeState;
pub(crate) type DepthStencilState = WGPUDepthStencilState;
pub(crate) type FutureWaitInfo = WGPUFutureWaitInfo;
pub(crate) type InstanceDescriptor = WGPUInstanceDescriptor;
pub(crate) type Limits = WGPULimits;
pub(crate) type RenderPassColorAttachment = WGPURenderPassColorAttachment;
pub(crate) type RequestAdapterOptions = WGPURequestAdapterOptions;
pub(crate) type ShaderModuleDescriptor = WGPUShaderModuleDescriptor;
pub(crate) type SurfaceDescriptor = WGPUSurfaceDescriptor;
pub(crate) type TexelCopyBufferInfo = WGPUTexelCopyBufferInfo;
pub(crate) type TexelCopyTextureInfo = WGPUTexelCopyTextureInfo;
pub(crate) type TextureComponentSwizzleDescriptor = WGPUTextureComponentSwizzleDescriptor;
pub(crate) type TextureDescriptor = WGPUTextureDescriptor;
pub(crate) type VertexBufferLayout = WGPUVertexBufferLayout;
pub(crate) type BindGroupLayoutDescriptor = WGPUBindGroupLayoutDescriptor;
pub(crate) type ColorTargetState = WGPUColorTargetState;
pub(crate) type CompilationInfo = WGPUCompilationInfo;
pub(crate) type ComputePipelineDescriptor = WGPUComputePipelineDescriptor;
pub(crate) type RenderPassDescriptor = WGPURenderPassDescriptor;
pub(crate) type TextureViewDescriptor = WGPUTextureViewDescriptor;
pub(crate) type VertexState = WGPUVertexState;
pub(crate) type FragmentState = WGPUFragmentState;
pub(crate) type RenderPipelineDescriptor = WGPURenderPipelineDescriptor;
pub(crate) type DeviceDescriptor = WGPUDeviceDescriptor;

#[repr(transparent)]
pub(crate) struct AdapterInfo(WGPUAdapterInfo);
impl AdapterInfo {
    pub(crate) fn asRaw(&self) -> &WGPUAdapterInfo { &self.0 }
    fn resetForOutput(&mut self) -> *mut WGPUAdapterInfo {
        self.freeMembers();
        self.0 = WGPUAdapterInfo::default();
        &mut self.0
    }
    fn freeMembers(&mut self) {
        if !self.0.vendor.data.is_null() || !self.0.architecture.data.is_null() || !self.0.device.data.is_null() || !self.0.description.data.is_null() { unsafe { wgpuAdapterInfoFreeMembers(std::ptr::read(&self.0)); } }
    }
}
impl Default for AdapterInfo {
    fn default() -> Self { Self(WGPUAdapterInfo::default()) }
}
impl Drop for AdapterInfo {
    fn drop(&mut self) { self.freeMembers(); }
}

#[repr(transparent)]
pub(crate) struct SupportedFeatures(WGPUSupportedFeatures);
impl SupportedFeatures {
    pub(crate) fn asRaw(&self) -> &WGPUSupportedFeatures { &self.0 }
    fn resetForOutput(&mut self) -> *mut WGPUSupportedFeatures {
        self.freeMembers();
        self.0 = WGPUSupportedFeatures::default();
        &mut self.0
    }
    fn freeMembers(&mut self) {
        if !self.0.features.is_null() { unsafe { wgpuSupportedFeaturesFreeMembers(std::ptr::read(&self.0)); } }
    }
}
impl Default for SupportedFeatures {
    fn default() -> Self { Self(WGPUSupportedFeatures::default()) }
}
impl Drop for SupportedFeatures {
    fn drop(&mut self) { self.freeMembers(); }
}

#[repr(transparent)]
pub(crate) struct SupportedInstanceFeatures(WGPUSupportedInstanceFeatures);
impl SupportedInstanceFeatures {
    pub(crate) fn asRaw(&self) -> &WGPUSupportedInstanceFeatures { &self.0 }
    fn resetForOutput(&mut self) -> *mut WGPUSupportedInstanceFeatures {
        self.freeMembers();
        self.0 = WGPUSupportedInstanceFeatures::default();
        &mut self.0
    }
    fn freeMembers(&mut self) {
        if !self.0.features.is_null() { unsafe { wgpuSupportedInstanceFeaturesFreeMembers(std::ptr::read(&self.0)); } }
    }
}
impl Default for SupportedInstanceFeatures {
    fn default() -> Self { Self(WGPUSupportedInstanceFeatures::default()) }
}
impl Drop for SupportedInstanceFeatures {
    fn drop(&mut self) { self.freeMembers(); }
}

#[repr(transparent)]
pub(crate) struct SupportedWGSLLanguageFeatures(WGPUSupportedWGSLLanguageFeatures);
impl SupportedWGSLLanguageFeatures {
    pub(crate) fn asRaw(&self) -> &WGPUSupportedWGSLLanguageFeatures { &self.0 }
    fn resetForOutput(&mut self) -> *mut WGPUSupportedWGSLLanguageFeatures {
        self.freeMembers();
        self.0 = WGPUSupportedWGSLLanguageFeatures::default();
        &mut self.0
    }
    fn freeMembers(&mut self) {
        if !self.0.features.is_null() { unsafe { wgpuSupportedWGSLLanguageFeaturesFreeMembers(std::ptr::read(&self.0)); } }
    }
}
impl Default for SupportedWGSLLanguageFeatures {
    fn default() -> Self { Self(WGPUSupportedWGSLLanguageFeatures::default()) }
}
impl Drop for SupportedWGSLLanguageFeatures {
    fn drop(&mut self) { self.freeMembers(); }
}

#[repr(transparent)]
pub(crate) struct SurfaceCapabilities(WGPUSurfaceCapabilities);
impl SurfaceCapabilities {
    pub(crate) fn asRaw(&self) -> &WGPUSurfaceCapabilities { &self.0 }
    fn resetForOutput(&mut self) -> *mut WGPUSurfaceCapabilities {
        self.freeMembers();
        self.0 = WGPUSurfaceCapabilities::default();
        &mut self.0
    }
    fn freeMembers(&mut self) {
        if !self.0.formats.is_null() || !self.0.presentModes.is_null() || !self.0.alphaModes.is_null() { unsafe { wgpuSurfaceCapabilitiesFreeMembers(std::ptr::read(&self.0)); } }
    }
}
impl Default for SurfaceCapabilities {
    fn default() -> Self { Self(WGPUSurfaceCapabilities::default()) }
}
impl Drop for SurfaceCapabilities {
    fn drop(&mut self) { self.freeMembers(); }
}

#[repr(transparent)]
pub(crate) struct Adapter { handle: WGPUAdapter }
impl Adapter {
    pub(crate) const fn Get(&self) -> WGPUAdapter { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUAdapter) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUAdapter) -> Self {
        if !handle.is_null() { wgpuAdapterAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUAdapter {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn GetFeatures(&self, output: &mut SupportedFeatures) {
        wgpuAdapterGetFeatures(self.handle, output.resetForOutput())
    }
    pub(crate) unsafe fn GetInfo(&self, output: &mut AdapterInfo) -> ConvertibleStatus {
        ConvertibleStatus(Status::from(wgpuAdapterGetInfo(self.handle, output.resetForOutput())))
    }
    pub(crate) unsafe fn GetLimits(&self, arg1: *mut WGPULimits) -> ConvertibleStatus {
        ConvertibleStatus(Status::from(wgpuAdapterGetLimits(self.handle, arg1)))
    }
    pub(crate) unsafe fn HasFeature(&self, arg1: WGPUFeatureName) -> Bool {
        Bool::from(wgpuAdapterHasFeature(self.handle, arg1))
    }
    pub(crate) unsafe fn RequestDevice(&self, arg1: *const WGPUDeviceDescriptor, arg2: WGPURequestDeviceCallbackInfo) -> WGPUFuture {
        wgpuAdapterRequestDevice(self.handle, arg1, arg2)
    }
}

impl Default for Adapter {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for Adapter {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuAdapterAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for Adapter {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuAdapterRelease(self.handle); } }
    }
}
impl PartialEq for Adapter {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for Adapter {}

#[repr(transparent)]
pub(crate) struct BindGroup { handle: WGPUBindGroup }
impl BindGroup {
    pub(crate) const fn Get(&self) -> WGPUBindGroup { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUBindGroup) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUBindGroup) -> Self {
        if !handle.is_null() { wgpuBindGroupAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUBindGroup {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuBindGroupSetLabel(self.handle, arg1)
    }
}

impl Default for BindGroup {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for BindGroup {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuBindGroupAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for BindGroup {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuBindGroupRelease(self.handle); } }
    }
}
impl PartialEq for BindGroup {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for BindGroup {}

#[repr(transparent)]
pub(crate) struct BindGroupLayout { handle: WGPUBindGroupLayout }
impl BindGroupLayout {
    pub(crate) const fn Get(&self) -> WGPUBindGroupLayout { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUBindGroupLayout) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUBindGroupLayout) -> Self {
        if !handle.is_null() { wgpuBindGroupLayoutAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUBindGroupLayout {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuBindGroupLayoutSetLabel(self.handle, arg1)
    }
}

impl Default for BindGroupLayout {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for BindGroupLayout {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuBindGroupLayoutAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for BindGroupLayout {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuBindGroupLayoutRelease(self.handle); } }
    }
}
impl PartialEq for BindGroupLayout {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for BindGroupLayout {}

#[repr(transparent)]
pub(crate) struct Buffer { handle: WGPUBuffer }
impl Buffer {
    pub(crate) const fn Get(&self) -> WGPUBuffer { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUBuffer) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUBuffer) -> Self {
        if !handle.is_null() { wgpuBufferAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUBuffer {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn Destroy(&self) {
        wgpuBufferDestroy(self.handle)
    }
    pub(crate) unsafe fn GetConstMappedRange(&self, arg1: usize, arg2: usize) -> *const std::ffi::c_void {
        wgpuBufferGetConstMappedRange(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn GetMappedRange(&self, arg1: usize, arg2: usize) -> *mut std::ffi::c_void {
        wgpuBufferGetMappedRange(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn GetMapState(&self) -> BufferMapState {
        BufferMapState::from(wgpuBufferGetMapState(self.handle))
    }
    pub(crate) unsafe fn GetSize(&self) -> u64 {
        wgpuBufferGetSize(self.handle)
    }
    pub(crate) unsafe fn GetUsage(&self) -> BufferUsage {
        BufferUsage::from(wgpuBufferGetUsage(self.handle))
    }
    pub(crate) unsafe fn MapAsync(&self, arg1: WGPUMapMode, arg2: usize, arg3: usize, arg4: WGPUBufferMapCallbackInfo) -> WGPUFuture {
        wgpuBufferMapAsync(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn ReadMappedRange(&self, arg1: usize, arg2: *mut std::ffi::c_void, arg3: usize) -> ConvertibleStatus {
        ConvertibleStatus(Status::from(wgpuBufferReadMappedRange(self.handle, arg1, arg2, arg3)))
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuBufferSetLabel(self.handle, arg1)
    }
    pub(crate) unsafe fn Unmap(&self) {
        wgpuBufferUnmap(self.handle)
    }
    pub(crate) unsafe fn WriteMappedRange(&self, arg1: usize, arg2: *const std::ffi::c_void, arg3: usize) -> ConvertibleStatus {
        ConvertibleStatus(Status::from(wgpuBufferWriteMappedRange(self.handle, arg1, arg2, arg3)))
    }
}

impl Default for Buffer {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for Buffer {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuBufferAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuBufferRelease(self.handle); } }
    }
}
impl PartialEq for Buffer {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for Buffer {}

#[repr(transparent)]
pub(crate) struct CommandBuffer { handle: WGPUCommandBuffer }
impl CommandBuffer {
    pub(crate) const fn Get(&self) -> WGPUCommandBuffer { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUCommandBuffer) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUCommandBuffer) -> Self {
        if !handle.is_null() { wgpuCommandBufferAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUCommandBuffer {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuCommandBufferSetLabel(self.handle, arg1)
    }
}

impl Default for CommandBuffer {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for CommandBuffer {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuCommandBufferAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for CommandBuffer {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuCommandBufferRelease(self.handle); } }
    }
}
impl PartialEq for CommandBuffer {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for CommandBuffer {}

#[repr(transparent)]
pub(crate) struct CommandEncoder { handle: WGPUCommandEncoder }
impl CommandEncoder {
    pub(crate) const fn Get(&self) -> WGPUCommandEncoder { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUCommandEncoder) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUCommandEncoder) -> Self {
        if !handle.is_null() { wgpuCommandEncoderAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUCommandEncoder {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn BeginComputePass(&self, arg1: *const WGPUComputePassDescriptor) -> ComputePassEncoder {
        ComputePassEncoder::Acquire(wgpuCommandEncoderBeginComputePass(self.handle, arg1))
    }
    pub(crate) unsafe fn BeginRenderPass(&self, arg1: *const WGPURenderPassDescriptor) -> RenderPassEncoder {
        RenderPassEncoder::Acquire(wgpuCommandEncoderBeginRenderPass(self.handle, arg1))
    }
    pub(crate) unsafe fn ClearBuffer(&self, arg1: WGPUBuffer, arg2: u64, arg3: u64) {
        wgpuCommandEncoderClearBuffer(self.handle, arg1, arg2, arg3)
    }
    pub(crate) unsafe fn CopyBufferToBuffer(&self, arg1: WGPUBuffer, arg2: u64, arg3: WGPUBuffer, arg4: u64, arg5: u64) {
        wgpuCommandEncoderCopyBufferToBuffer(self.handle, arg1, arg2, arg3, arg4, arg5)
    }
    pub(crate) unsafe fn CopyBufferToTexture(&self, arg1: *const WGPUTexelCopyBufferInfo, arg2: *const WGPUTexelCopyTextureInfo, arg3: *const WGPUExtent3D) {
        wgpuCommandEncoderCopyBufferToTexture(self.handle, arg1, arg2, arg3)
    }
    pub(crate) unsafe fn CopyTextureToBuffer(&self, arg1: *const WGPUTexelCopyTextureInfo, arg2: *const WGPUTexelCopyBufferInfo, arg3: *const WGPUExtent3D) {
        wgpuCommandEncoderCopyTextureToBuffer(self.handle, arg1, arg2, arg3)
    }
    pub(crate) unsafe fn CopyTextureToTexture(&self, arg1: *const WGPUTexelCopyTextureInfo, arg2: *const WGPUTexelCopyTextureInfo, arg3: *const WGPUExtent3D) {
        wgpuCommandEncoderCopyTextureToTexture(self.handle, arg1, arg2, arg3)
    }
    pub(crate) unsafe fn Finish(&self, arg1: *const WGPUCommandBufferDescriptor) -> CommandBuffer {
        CommandBuffer::Acquire(wgpuCommandEncoderFinish(self.handle, arg1))
    }
    pub(crate) unsafe fn InsertDebugMarker(&self, arg1: WGPUStringView) {
        wgpuCommandEncoderInsertDebugMarker(self.handle, arg1)
    }
    pub(crate) unsafe fn PopDebugGroup(&self) {
        wgpuCommandEncoderPopDebugGroup(self.handle)
    }
    pub(crate) unsafe fn PushDebugGroup(&self, arg1: WGPUStringView) {
        wgpuCommandEncoderPushDebugGroup(self.handle, arg1)
    }
    pub(crate) unsafe fn ResolveQuerySet(&self, arg1: WGPUQuerySet, arg2: u32, arg3: u32, arg4: WGPUBuffer, arg5: u64) {
        wgpuCommandEncoderResolveQuerySet(self.handle, arg1, arg2, arg3, arg4, arg5)
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuCommandEncoderSetLabel(self.handle, arg1)
    }
    pub(crate) unsafe fn WriteTimestamp(&self, arg1: WGPUQuerySet, arg2: u32) {
        wgpuCommandEncoderWriteTimestamp(self.handle, arg1, arg2)
    }
}

impl Default for CommandEncoder {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for CommandEncoder {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuCommandEncoderAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for CommandEncoder {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuCommandEncoderRelease(self.handle); } }
    }
}
impl PartialEq for CommandEncoder {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for CommandEncoder {}

#[repr(transparent)]
pub(crate) struct ComputePassEncoder { handle: WGPUComputePassEncoder }
impl ComputePassEncoder {
    pub(crate) const fn Get(&self) -> WGPUComputePassEncoder { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUComputePassEncoder) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUComputePassEncoder) -> Self {
        if !handle.is_null() { wgpuComputePassEncoderAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUComputePassEncoder {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn DispatchWorkgroups(&self, arg1: u32, arg2: u32, arg3: u32) {
        wgpuComputePassEncoderDispatchWorkgroups(self.handle, arg1, arg2, arg3)
    }
    pub(crate) unsafe fn DispatchWorkgroupsIndirect(&self, arg1: WGPUBuffer, arg2: u64) {
        wgpuComputePassEncoderDispatchWorkgroupsIndirect(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn End(&self) {
        wgpuComputePassEncoderEnd(self.handle)
    }
    pub(crate) unsafe fn InsertDebugMarker(&self, arg1: WGPUStringView) {
        wgpuComputePassEncoderInsertDebugMarker(self.handle, arg1)
    }
    pub(crate) unsafe fn PopDebugGroup(&self) {
        wgpuComputePassEncoderPopDebugGroup(self.handle)
    }
    pub(crate) unsafe fn PushDebugGroup(&self, arg1: WGPUStringView) {
        wgpuComputePassEncoderPushDebugGroup(self.handle, arg1)
    }
    pub(crate) unsafe fn SetBindGroup(&self, arg1: u32, arg2: WGPUBindGroup, arg3: usize, arg4: *const u32) {
        wgpuComputePassEncoderSetBindGroup(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuComputePassEncoderSetLabel(self.handle, arg1)
    }
    pub(crate) unsafe fn SetPipeline(&self, arg1: WGPUComputePipeline) {
        wgpuComputePassEncoderSetPipeline(self.handle, arg1)
    }
    pub(crate) unsafe fn WriteTimestamp(&self, arg1: WGPUQuerySet, arg2: u32) {
        wgpuComputePassEncoderWriteTimestamp(self.handle, arg1, arg2)
    }
}

impl Default for ComputePassEncoder {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for ComputePassEncoder {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuComputePassEncoderAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for ComputePassEncoder {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuComputePassEncoderRelease(self.handle); } }
    }
}
impl PartialEq for ComputePassEncoder {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for ComputePassEncoder {}

#[repr(transparent)]
pub(crate) struct ComputePipeline { handle: WGPUComputePipeline }
impl ComputePipeline {
    pub(crate) const fn Get(&self) -> WGPUComputePipeline { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUComputePipeline) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUComputePipeline) -> Self {
        if !handle.is_null() { wgpuComputePipelineAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUComputePipeline {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn GetBindGroupLayout(&self, arg1: u32) -> BindGroupLayout {
        BindGroupLayout::Acquire(wgpuComputePipelineGetBindGroupLayout(self.handle, arg1))
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuComputePipelineSetLabel(self.handle, arg1)
    }
}

impl Default for ComputePipeline {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for ComputePipeline {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuComputePipelineAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for ComputePipeline {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuComputePipelineRelease(self.handle); } }
    }
}
impl PartialEq for ComputePipeline {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for ComputePipeline {}

#[repr(transparent)]
pub(crate) struct Device { handle: WGPUDevice }
impl Device {
    pub(crate) const fn Get(&self) -> WGPUDevice { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUDevice) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUDevice) -> Self {
        if !handle.is_null() { wgpuDeviceAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUDevice {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn CreateBindGroup(&self, arg1: *const WGPUBindGroupDescriptor) -> BindGroup {
        BindGroup::Acquire(wgpuDeviceCreateBindGroup(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateBindGroupLayout(&self, arg1: *const WGPUBindGroupLayoutDescriptor) -> BindGroupLayout {
        BindGroupLayout::Acquire(wgpuDeviceCreateBindGroupLayout(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateBuffer(&self, arg1: *const WGPUBufferDescriptor) -> Buffer {
        Buffer::Acquire(wgpuDeviceCreateBuffer(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateCommandEncoder(&self, arg1: *const WGPUCommandEncoderDescriptor) -> CommandEncoder {
        CommandEncoder::Acquire(wgpuDeviceCreateCommandEncoder(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateComputePipeline(&self, arg1: *const WGPUComputePipelineDescriptor) -> ComputePipeline {
        ComputePipeline::Acquire(wgpuDeviceCreateComputePipeline(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateComputePipelineAsync(&self, arg1: *const WGPUComputePipelineDescriptor, arg2: WGPUCreateComputePipelineAsyncCallbackInfo) -> WGPUFuture {
        wgpuDeviceCreateComputePipelineAsync(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn CreatePipelineLayout(&self, arg1: *const WGPUPipelineLayoutDescriptor) -> PipelineLayout {
        PipelineLayout::Acquire(wgpuDeviceCreatePipelineLayout(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateQuerySet(&self, arg1: *const WGPUQuerySetDescriptor) -> QuerySet {
        QuerySet::Acquire(wgpuDeviceCreateQuerySet(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateRenderBundleEncoder(&self, arg1: *const WGPURenderBundleEncoderDescriptor) -> RenderBundleEncoder {
        RenderBundleEncoder::Acquire(wgpuDeviceCreateRenderBundleEncoder(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateRenderPipeline(&self, arg1: *const WGPURenderPipelineDescriptor) -> RenderPipeline {
        RenderPipeline::Acquire(wgpuDeviceCreateRenderPipeline(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateRenderPipelineAsync(&self, arg1: *const WGPURenderPipelineDescriptor, arg2: WGPUCreateRenderPipelineAsyncCallbackInfo) -> WGPUFuture {
        wgpuDeviceCreateRenderPipelineAsync(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn CreateSampler(&self, arg1: *const WGPUSamplerDescriptor) -> Sampler {
        Sampler::Acquire(wgpuDeviceCreateSampler(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateShaderModule(&self, arg1: *const WGPUShaderModuleDescriptor) -> ShaderModule {
        ShaderModule::Acquire(wgpuDeviceCreateShaderModule(self.handle, arg1))
    }
    pub(crate) unsafe fn CreateTexture(&self, arg1: *const WGPUTextureDescriptor) -> Texture {
        Texture::Acquire(wgpuDeviceCreateTexture(self.handle, arg1))
    }
    pub(crate) unsafe fn Destroy(&self) {
        wgpuDeviceDestroy(self.handle)
    }
    pub(crate) unsafe fn GetAdapterInfo(&self, output: &mut AdapterInfo) -> ConvertibleStatus {
        ConvertibleStatus(Status::from(wgpuDeviceGetAdapterInfo(self.handle, output.resetForOutput())))
    }
    pub(crate) unsafe fn GetFeatures(&self, output: &mut SupportedFeatures) {
        wgpuDeviceGetFeatures(self.handle, output.resetForOutput())
    }
    pub(crate) unsafe fn GetLimits(&self, arg1: *mut WGPULimits) -> ConvertibleStatus {
        ConvertibleStatus(Status::from(wgpuDeviceGetLimits(self.handle, arg1)))
    }
    pub(crate) unsafe fn GetLostFuture(&self) -> WGPUFuture {
        wgpuDeviceGetLostFuture(self.handle)
    }
    pub(crate) unsafe fn GetQueue(&self) -> Queue {
        Queue::Acquire(wgpuDeviceGetQueue(self.handle))
    }
    pub(crate) unsafe fn HasFeature(&self, arg1: WGPUFeatureName) -> Bool {
        Bool::from(wgpuDeviceHasFeature(self.handle, arg1))
    }
    pub(crate) unsafe fn PopErrorScope(&self, arg1: WGPUPopErrorScopeCallbackInfo) -> WGPUFuture {
        wgpuDevicePopErrorScope(self.handle, arg1)
    }
    pub(crate) unsafe fn PushErrorScope(&self, arg1: WGPUErrorFilter) {
        wgpuDevicePushErrorScope(self.handle, arg1)
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuDeviceSetLabel(self.handle, arg1)
    }
}

impl Default for Device {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for Device {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuDeviceAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for Device {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuDeviceRelease(self.handle); } }
    }
}
impl PartialEq for Device {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for Device {}

#[repr(transparent)]
pub(crate) struct Instance { handle: WGPUInstance }
impl Instance {
    pub(crate) const fn Get(&self) -> WGPUInstance { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUInstance) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUInstance) -> Self {
        if !handle.is_null() { wgpuInstanceAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUInstance {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn CreateSurface(&self, arg1: *const WGPUSurfaceDescriptor) -> Surface {
        Surface::Acquire(wgpuInstanceCreateSurface(self.handle, arg1))
    }
    pub(crate) unsafe fn GetWGSLLanguageFeatures(&self, output: &mut SupportedWGSLLanguageFeatures) {
        wgpuInstanceGetWGSLLanguageFeatures(self.handle, output.resetForOutput())
    }
    pub(crate) unsafe fn HasWGSLLanguageFeature(&self, arg1: WGPUWGSLLanguageFeatureName) -> Bool {
        Bool::from(wgpuInstanceHasWGSLLanguageFeature(self.handle, arg1))
    }
    pub(crate) unsafe fn ProcessEvents(&self) {
        wgpuInstanceProcessEvents(self.handle)
    }
    pub(crate) unsafe fn RequestAdapter(&self, arg1: *const WGPURequestAdapterOptions, arg2: WGPURequestAdapterCallbackInfo) -> WGPUFuture {
        wgpuInstanceRequestAdapter(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn WaitAny(&self, arg1: usize, arg2: *mut WGPUFutureWaitInfo, arg3: u64) -> WaitStatus {
        WaitStatus::from(wgpuInstanceWaitAny(self.handle, arg1, arg2, arg3))
    }
}

impl Default for Instance {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for Instance {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuInstanceAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for Instance {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuInstanceRelease(self.handle); } }
    }
}
impl PartialEq for Instance {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for Instance {}

#[repr(transparent)]
pub(crate) struct PipelineLayout { handle: WGPUPipelineLayout }
impl PipelineLayout {
    pub(crate) const fn Get(&self) -> WGPUPipelineLayout { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUPipelineLayout) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUPipelineLayout) -> Self {
        if !handle.is_null() { wgpuPipelineLayoutAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUPipelineLayout {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuPipelineLayoutSetLabel(self.handle, arg1)
    }
}

impl Default for PipelineLayout {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for PipelineLayout {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuPipelineLayoutAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for PipelineLayout {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuPipelineLayoutRelease(self.handle); } }
    }
}
impl PartialEq for PipelineLayout {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for PipelineLayout {}

#[repr(transparent)]
pub(crate) struct QuerySet { handle: WGPUQuerySet }
impl QuerySet {
    pub(crate) const fn Get(&self) -> WGPUQuerySet { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUQuerySet) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUQuerySet) -> Self {
        if !handle.is_null() { wgpuQuerySetAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUQuerySet {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn Destroy(&self) {
        wgpuQuerySetDestroy(self.handle)
    }
    pub(crate) unsafe fn GetCount(&self) -> u32 {
        wgpuQuerySetGetCount(self.handle)
    }
    pub(crate) unsafe fn GetType(&self) -> QueryType {
        QueryType::from(wgpuQuerySetGetType(self.handle))
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuQuerySetSetLabel(self.handle, arg1)
    }
}

impl Default for QuerySet {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for QuerySet {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuQuerySetAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for QuerySet {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuQuerySetRelease(self.handle); } }
    }
}
impl PartialEq for QuerySet {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for QuerySet {}

#[repr(transparent)]
pub(crate) struct Queue { handle: WGPUQueue }
impl Queue {
    pub(crate) const fn Get(&self) -> WGPUQueue { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUQueue) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUQueue) -> Self {
        if !handle.is_null() { wgpuQueueAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUQueue {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn OnSubmittedWorkDone(&self, arg1: WGPUQueueWorkDoneCallbackInfo) -> WGPUFuture {
        wgpuQueueOnSubmittedWorkDone(self.handle, arg1)
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuQueueSetLabel(self.handle, arg1)
    }
    pub(crate) unsafe fn Submit(&self, arg1: usize, arg2: *const WGPUCommandBuffer) {
        wgpuQueueSubmit(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn WriteBuffer(&self, arg1: WGPUBuffer, arg2: u64, arg3: *const std::ffi::c_void, arg4: usize) {
        wgpuQueueWriteBuffer(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn WriteTexture(&self, arg1: *const WGPUTexelCopyTextureInfo, arg2: *const std::ffi::c_void, arg3: usize, arg4: *const WGPUTexelCopyBufferLayout, arg5: *const WGPUExtent3D) {
        wgpuQueueWriteTexture(self.handle, arg1, arg2, arg3, arg4, arg5)
    }
}

impl Default for Queue {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for Queue {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuQueueAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for Queue {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuQueueRelease(self.handle); } }
    }
}
impl PartialEq for Queue {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for Queue {}

#[repr(transparent)]
pub(crate) struct RenderBundle { handle: WGPURenderBundle }
impl RenderBundle {
    pub(crate) const fn Get(&self) -> WGPURenderBundle { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPURenderBundle) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPURenderBundle) -> Self {
        if !handle.is_null() { wgpuRenderBundleAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPURenderBundle {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuRenderBundleSetLabel(self.handle, arg1)
    }
}

impl Default for RenderBundle {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for RenderBundle {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuRenderBundleAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for RenderBundle {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuRenderBundleRelease(self.handle); } }
    }
}
impl PartialEq for RenderBundle {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for RenderBundle {}

#[repr(transparent)]
pub(crate) struct RenderBundleEncoder { handle: WGPURenderBundleEncoder }
impl RenderBundleEncoder {
    pub(crate) const fn Get(&self) -> WGPURenderBundleEncoder { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPURenderBundleEncoder) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPURenderBundleEncoder) -> Self {
        if !handle.is_null() { wgpuRenderBundleEncoderAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPURenderBundleEncoder {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn Draw(&self, arg1: u32, arg2: u32, arg3: u32, arg4: u32) {
        wgpuRenderBundleEncoderDraw(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn DrawIndexed(&self, arg1: u32, arg2: u32, arg3: u32, arg4: i32, arg5: u32) {
        wgpuRenderBundleEncoderDrawIndexed(self.handle, arg1, arg2, arg3, arg4, arg5)
    }
    pub(crate) unsafe fn DrawIndexedIndirect(&self, arg1: WGPUBuffer, arg2: u64) {
        wgpuRenderBundleEncoderDrawIndexedIndirect(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn DrawIndirect(&self, arg1: WGPUBuffer, arg2: u64) {
        wgpuRenderBundleEncoderDrawIndirect(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn Finish(&self, arg1: *const WGPURenderBundleDescriptor) -> RenderBundle {
        RenderBundle::Acquire(wgpuRenderBundleEncoderFinish(self.handle, arg1))
    }
    pub(crate) unsafe fn InsertDebugMarker(&self, arg1: WGPUStringView) {
        wgpuRenderBundleEncoderInsertDebugMarker(self.handle, arg1)
    }
    pub(crate) unsafe fn PopDebugGroup(&self) {
        wgpuRenderBundleEncoderPopDebugGroup(self.handle)
    }
    pub(crate) unsafe fn PushDebugGroup(&self, arg1: WGPUStringView) {
        wgpuRenderBundleEncoderPushDebugGroup(self.handle, arg1)
    }
    pub(crate) unsafe fn SetBindGroup(&self, arg1: u32, arg2: WGPUBindGroup, arg3: usize, arg4: *const u32) {
        wgpuRenderBundleEncoderSetBindGroup(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn SetIndexBuffer(&self, arg1: WGPUBuffer, arg2: WGPUIndexFormat, arg3: u64, arg4: u64) {
        wgpuRenderBundleEncoderSetIndexBuffer(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuRenderBundleEncoderSetLabel(self.handle, arg1)
    }
    pub(crate) unsafe fn SetPipeline(&self, arg1: WGPURenderPipeline) {
        wgpuRenderBundleEncoderSetPipeline(self.handle, arg1)
    }
    pub(crate) unsafe fn SetVertexBuffer(&self, arg1: u32, arg2: WGPUBuffer, arg3: u64, arg4: u64) {
        wgpuRenderBundleEncoderSetVertexBuffer(self.handle, arg1, arg2, arg3, arg4)
    }
}

impl Default for RenderBundleEncoder {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for RenderBundleEncoder {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuRenderBundleEncoderAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for RenderBundleEncoder {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuRenderBundleEncoderRelease(self.handle); } }
    }
}
impl PartialEq for RenderBundleEncoder {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for RenderBundleEncoder {}

#[repr(transparent)]
pub(crate) struct RenderPassEncoder { handle: WGPURenderPassEncoder }
impl RenderPassEncoder {
    pub(crate) const fn Get(&self) -> WGPURenderPassEncoder { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPURenderPassEncoder) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPURenderPassEncoder) -> Self {
        if !handle.is_null() { wgpuRenderPassEncoderAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPURenderPassEncoder {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn BeginOcclusionQuery(&self, arg1: u32) {
        wgpuRenderPassEncoderBeginOcclusionQuery(self.handle, arg1)
    }
    pub(crate) unsafe fn Draw(&self, arg1: u32, arg2: u32, arg3: u32, arg4: u32) {
        wgpuRenderPassEncoderDraw(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn DrawIndexed(&self, arg1: u32, arg2: u32, arg3: u32, arg4: i32, arg5: u32) {
        wgpuRenderPassEncoderDrawIndexed(self.handle, arg1, arg2, arg3, arg4, arg5)
    }
    pub(crate) unsafe fn DrawIndexedIndirect(&self, arg1: WGPUBuffer, arg2: u64) {
        wgpuRenderPassEncoderDrawIndexedIndirect(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn DrawIndirect(&self, arg1: WGPUBuffer, arg2: u64) {
        wgpuRenderPassEncoderDrawIndirect(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn End(&self) {
        wgpuRenderPassEncoderEnd(self.handle)
    }
    pub(crate) unsafe fn EndOcclusionQuery(&self) {
        wgpuRenderPassEncoderEndOcclusionQuery(self.handle)
    }
    pub(crate) unsafe fn ExecuteBundles(&self, arg1: usize, arg2: *const WGPURenderBundle) {
        wgpuRenderPassEncoderExecuteBundles(self.handle, arg1, arg2)
    }
    pub(crate) unsafe fn InsertDebugMarker(&self, arg1: WGPUStringView) {
        wgpuRenderPassEncoderInsertDebugMarker(self.handle, arg1)
    }
    pub(crate) unsafe fn MultiDrawIndexedIndirect(&self, arg1: WGPUBuffer, arg2: u64, arg3: u32, arg4: WGPUBuffer, arg5: u64) {
        wgpuRenderPassEncoderMultiDrawIndexedIndirect(self.handle, arg1, arg2, arg3, arg4, arg5)
    }
    pub(crate) unsafe fn MultiDrawIndirect(&self, arg1: WGPUBuffer, arg2: u64, arg3: u32, arg4: WGPUBuffer, arg5: u64) {
        wgpuRenderPassEncoderMultiDrawIndirect(self.handle, arg1, arg2, arg3, arg4, arg5)
    }
    pub(crate) unsafe fn PopDebugGroup(&self) {
        wgpuRenderPassEncoderPopDebugGroup(self.handle)
    }
    pub(crate) unsafe fn PushDebugGroup(&self, arg1: WGPUStringView) {
        wgpuRenderPassEncoderPushDebugGroup(self.handle, arg1)
    }
    pub(crate) unsafe fn SetBindGroup(&self, arg1: u32, arg2: WGPUBindGroup, arg3: usize, arg4: *const u32) {
        wgpuRenderPassEncoderSetBindGroup(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn SetBlendConstant(&self, arg1: *const WGPUColor) {
        wgpuRenderPassEncoderSetBlendConstant(self.handle, arg1)
    }
    pub(crate) unsafe fn SetIndexBuffer(&self, arg1: WGPUBuffer, arg2: WGPUIndexFormat, arg3: u64, arg4: u64) {
        wgpuRenderPassEncoderSetIndexBuffer(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuRenderPassEncoderSetLabel(self.handle, arg1)
    }
    pub(crate) unsafe fn SetPipeline(&self, arg1: WGPURenderPipeline) {
        wgpuRenderPassEncoderSetPipeline(self.handle, arg1)
    }
    pub(crate) unsafe fn SetScissorRect(&self, arg1: u32, arg2: u32, arg3: u32, arg4: u32) {
        wgpuRenderPassEncoderSetScissorRect(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn SetStencilReference(&self, arg1: u32) {
        wgpuRenderPassEncoderSetStencilReference(self.handle, arg1)
    }
    pub(crate) unsafe fn SetVertexBuffer(&self, arg1: u32, arg2: WGPUBuffer, arg3: u64, arg4: u64) {
        wgpuRenderPassEncoderSetVertexBuffer(self.handle, arg1, arg2, arg3, arg4)
    }
    pub(crate) unsafe fn SetViewport(&self, arg1: f32, arg2: f32, arg3: f32, arg4: f32, arg5: f32, arg6: f32) {
        wgpuRenderPassEncoderSetViewport(self.handle, arg1, arg2, arg3, arg4, arg5, arg6)
    }
    pub(crate) unsafe fn WriteTimestamp(&self, arg1: WGPUQuerySet, arg2: u32) {
        wgpuRenderPassEncoderWriteTimestamp(self.handle, arg1, arg2)
    }
}

impl Default for RenderPassEncoder {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for RenderPassEncoder {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuRenderPassEncoderAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for RenderPassEncoder {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuRenderPassEncoderRelease(self.handle); } }
    }
}
impl PartialEq for RenderPassEncoder {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for RenderPassEncoder {}

#[repr(transparent)]
pub(crate) struct RenderPipeline { handle: WGPURenderPipeline }
impl RenderPipeline {
    pub(crate) const fn Get(&self) -> WGPURenderPipeline { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPURenderPipeline) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPURenderPipeline) -> Self {
        if !handle.is_null() { wgpuRenderPipelineAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPURenderPipeline {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn GetBindGroupLayout(&self, arg1: u32) -> BindGroupLayout {
        BindGroupLayout::Acquire(wgpuRenderPipelineGetBindGroupLayout(self.handle, arg1))
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuRenderPipelineSetLabel(self.handle, arg1)
    }
}

impl Default for RenderPipeline {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for RenderPipeline {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuRenderPipelineAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for RenderPipeline {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuRenderPipelineRelease(self.handle); } }
    }
}
impl PartialEq for RenderPipeline {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for RenderPipeline {}

#[repr(transparent)]
pub(crate) struct Sampler { handle: WGPUSampler }
impl Sampler {
    pub(crate) const fn Get(&self) -> WGPUSampler { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUSampler) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUSampler) -> Self {
        if !handle.is_null() { wgpuSamplerAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUSampler {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuSamplerSetLabel(self.handle, arg1)
    }
}

impl Default for Sampler {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for Sampler {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuSamplerAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for Sampler {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuSamplerRelease(self.handle); } }
    }
}
impl PartialEq for Sampler {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for Sampler {}

#[repr(transparent)]
pub(crate) struct ShaderModule { handle: WGPUShaderModule }
impl ShaderModule {
    pub(crate) const fn Get(&self) -> WGPUShaderModule { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUShaderModule) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUShaderModule) -> Self {
        if !handle.is_null() { wgpuShaderModuleAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUShaderModule {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn GetCompilationInfo(&self, arg1: WGPUCompilationInfoCallbackInfo) -> WGPUFuture {
        wgpuShaderModuleGetCompilationInfo(self.handle, arg1)
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuShaderModuleSetLabel(self.handle, arg1)
    }
}

impl Default for ShaderModule {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for ShaderModule {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuShaderModuleAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for ShaderModule {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuShaderModuleRelease(self.handle); } }
    }
}
impl PartialEq for ShaderModule {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for ShaderModule {}

#[repr(transparent)]
pub(crate) struct Surface { handle: WGPUSurface }
impl Surface {
    pub(crate) const fn Get(&self) -> WGPUSurface { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUSurface) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUSurface) -> Self {
        if !handle.is_null() { wgpuSurfaceAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUSurface {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn Configure(&self, arg1: *const WGPUSurfaceConfiguration) {
        wgpuSurfaceConfigure(self.handle, arg1)
    }
    pub(crate) unsafe fn GetCapabilities(&self, adapter: WGPUAdapter, output: &mut SurfaceCapabilities) -> ConvertibleStatus {
        ConvertibleStatus(Status::from(wgpuSurfaceGetCapabilities(self.handle, adapter, output.resetForOutput())))
    }
    pub(crate) unsafe fn GetCurrentTexture(&self, arg1: *mut WGPUSurfaceTexture) {
        wgpuSurfaceGetCurrentTexture(self.handle, arg1)
    }
    pub(crate) unsafe fn Present(&self) -> ConvertibleStatus {
        ConvertibleStatus(Status::from(wgpuSurfacePresent(self.handle)))
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuSurfaceSetLabel(self.handle, arg1)
    }
    pub(crate) unsafe fn Unconfigure(&self) {
        wgpuSurfaceUnconfigure(self.handle)
    }
}

impl Default for Surface {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for Surface {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuSurfaceAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for Surface {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuSurfaceRelease(self.handle); } }
    }
}
impl PartialEq for Surface {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for Surface {}

#[repr(transparent)]
pub(crate) struct Texture { handle: WGPUTexture }
impl Texture {
    pub(crate) const fn Get(&self) -> WGPUTexture { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUTexture) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUTexture) -> Self {
        if !handle.is_null() { wgpuTextureAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUTexture {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn CreateView(&self, arg1: *const WGPUTextureViewDescriptor) -> TextureView {
        TextureView::Acquire(wgpuTextureCreateView(self.handle, arg1))
    }
    pub(crate) unsafe fn Destroy(&self) {
        wgpuTextureDestroy(self.handle)
    }
    pub(crate) unsafe fn GetDepthOrArrayLayers(&self) -> u32 {
        wgpuTextureGetDepthOrArrayLayers(self.handle)
    }
    pub(crate) unsafe fn GetDimension(&self) -> TextureDimension {
        TextureDimension::from(wgpuTextureGetDimension(self.handle))
    }
    pub(crate) unsafe fn GetFormat(&self) -> TextureFormat {
        TextureFormat::from(wgpuTextureGetFormat(self.handle))
    }
    pub(crate) unsafe fn GetHeight(&self) -> u32 {
        wgpuTextureGetHeight(self.handle)
    }
    pub(crate) unsafe fn GetMipLevelCount(&self) -> u32 {
        wgpuTextureGetMipLevelCount(self.handle)
    }
    pub(crate) unsafe fn GetSampleCount(&self) -> u32 {
        wgpuTextureGetSampleCount(self.handle)
    }
    pub(crate) unsafe fn GetUsage(&self) -> TextureUsage {
        TextureUsage::from(wgpuTextureGetUsage(self.handle))
    }
    pub(crate) unsafe fn GetWidth(&self) -> u32 {
        wgpuTextureGetWidth(self.handle)
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuTextureSetLabel(self.handle, arg1)
    }
}

impl Default for Texture {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for Texture {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuTextureAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for Texture {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuTextureRelease(self.handle); } }
    }
}
impl PartialEq for Texture {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for Texture {}

#[repr(transparent)]
pub(crate) struct TextureView { handle: WGPUTextureView }
impl TextureView {
    pub(crate) const fn Get(&self) -> WGPUTextureView { self.handle }
    pub(crate) unsafe fn Acquire(handle: WGPUTextureView) -> Self { Self { handle } }
    pub(crate) unsafe fn FromBorrowed(handle: WGPUTextureView) -> Self {
        if !handle.is_null() { wgpuTextureViewAddRef(handle); }
        Self { handle }
    }
    pub(crate) fn MoveToCHandle(mut self) -> WGPUTextureView {
        let handle = self.handle;
        self.handle = std::ptr::null_mut();
        handle
    }
    pub(crate) unsafe fn SetLabel(&self, arg1: WGPUStringView) {
        wgpuTextureViewSetLabel(self.handle, arg1)
    }
}

impl Default for TextureView {
    fn default() -> Self { Self { handle: std::ptr::null_mut() } }
}
impl Clone for TextureView {
    fn clone(&self) -> Self {
        unsafe { if !self.handle.is_null() { wgpuTextureViewAddRef(self.handle); } }
        Self { handle: self.handle }
    }
}
impl Drop for TextureView {
    fn drop(&mut self) {
        unsafe { if !self.handle.is_null() { wgpuTextureViewRelease(self.handle); } }
    }
}
impl PartialEq for TextureView {
    fn eq(&self, other: &Self) -> bool { self.handle == other.handle }
}
impl Eq for TextureView {}

pub(crate) unsafe fn CreateInstance(descriptor: Option<&InstanceDescriptor>) -> Instance {
    let raw = descriptor.map_or(std::ptr::null(), core::ptr::from_ref);
    unsafe { Instance::Acquire(wgpuCreateInstance(raw)) }
}

pub(crate) unsafe fn GetInstanceFeatures(output: &mut SupportedInstanceFeatures) {
    unsafe { wgpuGetInstanceFeatures(output.resetForOutput()) };
}

pub(crate) unsafe fn GetInstanceLimits(output: &mut InstanceLimits) -> Status {
    unsafe { Status::from(wgpuGetInstanceLimits(output)) }
}

pub(crate) unsafe fn HasInstanceFeature(feature: InstanceFeatureName) -> Bool {
    unsafe { Bool::from(wgpuHasInstanceFeature(feature.into())) }
}

pub(crate) unsafe fn GetProcAddress(procName: StringView) -> Proc {
    unsafe { wgpuGetProcAddress(procName) }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct SourceSymbol {
    pub(crate) owner: &'static str,
    pub(crate) name: &'static str,
}

pub(crate) const CPP_METHOD_DEFINITIONS: &[SourceSymbol] = &[
    SourceSymbol { owner: "Derived", name: "WGPURelease" },
    SourceSymbol { owner: "std", name: "string_view" },
    SourceSymbol { owner: "StringView", name: "StringView" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "AdapterInfo", name: "FreeMembers" },
    SourceSymbol { owner: "AdapterInfo", name: "Reset" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "SupportedFeatures", name: "FreeMembers" },
    SourceSymbol { owner: "SupportedFeatures", name: "Reset" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "SupportedInstanceFeatures", name: "FreeMembers" },
    SourceSymbol { owner: "SupportedInstanceFeatures", name: "Reset" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "SupportedWGSLLanguageFeatures", name: "FreeMembers" },
    SourceSymbol { owner: "SupportedWGSLLanguageFeatures", name: "Reset" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "SurfaceCapabilities", name: "FreeMembers" },
    SourceSymbol { owner: "SurfaceCapabilities", name: "Reset" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "detail", name: "AsNonConstReference" },
    SourceSymbol { owner: "std", name: "move" },
    SourceSymbol { owner: "std", name: "move" },
    SourceSymbol { owner: "std", name: "move" },
    SourceSymbol { owner: "std", name: "move" },
    SourceSymbol { owner: "std", name: "move" },
    SourceSymbol { owner: "DeviceDescriptor", name: "SetDeviceLostCallback" },
    SourceSymbol { owner: "DeviceDescriptor", name: "SetDeviceLostCallback" },
    SourceSymbol { owner: "DeviceDescriptor", name: "SetUncapturedErrorCallback" },
    SourceSymbol { owner: "DeviceDescriptor", name: "SetUncapturedErrorCallback" },
    SourceSymbol { owner: "Adapter", name: "GetFeatures" },
    SourceSymbol { owner: "Adapter", name: "GetInfo" },
    SourceSymbol { owner: "Adapter", name: "GetLimits" },
    SourceSymbol { owner: "Adapter", name: "HasFeature" },
    SourceSymbol { owner: "Adapter", name: "RequestDevice" },
    SourceSymbol { owner: "Adapter", name: "RequestDevice" },
    SourceSymbol { owner: "Adapter", name: "WGPUAddRef" },
    SourceSymbol { owner: "Adapter", name: "WGPURelease" },
    SourceSymbol { owner: "BindGroup", name: "SetLabel" },
    SourceSymbol { owner: "BindGroup", name: "WGPUAddRef" },
    SourceSymbol { owner: "BindGroup", name: "WGPURelease" },
    SourceSymbol { owner: "BindGroupLayout", name: "SetLabel" },
    SourceSymbol { owner: "BindGroupLayout", name: "WGPUAddRef" },
    SourceSymbol { owner: "BindGroupLayout", name: "WGPURelease" },
    SourceSymbol { owner: "Buffer", name: "Destroy" },
    SourceSymbol { owner: "Buffer", name: "GetConstMappedRange" },
    SourceSymbol { owner: "Buffer", name: "GetMappedRange" },
    SourceSymbol { owner: "Buffer", name: "GetMapState" },
    SourceSymbol { owner: "Buffer", name: "GetSize" },
    SourceSymbol { owner: "Buffer", name: "GetUsage" },
    SourceSymbol { owner: "Buffer", name: "MapAsync" },
    SourceSymbol { owner: "Buffer", name: "MapAsync" },
    SourceSymbol { owner: "Buffer", name: "ReadMappedRange" },
    SourceSymbol { owner: "Buffer", name: "SetLabel" },
    SourceSymbol { owner: "Buffer", name: "Unmap" },
    SourceSymbol { owner: "Buffer", name: "WriteMappedRange" },
    SourceSymbol { owner: "Buffer", name: "WGPUAddRef" },
    SourceSymbol { owner: "Buffer", name: "WGPURelease" },
    SourceSymbol { owner: "CommandBuffer", name: "SetLabel" },
    SourceSymbol { owner: "CommandBuffer", name: "WGPUAddRef" },
    SourceSymbol { owner: "CommandBuffer", name: "WGPURelease" },
    SourceSymbol { owner: "CommandEncoder", name: "BeginComputePass" },
    SourceSymbol { owner: "ComputePassEncoder", name: "Acquire" },
    SourceSymbol { owner: "CommandEncoder", name: "BeginRenderPass" },
    SourceSymbol { owner: "RenderPassEncoder", name: "Acquire" },
    SourceSymbol { owner: "CommandEncoder", name: "ClearBuffer" },
    SourceSymbol { owner: "CommandEncoder", name: "CopyBufferToBuffer" },
    SourceSymbol { owner: "CommandEncoder", name: "CopyBufferToTexture" },
    SourceSymbol { owner: "CommandEncoder", name: "CopyTextureToBuffer" },
    SourceSymbol { owner: "CommandEncoder", name: "CopyTextureToTexture" },
    SourceSymbol { owner: "CommandEncoder", name: "Finish" },
    SourceSymbol { owner: "CommandBuffer", name: "Acquire" },
    SourceSymbol { owner: "CommandEncoder", name: "InsertDebugMarker" },
    SourceSymbol { owner: "CommandEncoder", name: "PopDebugGroup" },
    SourceSymbol { owner: "CommandEncoder", name: "PushDebugGroup" },
    SourceSymbol { owner: "CommandEncoder", name: "ResolveQuerySet" },
    SourceSymbol { owner: "CommandEncoder", name: "SetLabel" },
    SourceSymbol { owner: "CommandEncoder", name: "WriteTimestamp" },
    SourceSymbol { owner: "CommandEncoder", name: "WGPUAddRef" },
    SourceSymbol { owner: "CommandEncoder", name: "WGPURelease" },
    SourceSymbol { owner: "ComputePassEncoder", name: "DispatchWorkgroups" },
    SourceSymbol { owner: "ComputePassEncoder", name: "DispatchWorkgroupsIndirect" },
    SourceSymbol { owner: "ComputePassEncoder", name: "End" },
    SourceSymbol { owner: "ComputePassEncoder", name: "InsertDebugMarker" },
    SourceSymbol { owner: "ComputePassEncoder", name: "PopDebugGroup" },
    SourceSymbol { owner: "ComputePassEncoder", name: "PushDebugGroup" },
    SourceSymbol { owner: "ComputePassEncoder", name: "SetBindGroup" },
    SourceSymbol { owner: "ComputePassEncoder", name: "SetLabel" },
    SourceSymbol { owner: "ComputePassEncoder", name: "SetPipeline" },
    SourceSymbol { owner: "ComputePassEncoder", name: "WriteTimestamp" },
    SourceSymbol { owner: "ComputePassEncoder", name: "WGPUAddRef" },
    SourceSymbol { owner: "ComputePassEncoder", name: "WGPURelease" },
    SourceSymbol { owner: "ComputePipeline", name: "GetBindGroupLayout" },
    SourceSymbol { owner: "BindGroupLayout", name: "Acquire" },
    SourceSymbol { owner: "ComputePipeline", name: "SetLabel" },
    SourceSymbol { owner: "ComputePipeline", name: "WGPUAddRef" },
    SourceSymbol { owner: "ComputePipeline", name: "WGPURelease" },
    SourceSymbol { owner: "Device", name: "CreateBindGroup" },
    SourceSymbol { owner: "BindGroup", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateBindGroupLayout" },
    SourceSymbol { owner: "BindGroupLayout", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateBuffer" },
    SourceSymbol { owner: "Buffer", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateCommandEncoder" },
    SourceSymbol { owner: "CommandEncoder", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateComputePipeline" },
    SourceSymbol { owner: "ComputePipeline", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateComputePipelineAsync" },
    SourceSymbol { owner: "Device", name: "CreateComputePipelineAsync" },
    SourceSymbol { owner: "Device", name: "CreatePipelineLayout" },
    SourceSymbol { owner: "PipelineLayout", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateQuerySet" },
    SourceSymbol { owner: "QuerySet", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateRenderBundleEncoder" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateRenderPipeline" },
    SourceSymbol { owner: "RenderPipeline", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateRenderPipelineAsync" },
    SourceSymbol { owner: "Device", name: "CreateRenderPipelineAsync" },
    SourceSymbol { owner: "Device", name: "CreateSampler" },
    SourceSymbol { owner: "Sampler", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateShaderModule" },
    SourceSymbol { owner: "ShaderModule", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "CreateTexture" },
    SourceSymbol { owner: "Texture", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "Destroy" },
    SourceSymbol { owner: "Device", name: "GetAdapterInfo" },
    SourceSymbol { owner: "Device", name: "GetFeatures" },
    SourceSymbol { owner: "Device", name: "GetLimits" },
    SourceSymbol { owner: "Device", name: "GetLostFuture" },
    SourceSymbol { owner: "Device", name: "GetQueue" },
    SourceSymbol { owner: "Queue", name: "Acquire" },
    SourceSymbol { owner: "Device", name: "HasFeature" },
    SourceSymbol { owner: "Device", name: "PopErrorScope" },
    SourceSymbol { owner: "Device", name: "PopErrorScope" },
    SourceSymbol { owner: "Device", name: "PushErrorScope" },
    SourceSymbol { owner: "Device", name: "SetLabel" },
    SourceSymbol { owner: "Device", name: "WGPUAddRef" },
    SourceSymbol { owner: "Device", name: "WGPURelease" },
    SourceSymbol { owner: "Instance", name: "CreateSurface" },
    SourceSymbol { owner: "Surface", name: "Acquire" },
    SourceSymbol { owner: "Instance", name: "GetWGSLLanguageFeatures" },
    SourceSymbol { owner: "Instance", name: "HasWGSLLanguageFeature" },
    SourceSymbol { owner: "Instance", name: "ProcessEvents" },
    SourceSymbol { owner: "Instance", name: "RequestAdapter" },
    SourceSymbol { owner: "Instance", name: "RequestAdapter" },
    SourceSymbol { owner: "Instance", name: "WaitAny" },
    SourceSymbol { owner: "Instance", name: "WaitAny" },
    SourceSymbol { owner: "Instance", name: "WGPUAddRef" },
    SourceSymbol { owner: "Instance", name: "WGPURelease" },
    SourceSymbol { owner: "PipelineLayout", name: "SetLabel" },
    SourceSymbol { owner: "PipelineLayout", name: "WGPUAddRef" },
    SourceSymbol { owner: "PipelineLayout", name: "WGPURelease" },
    SourceSymbol { owner: "QuerySet", name: "Destroy" },
    SourceSymbol { owner: "QuerySet", name: "GetCount" },
    SourceSymbol { owner: "QuerySet", name: "GetType" },
    SourceSymbol { owner: "QuerySet", name: "SetLabel" },
    SourceSymbol { owner: "QuerySet", name: "WGPUAddRef" },
    SourceSymbol { owner: "QuerySet", name: "WGPURelease" },
    SourceSymbol { owner: "Queue", name: "OnSubmittedWorkDone" },
    SourceSymbol { owner: "Queue", name: "OnSubmittedWorkDone" },
    SourceSymbol { owner: "Queue", name: "SetLabel" },
    SourceSymbol { owner: "Queue", name: "Submit" },
    SourceSymbol { owner: "Queue", name: "WriteBuffer" },
    SourceSymbol { owner: "Queue", name: "WriteTexture" },
    SourceSymbol { owner: "Queue", name: "WGPUAddRef" },
    SourceSymbol { owner: "Queue", name: "WGPURelease" },
    SourceSymbol { owner: "RenderBundle", name: "SetLabel" },
    SourceSymbol { owner: "RenderBundle", name: "WGPUAddRef" },
    SourceSymbol { owner: "RenderBundle", name: "WGPURelease" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "Draw" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "DrawIndexed" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "DrawIndexedIndirect" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "DrawIndirect" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "Finish" },
    SourceSymbol { owner: "RenderBundle", name: "Acquire" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "InsertDebugMarker" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "PopDebugGroup" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "PushDebugGroup" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "SetBindGroup" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "SetIndexBuffer" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "SetLabel" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "SetPipeline" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "SetVertexBuffer" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "WGPUAddRef" },
    SourceSymbol { owner: "RenderBundleEncoder", name: "WGPURelease" },
    SourceSymbol { owner: "RenderPassEncoder", name: "BeginOcclusionQuery" },
    SourceSymbol { owner: "RenderPassEncoder", name: "Draw" },
    SourceSymbol { owner: "RenderPassEncoder", name: "DrawIndexed" },
    SourceSymbol { owner: "RenderPassEncoder", name: "DrawIndexedIndirect" },
    SourceSymbol { owner: "RenderPassEncoder", name: "DrawIndirect" },
    SourceSymbol { owner: "RenderPassEncoder", name: "End" },
    SourceSymbol { owner: "RenderPassEncoder", name: "EndOcclusionQuery" },
    SourceSymbol { owner: "RenderPassEncoder", name: "ExecuteBundles" },
    SourceSymbol { owner: "RenderPassEncoder", name: "InsertDebugMarker" },
    SourceSymbol { owner: "RenderPassEncoder", name: "MultiDrawIndexedIndirect" },
    SourceSymbol { owner: "RenderPassEncoder", name: "MultiDrawIndirect" },
    SourceSymbol { owner: "RenderPassEncoder", name: "PopDebugGroup" },
    SourceSymbol { owner: "RenderPassEncoder", name: "PushDebugGroup" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetBindGroup" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetBlendConstant" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetIndexBuffer" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetLabel" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetPipeline" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetScissorRect" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetStencilReference" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetVertexBuffer" },
    SourceSymbol { owner: "RenderPassEncoder", name: "SetViewport" },
    SourceSymbol { owner: "RenderPassEncoder", name: "WriteTimestamp" },
    SourceSymbol { owner: "RenderPassEncoder", name: "WGPUAddRef" },
    SourceSymbol { owner: "RenderPassEncoder", name: "WGPURelease" },
    SourceSymbol { owner: "RenderPipeline", name: "GetBindGroupLayout" },
    SourceSymbol { owner: "BindGroupLayout", name: "Acquire" },
    SourceSymbol { owner: "RenderPipeline", name: "SetLabel" },
    SourceSymbol { owner: "RenderPipeline", name: "WGPUAddRef" },
    SourceSymbol { owner: "RenderPipeline", name: "WGPURelease" },
    SourceSymbol { owner: "Sampler", name: "SetLabel" },
    SourceSymbol { owner: "Sampler", name: "WGPUAddRef" },
    SourceSymbol { owner: "Sampler", name: "WGPURelease" },
    SourceSymbol { owner: "ShaderModule", name: "GetCompilationInfo" },
    SourceSymbol { owner: "ShaderModule", name: "GetCompilationInfo" },
    SourceSymbol { owner: "ShaderModule", name: "SetLabel" },
    SourceSymbol { owner: "ShaderModule", name: "WGPUAddRef" },
    SourceSymbol { owner: "ShaderModule", name: "WGPURelease" },
    SourceSymbol { owner: "Surface", name: "Configure" },
    SourceSymbol { owner: "Surface", name: "GetCapabilities" },
    SourceSymbol { owner: "Surface", name: "GetCurrentTexture" },
    SourceSymbol { owner: "Surface", name: "Present" },
    SourceSymbol { owner: "Surface", name: "SetLabel" },
    SourceSymbol { owner: "Surface", name: "Unconfigure" },
    SourceSymbol { owner: "Surface", name: "WGPUAddRef" },
    SourceSymbol { owner: "Surface", name: "WGPURelease" },
    SourceSymbol { owner: "Texture", name: "CreateView" },
    SourceSymbol { owner: "TextureView", name: "Acquire" },
    SourceSymbol { owner: "Texture", name: "Destroy" },
    SourceSymbol { owner: "Texture", name: "GetDepthOrArrayLayers" },
    SourceSymbol { owner: "Texture", name: "GetDimension" },
    SourceSymbol { owner: "Texture", name: "GetFormat" },
    SourceSymbol { owner: "Texture", name: "GetHeight" },
    SourceSymbol { owner: "Texture", name: "GetMipLevelCount" },
    SourceSymbol { owner: "Texture", name: "GetSampleCount" },
    SourceSymbol { owner: "Texture", name: "GetUsage" },
    SourceSymbol { owner: "Texture", name: "GetWidth" },
    SourceSymbol { owner: "Texture", name: "SetLabel" },
    SourceSymbol { owner: "Texture", name: "WGPUAddRef" },
    SourceSymbol { owner: "Texture", name: "WGPURelease" },
    SourceSymbol { owner: "TextureView", name: "SetLabel" },
    SourceSymbol { owner: "TextureView", name: "WGPUAddRef" },
    SourceSymbol { owner: "TextureView", name: "WGPURelease" },
    SourceSymbol { owner: "Instance", name: "Acquire" },
];

pub(crate) const RAW_MEMBER_ENTRY_POINTS: &[&str] = &[
    "wgpuAdapterAddRef",
    "wgpuAdapterGetFeatures",
    "wgpuAdapterGetInfo",
    "wgpuAdapterGetLimits",
    "wgpuAdapterHasFeature",
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
    "wgpuSurfaceAddRef",
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

pub(crate) const RAW_FREE_ENTRY_POINTS: &[&str] = &[
    "wgpuCreateInstance",
    "wgpuGetInstanceFeatures",
    "wgpuGetInstanceLimits",
    "wgpuHasInstanceFeature",
    "wgpuGetProcAddress",
    "wgpuAdapterInfoFreeMembers",
    "wgpuSupportedFeaturesFreeMembers",
    "wgpuSupportedInstanceFeaturesFreeMembers",
    "wgpuSupportedWGSLLanguageFeaturesFreeMembers",
    "wgpuSurfaceCapabilitiesFreeMembers",
];

pub(crate) const CPP_ENUM_CLASS_COUNT: usize = 58;
pub(crate) const CPP_ABI_PAIR_COUNT: usize = 156;
pub(crate) const CPP_METHOD_DEFINITION_COUNT: usize = 288;
pub(crate) const CPP_TEMPLATE_DECLARATION_COUNT: usize = 43;
pub(crate) const CPP_STATIC_ASSERT_COUNT: usize = 663;
pub(crate) const C_STRUCTURE_INITIALIZER_COUNT: usize = 86;
pub(crate) const RAW_MEMBER_ENTRY_POINT_COUNT: usize = 189;
pub(crate) const RAW_FREE_ENTRY_POINT_COUNT: usize = 10;
const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, size_of};

    #[test]
    fn complete_generated_denominators_are_locked() {
        assert_eq!(CPP_ENUM_CLASS_COUNT, 58);
        assert_eq!(CPP_ABI_PAIR_COUNT, 156);
        assert_eq!(CPP_METHOD_DEFINITION_COUNT, 288);
        assert_eq!(CPP_TEMPLATE_DECLARATION_COUNT, 43);
        assert_eq!(C_STRUCTURE_INITIALIZER_COUNT, 86);
        assert_eq!(RAW_MEMBER_ENTRY_POINT_COUNT + RAW_FREE_ENTRY_POINT_COUNT, 199);
        assert_eq!(RAW_MEMBER_ENTRY_POINTS.len(), RAW_MEMBER_ENTRY_POINT_COUNT);
        assert_eq!(RAW_FREE_ENTRY_POINTS.len(), RAW_FREE_ENTRY_POINT_COUNT);
    }

    #[test]
    fn generated_defaults_preserve_nonzero_and_chain_initializers() {
        let binding = WGPUBindGroupEntry::default();
        assert_eq!(binding.size, WGPU_WHOLE_SIZE);
        let sampler = WGPUSamplerDescriptor::default();
        assert_eq!(sampler.lodMaxClamp, 32.0);
        let max_draw = WGPURenderPassMaxDrawCount::default();
        assert_eq!(max_draw.chain.sType, WGPUSType_RenderPassMaxDrawCount);
        assert_eq!(max_draw.maxDrawCount, 50_000_000);
        let depth = WGPURenderPassDepthStencilAttachment::default();
        assert!(depth.depthClearValue.is_nan());
    }

    #[test]
    fn enum_and_object_abi_is_transparent() {
        assert_eq!(size_of::<TextureFormat>(), size_of::<WGPUTextureFormat>());
        assert_eq!(align_of::<TextureFormat>(), align_of::<WGPUTextureFormat>());
        assert_eq!(size_of::<Device>(), size_of::<WGPUDevice>());
        assert_eq!(align_of::<Device>(), align_of::<WGPUDevice>());
    }

    #[test]
    fn optional_bool_preserves_three_source_states() {
        assert_eq!(OptionalBool::Undefined.intoOption(), None);
        assert_eq!(OptionalBool::False.intoOption(), Some(false));
        assert_eq!(OptionalBool::True.intoOption(), Some(true));
    }
}
