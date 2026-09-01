//! Public Rust entry point for the translated Rive runtime.
//!
//! `File` is the upstream owner, not the former descriptor/graph facade.
//! Imports require a retained renderer factory. Artboard instances keep that
//! factory and draw with a renderer; there is no factory-free import, late
//! attachment, or backwards-compatibility execution path.
//!
//! Host authorization and allocation limits are explicit import boundaries.
//! Applications and bindings must migrate to this API separately.

pub use nuxie_runtime::source as runtime;
// Renderer DTOs have their own namespace; the primary math names below are
// the types accepted and returned by the native runtime owners.
pub use nuxie_render_api as render_api;
pub use runtime::{
    advance_flags::AdvanceFlags,
    animation::{
        linear_animation::LinearAnimation,
        state_machine::StateMachine,
        state_machine_input_instance::{SMIBool, SMIInput, SMINumber, SMITrigger},
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    bindable_artboard::RuntimeBindableArtboardHandle,
    command_queue, command_server,
    core::CoreHandle,
    factory::RuntimeFactoryHandle,
    file::{File, ImportResult, RuntimeFileHandle, RuntimeFileWeakHandle},
    file_asset_loader::{FileAssetLoader, FileAssetLoaderRef},
    lua::scripting_vm::RuntimeScriptingVmHandle,
    math::{aabb::Aabb, mat2d::Mat2D, path_types::PathVerb, raw_path::RawPath, vec2d::Vec2D},
    text::raw_text::RawText,
    text_engine::{FontRef, TextAlign, TextOrigin, TextOverflow, TextSizing, TextWrap},
    viewmodel::{
        runtime::{
            viewmodel_instance_runtime::{
                RuntimeViewModelInstanceHandle, ViewModelInstanceRuntime,
            },
            viewmodel_runtime::RuntimeViewModelHandle,
        },
        viewmodel::ViewModel,
    },
};

pub use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, GpuCanvasAttachmentView, GpuCanvasBlendState,
    GpuCanvasColorAttachment, GpuCanvasColorTarget, GpuCanvasDepthStencilAttachment,
    GpuCanvasDepthStencilState, GpuCanvasDrawCommand, GpuCanvasError, GpuCanvasIndexBuffer,
    GpuCanvasIndexedDraw, GpuCanvasPassState, GpuCanvasPipelinePlan, GpuCanvasPipelineShaders,
    GpuCanvasPipelineState, GpuCanvasPlan, GpuCanvasRenderPass, GpuCanvasResourceLifetime,
    GpuCanvasSamplerBinding, GpuCanvasShader, GpuCanvasShaderArtifact, GpuCanvasShaderBinding,
    GpuCanvasShaderEntry, GpuCanvasShaderEntrySelection, GpuCanvasShaderLoad,
    GpuCanvasShaderProfile, GpuCanvasShaderResourceKind, GpuCanvasShaderStage,
    GpuCanvasShaderTextureSampleType, GpuCanvasShaderTextureViewDimension, GpuCanvasStencilFace,
    GpuCanvasTextureBinding, GpuCanvasTextureUpload, ImageDecodeError, ImageFilter, ImageSampler,
    ImageWrap, PersistentFactory, PersistentFactoryContext, RecordingFactory, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderCanvas, RenderCanvasError, RenderCanvasFrame,
    RenderGpuCanvasShader, RenderImage, RenderPaint, RenderPaintStyle, RenderPath, RenderShader,
    Renderer, StrokeCap, StrokeJoin,
};
#[cfg(all(
    feature = "renderer-metal",
    any(
        target_os = "ios",
        target_os = "macos",
        target_os = "tvos",
        target_os = "visionos"
    )
))]
pub use nuxie_renderer::{
    NativeMetalContextOptions, NativeMetalDrawableFrame, NativeMetalExecutionInventory,
    NativeMetalFactory, NativeMetalFrame, NativeMetalFrameOutput,
    NativeMetalSynthesizedFailureType, ShaderCompilationMode,
};
#[cfg(feature = "renderer-vulkan")]
pub use nuxie_renderer::{NativeVulkanFactory, NativeVulkanFrame};
#[cfg(feature = "renderer-webgpu")]
pub use nuxie_renderer::{NativeWebGpuFactory, NativeWebGpuFrame};
#[cfg(any(
    feature = "renderer-metal",
    feature = "renderer-vulkan",
    feature = "renderer-webgpu",
    feature = "renderer-webgl2"
))]
pub use nuxie_renderer::{RenderMode, RendererError};
#[cfg(all(
    feature = "renderer-webgl2",
    target_arch = "wasm32",
    target_os = "unknown"
))]
pub use nuxie_renderer::{WebGl2Factory, WebGl2Frame};

pub use nuxie_runtime::{
    ArtboardInstance, AudioDecodeError, AudioEngine, AudioEngineError, AudioFormat, AudioReader,
    AudioSound, AudioSource, LinearAnimationInstance, RuntimeAudioAssetOwners, RuntimeBlobAsset,
    RuntimeEventPropertyValue, RuntimeFontAssetOwners, RuntimeHitResult, RuntimeImageAssetOwners,
    RuntimeOwnedViewModelGraphTransaction, RuntimeOwnedViewModelHandle,
    RuntimeOwnedViewModelInstance, RuntimeOwnedViewModelTransaction, RuntimeScriptProgram,
    RuntimeViewModelChange, RuntimeViewModelChangeCapture, RuntimeViewModelChangeValue,
    RuntimeViewModelGraphTransactionError, RuntimeViewModelLinkError, ScriptAssetRegistration,
    ScriptAssetRegistrationResult, ScriptDataConverterMethod, ScriptDataConverterOptionalCall,
    ScriptError, ScriptHost, ScriptInstance, ScriptMethod, ScriptOptionalMethodResult,
    ScriptProgramAdapter, ScriptValue, ScriptViewModel, ScriptedContextSource, ScriptingVm,
    StateMachineInstance, StateMachineReportedEvent,
};
pub use runtime::assets::{
    audio_asset::AudioAsset, font_asset::FontAsset, image_asset::ImageAsset,
    script_asset::ScriptAsset,
};
pub use runtime::scripted::scripted_drawable::ScriptedDrawable;

mod import_limits;
mod native_file;
pub use import_limits::FileImportLimits;
pub use native_file::import_native;

#[cfg(feature = "scripting")]
mod script_import;
#[cfg(feature = "scripting")]
pub use nuxie_scripting::host_commands::{HostCommand, HostCommandLimits, HostValue};
#[cfg(feature = "scripting")]
pub use nuxie_scripting::vm::{
    ScriptExecutionLimits, ScriptVm, ScriptingLogLevel, ScriptingLogSink,
};
#[cfg(feature = "scripting")]
pub use script_import::*;

#[cfg(all(
    feature = "ore-metal-authored-msl",
    any(target_os = "ios", target_os = "macos")
))]
#[doc(hidden)]
pub mod ore_metal_gpu_canvas;

// Assertions are retained, not disabled to conceal the deliberate API break.
// Adapting their old call sites is part of the later consumer/validation work.
#[cfg(test)]
mod previous_command_server_tests;
#[cfg(test)]
mod previous_facade_tests;
#[cfg(all(test, feature = "scripting"))]
mod scripted_interpolator_tests;
#[cfg(test)]
mod wave_c1_no_loader_owner_tests;
