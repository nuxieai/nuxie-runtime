//! Product bridge from the mechanically translated generic render context to
//! the mechanically translated Metal execution engine.
//!
//! This owner deliberately does not introduce another shader scheduler or
//! compiler. `MechanicalMetalHost` routes the source engine's compilation
//! callbacks into the one mechanically translated Metal context.

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use core::ffi::c_void;
use core::mem::ManuallyDrop;
use core::pin::Pin;
use core::ptr::NonNull;
use std::sync::{Arc, Condvar, Mutex};
use std::time::Instant;

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{MTLCommandBuffer, MTLCommandQueue, MTLDevice, MTLPixelFormat, MTLTexture};

use crate::mechanical_metal_implementation as mechanical_metal;
use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat;
use crate::mechanical_port::source::include::rive::factory_hpp::FactoryContract;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    FlushDescriptor, StorageBufferStructure,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, FrameDescriptor, RenderContext,
};
#[cfg(feature = "rive-decoders")]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    BitmapDecodeResult, BitmapDecoderContract, BitmapPixelFormat,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::{
    RenderContextImpl, RenderContextImplContract,
};
#[cfg(feature = "native-ore-metal-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
#[cfg(feature = "native-ore-metal-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::OreContext;
#[cfg(feature = "native-ore-metal-experimental")]
use nuxie_ore_metal::metal::context::ContextMetal as OreContextMetal;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::{RenderResourceDomain, RiveRenderBufferHandle};
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;
use crate::mechanical_port::source::renderer::src::gradient_hpp::GradientShader;
use crate::mechanical_port::source::renderer::src::rive_render_paint_hpp::RiveRenderPaintHandle;
use crate::mechanical_port::source::renderer::src::rive_render_path_hpp::RiveRenderPathHandle;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use crate::mechanical_port::source::renderer::include::rive::renderer::metal::render_context_metal_impl_h::{
    ContextOptions as SourceContextOptions,
    ShaderCompilationMode as SourceContextShaderCompilationMode,
};
use crate::{RenderMode, RendererError};

use super::command_submission::{make_command_buffer_on_queue, NativeMetalSubmissionCompletion};
use super::context_options::{NativeMetalContextOptions, ShaderCompilationMode};
use super::objc2_execution::{ActualMetalExecutionInventory, Objc2MetalExecution};
use super::source_capabilities::MetalCapabilitySelection;
use super::MechanicalMetalHost;

use mechanical_metal::source_execution::{
    Handle, MetalExecution, PixelFormat, RenderContextMetal, RenderTargetMetal,
};

const FLUSH_UNIFORM_RING: &str = "flushUniform";
const PATH_RING: &str = "path";
const PAINT_RING: &str = "paint";
const PAINT_AUX_RING: &str = "paintAux";
const CONTOUR_RING: &str = "contour";
const GRAD_SPAN_RING: &str = "gradSpan";
const TESS_VERTEX_SPAN_RING: &str = "tessSpan";
const TRIANGLE_RING: &str = "triangle";
const IMAGE_DRAW_INSTANCE_RING: &str = "imageDrawInstance";

#[cfg(test)]
std::thread_local! {
    /// One-shot unwind injection after the real source `postFlush` has armed
    /// its raw ring callback but before `flushExecutable` returns.
    static PANIC_AFTER_POST_FLUSH: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
fn inject_panic_after_post_flush_once() {
    PANIC_AFTER_POST_FLUSH.with(|slot| slot.set(true));
}

#[cfg(test)]
fn take_panic_after_post_flush() -> bool {
    PANIC_AFTER_POST_FLUSH.with(|slot| slot.replace(false))
}

fn source_compilation_mode(mode: ShaderCompilationMode) -> SourceContextShaderCompilationMode {
    match mode {
        ShaderCompilationMode::AllowAsynchronous => {
            SourceContextShaderCompilationMode::allowAsynchronous
        }
        ShaderCompilationMode::AlwaysSynchronous => {
            SourceContextShaderCompilationMode::alwaysSynchronous
        }
        ShaderCompilationMode::OnlyUbershaders => {
            SourceContextShaderCompilationMode::onlyUbershaders
        }
    }
}

fn source_context_options(options: NativeMetalContextOptions) -> SourceContextOptions {
    SourceContextOptions {
        shaderCompilationMode: source_compilation_mode(options.shader_compilation_mode),
        disableFramebufferReads: options.disable_framebuffer_reads,
        #[cfg(feature = "with-rive-tools")]
        synthesizedFailureType: match options.synthesized_failure_type {
            super::context_options::NativeMetalSynthesizedFailureType::None => {
                crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType::none
            }
            super::context_options::NativeMetalSynthesizedFailureType::UbershaderLoad => {
                crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType::ubershaderLoad
            }
            super::context_options::NativeMetalSynthesizedFailureType::ShaderCompilation => {
                crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType::shaderCompilation
            }
            super::context_options::NativeMetalSynthesizedFailureType::PipelineCreation => {
                crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::SynthesizedFailureType::pipelineCreation
            }
        },
    }
}

#[cfg(feature = "rive-decoders")]
struct MechanicalBitmapDecoder;

#[cfg(feature = "rive-decoders")]
impl BitmapDecoderContract for MechanicalBitmapDecoder {
    fn decodeBitmap(&mut self, encoded: &[u8]) -> Option<BitmapDecodeResult> {
        let dimensions = nuxie_image_codec::preflight_encoded_image(encoded)?;
        let decoded = nuxie_image_codec::decode_image_rgba(encoded)?;
        if decoded.width != dimensions.width || decoded.height != dimensions.height {
            return None;
        }
        Some(BitmapDecodeResult {
            width: decoded.width,
            height: decoded.height,
            pixel_format: BitmapPixelFormat::rgbaPremul,
            bytes: decoded.pixels,
        })
    }

    fn convertToRGBAPremul(&mut self, bitmap: &mut BitmapDecodeResult) {
        // The canonical codec already returns premultiplied RGBA8.
        bitmap.pixel_format = BitmapPixelFormat::rgbaPremul;
    }
}

fn target_pixel_format(format: MTLPixelFormat) -> Option<PixelFormat> {
    match format {
        MTLPixelFormat::RGBA8Unorm => Some(PixelFormat::RGBA8Unorm),
        MTLPixelFormat::RGBA8Unorm_sRGB => Some(PixelFormat::RGBA8UnormSrgb),
        MTLPixelFormat::BGRA8Unorm => Some(PixelFormat::BGRA8Unorm),
        MTLPixelFormat::BGRA8Unorm_sRGB => Some(PixelFormat::BGRA8UnormSrgb),
        MTLPixelFormat::RGBA16Float => Some(PixelFormat::RGBA16Float),
        _ => None,
    }
}

#[cfg(feature = "native-ore-metal-experimental")]
pub(super) fn retained_canvas_target_texture(
    canvas: &RenderCanvas,
) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
    let target = canvas.render_target_ref();
    // SAFETY: RenderContextMetal::makeRenderCanvas installs the complete
    // MechanicalRenderTargetOwner allocation behind the source RenderTarget
    // base at offset zero. The source canvas keeps that intrusive owner alive
    // for the duration of this checked borrow.
    let owner = unsafe { &*core::ptr::from_ref(target).cast::<MechanicalRenderTargetOwner>() };
    owner.retained_target_texture()
}

#[repr(C)]
struct MechanicalRenderTargetOwner {
    metal: ManuallyDrop<RenderTargetMetal>,
}

impl MechanicalRenderTargetOwner {
    fn new(
        mut metal: RenderTargetMetal,
        texture: Option<Handle>,
        width: u32,
        height: u32,
        execution: &mut Objc2MetalExecution,
    ) -> Box<Self> {
        // For ordinary external targets, the transient registry creation +1
        // models the caller's assignment expression. `set_target_texture`
        // creates the target member's strong ARC owner, then the transient is
        // retired immediately. RenderCanvas already performed this exact
        // source assignment before reaching the adapter and therefore passes
        // no replacement here.
        if let Some(texture) = texture {
            metal.set_target_texture(execution, (texture != Handle::NIL).then_some(texture));
            execution.retire_handle(texture);
        }
        debug_assert_eq!(metal.base.width(), width);
        debug_assert_eq!(metal.base.height(), height);
        let mut owner = Box::new(Self {
            metal: ManuallyDrop::new(metal),
        });
        owner.metal.base.destroy_complete = destroy_render_target_owner;
        owner
    }

    fn retained_target_texture(&self) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
        let native = self.metal.m_targetTexture.as_ref()?.native_object()?;
        let native = unsafe { Retained::retain(core::ptr::from_ref(native).cast_mut()) }?;
        Some(unsafe { Retained::cast_unchecked::<ProtocolObject<dyn MTLTexture>>(native) })
    }
}

impl Drop for MechanicalRenderTargetOwner {
    fn drop(&mut self) {
        // Every source member invalidates its alias and releases its direct +1
        // from RenderTargetMetal's exact destructor. The adapter owns no
        // source resource and therefore performs no extraction or retirement.
        unsafe { ManuallyDrop::drop(&mut self.metal) };
    }
}

unsafe fn destroy_render_target_owner(base: *mut RenderTarget) {
    unsafe { drop(Box::from_raw(base.cast::<MechanicalRenderTargetOwner>())) };
}

/// Offset-zero implementation of the generic backend contract. The generic
/// context owns this value and drops all source Metal state while the
/// nonowning selector executor is still available.
#[repr(C)]
struct MechanicalRenderContextImpl {
    metal: ManuallyDrop<RenderContextMetal>,
    execution: Objc2MetalExecution,
    completion_slot: Arc<Mutex<Option<MechanicalCompletionToken>>>,
    clock_origin: Instant,
}

impl Drop for MechanicalRenderContextImpl {
    fn drop(&mut self) {
        // The host execution registry remains live while the complete
        // translated Metal class releases its derived state and then its
        // RenderContextHelperImpl/RenderContextImpl base. Adapter fields drop
        // only after this source owner is gone.
        unsafe {
            ManuallyDrop::drop(&mut self.metal);
        }
    }
}

// `RenderContextImplOwner` stores the concrete implementation behind the
// source virtual base. The translated Metal owner embeds that base at offset
// zero, so native integration projects the already-pinned owner back through
// the source pointer rather than reaching into the generic owner wrapper.
unsafe fn metal_impl_mut(context: &mut RenderContext) -> &mut MechanicalRenderContextImpl {
    unsafe { &mut *context.impl_ptr().cast::<MechanicalRenderContextImpl>() }
}

unsafe fn metal_impl_ref(context: &RenderContext) -> &MechanicalRenderContextImpl {
    unsafe { &*context.impl_ptr().cast::<MechanicalRenderContextImpl>() }
}

impl MechanicalRenderContextImpl {
    fn replace_command_queue(
        &mut self,
        queue: Option<Retained<ProtocolObject<dyn MTLCommandQueue>>>,
    ) {
        let queue = queue.map(|queue| self.execution.insert_command_queue(queue));
        self.metal.set_command_queue(&mut self.execution, queue);
        if let Some(queue) = queue {
            self.execution.retire_handle(queue);
        }
    }

    fn retained_command_queue(&self) -> Option<Retained<ProtocolObject<dyn MTLCommandQueue>>> {
        let native = self.metal.m_commandQueue.as_ref()?.native_object()?;
        let native = unsafe { Retained::retain(core::ptr::from_ref(native).cast_mut()) }?;
        Some(unsafe { Retained::cast_unchecked::<ProtocolObject<dyn MTLCommandQueue>>(native) })
    }

    fn resize_uniform_ring(&mut self, name: &'static str, capacity: usize) {
        self.metal
            .make_uniform_buffer_ring(&mut self.execution, name, capacity);
    }

    fn resize_storage_ring(&mut self, name: &'static str, capacity: usize) {
        self.metal
            .make_storage_buffer_ring(&mut self.execution, name, capacity);
    }

    fn resize_vertex_ring(&mut self, name: &'static str, capacity: usize) {
        self.metal
            .make_vertex_buffer_ring(&mut self.execution, name, capacity);
    }

    fn map_ring(&mut self, name: &'static str, size: usize) -> *mut c_void {
        // The translated helper owns the nine source rings in declaration
        // order; this adapter only forwards the virtual map call.
        self.metal.map_buffer_ring(name, size).cast()
    }

    fn unmap_ring(&mut self, name: &'static str, size: usize) {
        let _ = size;
        self.metal.unmap_buffer_ring(name);
    }

    unsafe fn command_handle(command_buffer: *mut c_void) -> Option<Handle> {
        NonNull::new(command_buffer.cast::<Handle>()).map(|pointer| unsafe { *pointer.as_ptr() })
    }
}

impl RenderContextImplContract for MechanicalRenderContextImpl {
    fn renderContextImpl(&self) -> &RenderContextImpl {
        &self.metal.base.base
    }

    fn renderContextImplMut(&mut self) -> &mut RenderContextImpl {
        &mut self.metal.base.base
    }

    fn platformDecodeImageTexture(&mut self, _encodedBytes: &[u8]) -> rcp<Texture> {
        // The pinned Metal implementation leaves this platform hook null;
        // RenderContext::decodeImage owns the Bitmap decoder fallback and its
        // source mip-level policy.
        rcp::new()
    }

    fn makeRenderBuffer(
        &mut self,
        bufferType: RenderBufferType,
        bufferFlags: RenderBufferFlags,
        sizeInBytes: usize,
    ) -> rcp<RenderBuffer> {
        let mapped_once = bufferFlags == RenderBufferFlags::mappedOnceAtInitialization;
        let owner = self.metal.make_render_buffer(
            &mut self.execution,
            bufferType,
            bufferFlags,
            sizeInBytes,
            mapped_once,
        );
        unsafe { rcp::from_ptr(Box::into_raw(Box::new(owner)).cast::<RenderBuffer>()) }
    }

    fn makeImageTexture(
        &mut self,
        width: u32,
        height: u32,
        mipLevelCount: u32,
        format: GPUTextureFormat,
        imageData: &[u8],
        blockWidth: u8,
        blockHeight: u8,
        srgb: bool,
        generateRemainingMips: bool,
    ) -> rcp<Texture> {
        let source_format = match format {
            GPUTextureFormat::rgba32 => mechanical_metal::TextureFormat::rgba32,
            GPUTextureFormat::bc7 => mechanical_metal::TextureFormat::bc7,
            GPUTextureFormat::astc => mechanical_metal::TextureFormat::astc,
            GPUTextureFormat::etc2 => mechanical_metal::TextureFormat::etc2,
            GPUTextureFormat::bc1 => mechanical_metal::TextureFormat::bc1,
            GPUTextureFormat::bc2 => mechanical_metal::TextureFormat::bc2,
            GPUTextureFormat::bc3 => mechanical_metal::TextureFormat::bc3,
        };
        let Some(native) = self.metal.make_image_texture(
            &mut self.execution,
            width,
            height,
            mipLevelCount,
            Arc::from(imageData),
            source_format,
            blockWidth,
            blockHeight,
            srgb,
            generateRemainingMips,
        ) else {
            return rcp::new();
        };
        unsafe { rcp::from_ptr(Box::into_raw(Box::new(native)).cast::<Texture>()) }
    }

    #[cfg(feature = "native-ore-metal-experimental")]
    fn makeRenderCanvas(&mut self, width: u32, height: u32) -> rcp<RenderCanvas> {
        let Some((texture_metal, target_metal, texture_descriptor)) = self
            .metal
            .make_render_canvas(&mut self.execution, width, height)
        else {
            return rcp::new();
        };
        let texture =
            unsafe { rcp::from_ptr(Box::into_raw(Box::new(texture_metal)).cast::<Texture>()) };
        let image = make_rcp(|| unsafe { RiveRenderImage::new(texture) });

        let target_owner = MechanicalRenderTargetOwner::new(
            target_metal,
            None,
            width,
            height,
            &mut self.execution,
        );
        let target = unsafe { rcp::from_ptr(Box::into_raw(target_owner).cast::<RenderTarget>()) };
        let canvas = make_rcp(|| unsafe { RenderCanvas::new(image, target) });
        // The canonical source descriptor remains live through the complete
        // outer RenderCanvas construction, then its original +1 is released.
        self.execution.owner_event(
            "RC-TD-CANVAS",
            crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::OwnerEventPhase::LastUse,
            texture_descriptor,
        );
        self.execution.retire_handle(texture_descriptor);
        self.execution.owner_event(
            "RC-TD-CANVAS",
            crate::mechanical_port::source::renderer::src::metal::render_context_metal_impl_mm::source_execution::OwnerEventPhase::Release,
            texture_descriptor,
        );
        canvas
    }

    #[cfg(feature = "native-ore-metal-experimental")]
    fn makeOreContext(&mut self) -> Option<Box<OreContext>> {
        // A queued nil transition is a failed-closed source boundary. Do not
        // hand an invalid queue handle to the translated HostExecution
        // callback or construct ORE without its required queue.
        let handle = self.metal.make_ore_context(&mut self.execution)?;
        let owner = self.execution.take_ore_context_owner(handle)?;
        let context = owner.downcast::<OreContextMetal>().ok()?;
        Some(Box::new(OreContext::Metal(context)))
    }

    fn resizeFlushUniformBuffer(&mut self, sizeInBytes: usize) {
        self.resize_uniform_ring(FLUSH_UNIFORM_RING, sizeInBytes);
    }

    fn resizePathBuffer(&mut self, sizeInBytes: usize, _: StorageBufferStructure) {
        self.resize_storage_ring(PATH_RING, sizeInBytes);
    }

    fn resizePaintBuffer(&mut self, sizeInBytes: usize, _: StorageBufferStructure) {
        self.resize_storage_ring(PAINT_RING, sizeInBytes);
    }

    fn resizePaintAuxBuffer(&mut self, sizeInBytes: usize, _: StorageBufferStructure) {
        self.resize_storage_ring(PAINT_AUX_RING, sizeInBytes);
    }

    fn resizeContourBuffer(&mut self, sizeInBytes: usize, _: StorageBufferStructure) {
        self.resize_storage_ring(CONTOUR_RING, sizeInBytes);
    }

    fn resizeGradSpanBuffer(&mut self, sizeInBytes: usize) {
        self.resize_vertex_ring(GRAD_SPAN_RING, sizeInBytes);
    }

    fn resizeTessVertexSpanBuffer(&mut self, sizeInBytes: usize) {
        self.resize_vertex_ring(TESS_VERTEX_SPAN_RING, sizeInBytes);
    }

    fn resizeTriangleVertexBuffer(&mut self, sizeInBytes: usize) {
        self.resize_vertex_ring(TRIANGLE_RING, sizeInBytes);
    }

    fn resizeImageDrawInstanceBuffer(&mut self, sizeInBytes: usize) {
        self.resize_vertex_ring(IMAGE_DRAW_INSTANCE_RING, sizeInBytes);
    }

    fn prepareToFlush(&mut self, _: u64, _: u64) {
        self.metal.prepare_to_flush();
    }

    fn mapFlushUniformBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(FLUSH_UNIFORM_RING, size)
    }

    fn mapPathBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(PATH_RING, size)
    }

    fn mapPaintBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(PAINT_RING, size)
    }

    fn mapPaintAuxBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(PAINT_AUX_RING, size)
    }

    fn mapContourBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(CONTOUR_RING, size)
    }

    fn mapGradSpanBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(GRAD_SPAN_RING, size)
    }

    fn mapTessVertexSpanBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(TESS_VERTEX_SPAN_RING, size)
    }

    fn mapTriangleVertexBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(TRIANGLE_RING, size)
    }

    fn mapImageDrawInstanceBuffer(&mut self, size: usize) -> *mut c_void {
        self.map_ring(IMAGE_DRAW_INSTANCE_RING, size)
    }

    fn unmapFlushUniformBuffer(&mut self, size: usize) {
        self.unmap_ring(FLUSH_UNIFORM_RING, size);
    }

    fn unmapPathBuffer(&mut self, size: usize) {
        self.unmap_ring(PATH_RING, size);
    }

    fn unmapPaintBuffer(&mut self, size: usize) {
        self.unmap_ring(PAINT_RING, size);
    }

    fn unmapPaintAuxBuffer(&mut self, size: usize) {
        self.unmap_ring(PAINT_AUX_RING, size);
    }

    fn unmapContourBuffer(&mut self, size: usize) {
        self.unmap_ring(CONTOUR_RING, size);
    }

    fn unmapGradSpanBuffer(&mut self, size: usize) {
        self.unmap_ring(GRAD_SPAN_RING, size);
    }

    fn unmapTessVertexSpanBuffer(&mut self, size: usize) {
        self.unmap_ring(TESS_VERTEX_SPAN_RING, size);
    }

    fn unmapTriangleVertexBuffer(&mut self, size: usize) {
        self.unmap_ring(TRIANGLE_RING, size);
    }

    fn unmapImageDrawInstanceBuffer(&mut self, size: usize) {
        self.unmap_ring(IMAGE_DRAW_INSTANCE_RING, size);
    }

    fn resizeGradientTexture(&mut self, width: u32, height: u32) {
        self.metal
            .resize_gradient(&mut self.execution, width, height);
    }

    fn resizeTessellationTexture(&mut self, width: u32, height: u32) {
        self.metal
            .resize_tessellation(&mut self.execution, width, height);
    }

    fn resizeFeatherAtlasTexture(&mut self, width: u32, height: u32) {
        self.metal
            .resize_feather(&mut self.execution, width, height);
    }

    unsafe fn flush(&mut self, descriptor: &FlushDescriptor) {
        let Some(target) = descriptor.renderTarget else {
            return;
        };
        let Some(command) = descriptor
            .externalCommandBuffer
            .and_then(|pointer| unsafe { Self::command_handle(pointer.as_ptr()) })
        else {
            return;
        };
        let target = unsafe { &mut *target.as_ptr().cast::<MechanicalRenderTargetOwner>() };
        unsafe {
            self.metal
                .flush(&mut self.execution, descriptor, &mut target.metal, command);
        }
    }

    unsafe fn postFlush(&mut self, resources: &FlushResources) {
        if let Some(command) = unsafe { Self::command_handle(resources.externalCommandBuffer) } {
            // Install the product completion continuation as part of the
            // source postFlush callback.  It runs only after that callback
            // releases the raw ring-lock pointer; a separately registered
            // command-buffer handler would have no FIFO ordering guarantee.
            let completion = self
                .completion_slot
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .take()
                .map(|completion| {
                    Arc::new(move |result| completion.mark_complete(result))
                        as Arc<dyn Fn(Result<(), String>) + Send + Sync + 'static>
                });
            unsafe {
                self.metal
                    .post_flush(&mut self.execution, command, completion)
            };
            #[cfg(test)]
            if take_panic_after_post_flush() {
                panic!("forced unwind after source postFlush callback installation");
            }
        }
    }

    fn makeCommandBuffer(&mut self) -> *mut c_void {
        self.metal
            .make_command_buffer(&mut self.execution)
            .map_or(core::ptr::null_mut(), |handle| {
                Box::into_raw(Box::new(handle)).cast()
            })
    }

    unsafe fn commitCommandBuffer(&mut self, commandBuffer: *mut c_void) {
        let Some(commandBuffer) = NonNull::new(commandBuffer.cast::<Handle>()) else {
            return;
        };
        let handle = unsafe { *Box::from_raw(commandBuffer.as_ptr()) };
        self.metal
            .commit_command_buffer(&mut self.execution, Some(handle));
    }

    fn secondsNow(&self) -> f64 {
        self.clock_origin.elapsed().as_secs_f64()
    }
}

#[derive(Default)]
struct MechanicalCompletionState {
    complete: Mutex<bool>,
    error: Mutex<Option<String>>,
    wake: Condvar,
}

/// CPU-side wait token for the exact source command buffer. The token owns no
/// Metal command object: `commitCommandBuffer` still consumes the retained
/// command-buffer owner, while the source `postFlush` completion block owns
/// only this state until it has released the ring-lock pointer and observed
/// Metal's terminal result.
#[derive(Clone, Default)]
pub(super) struct MechanicalCompletionToken {
    state: Arc<MechanicalCompletionState>,
}

impl MechanicalCompletionToken {
    fn mark_complete(&self, result: Result<(), String>) {
        if let Err(error) = result {
            *self
                .state
                .error
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()) = Some(error);
        }
        let mut complete = self
            .state
            .complete
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        *complete = true;
        self.state.wake.notify_all();
    }

    pub(super) fn wait(&self) -> Result<(), RendererError> {
        let mut complete = self
            .state
            .complete
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        while !*complete {
            complete = self
                .state
                .wake
                .wait(complete)
                .unwrap_or_else(|poison| poison.into_inner());
        }
        if let Some(error) = self
            .state
            .error
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
            .clone()
        {
            return Err(RendererError::NativeMetal(error));
        }
        Ok(())
    }
}

/// A source frame has already committed its render command before the
/// presentation command is allocated.  Every fallible operation after that
/// point must keep the context/ring completion alive until the committed work
/// has retired; otherwise an error or unwind can drop the frame while the
/// post-flush callback still owns a pointer into its pinned context.
struct CommittedFrameWaitGuard {
    completion: Option<MechanicalCompletionToken>,
    command: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
}

impl CommittedFrameWaitGuard {
    fn new(completion: MechanicalCompletionToken) -> Self {
        Self {
            completion: Some(completion),
            command: None,
        }
    }

    fn with_command(
        completion: MechanicalCompletionToken,
        command: Retained<ProtocolObject<dyn MTLCommandBuffer>>,
    ) -> Self {
        Self {
            completion: Some(completion),
            command: Some(command),
        }
    }

    fn disarm(&mut self) {
        self.completion = None;
        self.command = None;
    }
}

impl Drop for CommittedFrameWaitGuard {
    fn drop(&mut self) {
        // Metal documents `waitUntilCompleted` as waiting for the command and
        // all completion handlers. That is the terminal lifetime seam for the
        // raw SourceMutex pointer; a separately published token alone cannot
        // prove all handlers have retired.
        if let Some(command) = self.command.take() {
            command.waitUntilCompleted();
        }
        if let Some(completion) = self.completion.take() {
            let _ = completion.wait();
        }
    }
}

/// Source completion is the lifetime authority for a committed frame. A
/// presentation failure must never hide a source/ring failure, including when
/// both command buffers report errors.
fn finish_present_result(
    source: Result<(), RendererError>,
    presentation: Result<(), RendererError>,
) -> Result<(), RendererError> {
    match (source, presentation) {
        (Err(source), _) => Err(source),
        (Ok(()), Err(presentation)) => Err(presentation),
        (Ok(()), Ok(())) => Ok(()),
    }
}

#[cfg(test)]
mod committed_frame_guard_tests {
    use super::{
        finish_present_result, inject_panic_after_post_flush_once, CommittedFrameWaitGuard,
        MechanicalCompletionToken,
    };
    use crate::native_metal::NativeMetalFactory;
    use crate::RendererError;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;
    use std::thread;
    use std::time::{Duration, Instant};

    #[test]
    fn committed_guard_waits_for_source_completion_on_unwind_path() {
        let completion = MechanicalCompletionToken::default();
        let signal = completion.clone();
        let worker = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            signal.mark_complete(Ok(()));
        });
        let started = Instant::now();
        {
            let _guard = CommittedFrameWaitGuard::new(completion);
        }
        assert!(started.elapsed() >= Duration::from_millis(10));
        worker.join().unwrap();
    }

    #[test]
    fn product_completion_before_source_unlock_does_not_release_guard() {
        let completion = MechanicalCompletionToken::default();
        let source_unlocked = Arc::new(AtomicBool::new(false));

        // This models the product callback being invoked first. It cannot
        // publish the token: the raw source callback owns publication only
        // after it has released the ring-lock pointer.
        let product_callback = || {
            assert!(!source_unlocked.load(Ordering::Acquire));
        };
        product_callback();

        let source_completion = completion.clone();
        let source_unlocked_for_worker = Arc::clone(&source_unlocked);
        let worker = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            source_unlocked_for_worker.store(true, Ordering::Release);
            source_completion.mark_complete(Ok(()));
        });

        let started = Instant::now();
        {
            let _guard = CommittedFrameWaitGuard::new(completion);
        }
        assert!(source_unlocked.load(Ordering::Acquire));
        assert!(started.elapsed() >= Duration::from_millis(10));
        worker.join().unwrap();
    }

    #[test]
    fn presentation_result_matrix_preserves_source_first_precedence() {
        let source_error = || Err(RendererError::NativeMetal("source failed".into()));
        let presentation_error = || Err(RendererError::NativeMetal("presentation failed".into()));

        assert!(finish_present_result(Ok(()), Ok(())).is_ok());
        assert!(matches!(
            finish_present_result(Ok(()), presentation_error()),
            Err(RendererError::NativeMetal(message)) if message == "presentation failed"
        ));
        assert!(matches!(
            finish_present_result(source_error(), Ok(())),
            Err(RendererError::NativeMetal(message)) if message == "source failed"
        ));
        assert!(matches!(
            finish_present_result(source_error(), presentation_error()),
            Err(RendererError::NativeMetal(message)) if message == "source failed"
        ));
    }

    #[test]
    fn committed_guard_waits_during_unwind_before_context_can_drop() {
        let completion = MechanicalCompletionToken::default();
        let signal = completion.clone();
        let worker = thread::spawn(move || {
            thread::sleep(Duration::from_millis(20));
            signal.mark_complete(Ok(()));
        });
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _guard = CommittedFrameWaitGuard::new(completion);
            panic!("forced presentation allocation unwind");
        }));
        assert!(panic_result.is_err());
        worker.join().unwrap();
    }

    #[test]
    fn real_source_submission_unwind_waits_for_raw_ring_callback() {
        let factory = NativeMetalFactory::new(4, 4).expect("native Metal test factory");
        let frame = factory.begin_frame(0).expect("source frame");
        inject_panic_after_post_flush_once();
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = frame.finish_for_benchmark();
        }));
        assert!(panic_result.is_err(), "actual postFlush unwind was not injected");

        // Cycle the three exact source slots plus one reuse. If recovery had
        // returned before the raw callback unlocked its member pointer, this
        // fourth frame would block or touch a destroyed/still-held slot.
        for _ in 0..4 {
            let frame = factory.begin_frame(0).expect("frame after recovered unwind");
            frame
                .finish_for_benchmark()
                .expect("source ring remained reusable after unwind");
        }
    }
}

/// Frame-capable product owner for the mechanical generic/Metal path.
pub(super) struct MechanicalRenderContext {
    render_context: Pin<Box<RenderContext>>,
    target: Option<Box<MechanicalRenderTargetOwner>>,
    width: u32,
    height: u32,
    mode: RenderMode,
    completion_slot: Arc<Mutex<Option<MechanicalCompletionToken>>>,
    active_frame: bool,
    frame_number: u64,
    frame_queue: Option<Retained<ProtocolObject<dyn MTLCommandQueue>>>,
    resource_domain: RenderResourceDomain,
}

impl MechanicalRenderContext {
    pub(super) fn new(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
        target_texture: Retained<ProtocolObject<dyn MTLTexture>>,
        width: u32,
        height: u32,
        mode: RenderMode,
        options: NativeMetalContextOptions,
    ) -> Result<Self, RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::NativeMetal(
                "mechanical Metal target dimensions must be nonzero".into(),
            ));
        }
        let mut context = Self::new_source(device, width, height, mode, options)?;
        context.install_queue_and_target(queue, target_texture, width, height)?;
        Ok(context)
    }

    /// Roots the source RenderContextMetal before queue/target allocation.
    /// Product admission consumes its capability snapshot before calling
    /// `install_queue_and_target`.
    pub(super) fn new_source(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        width: u32,
        height: u32,
        mode: RenderMode,
        options: NativeMetalContextOptions,
    ) -> Result<Self, RendererError> {
        let mut execution = Objc2MetalExecution::new(device, Box::new(MechanicalMetalHost));
        let device_handle = execution.device_handle();
        let completion_slot = Arc::new(Mutex::new(None));
        let metal = RenderContextMetal::new(
            &mut execution,
            device_handle,
            source_context_options(options),
        );
        let implementation = Box::new(MechanicalRenderContextImpl {
            metal: ManuallyDrop::new(metal),
            execution,
            completion_slot: Arc::clone(&completion_slot),
            clock_origin: Instant::now(),
        });
        #[cfg_attr(not(feature = "rive-decoders"), allow(unused_mut))]
        let mut render_context = RenderContext::from_impl(implementation);
        #[cfg(feature = "rive-decoders")]
        unsafe {
            Pin::get_unchecked_mut(render_context.as_mut())
                .installBitmapDecoder(Box::new(MechanicalBitmapDecoder));
        }
        Ok(Self {
            render_context,
            target: None,
            width,
            height,
            mode,
            completion_slot,
            active_frame: false,
            frame_number: 0,
            frame_queue: None,
            resource_domain: RenderResourceDomain::new(),
        })
    }

    pub(super) fn install_queue_and_target(
        &mut self,
        queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
        target_texture: Retained<ProtocolObject<dyn MTLTexture>>,
        width: u32,
        height: u32,
    ) -> Result<(), RendererError> {
        let format = target_pixel_format(target_texture.pixelFormat()).ok_or_else(|| {
            RendererError::NativeMetal("unsupported mechanical Metal target format".into())
        })?;
        let implementation = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        let implementation_metal = unsafe { metal_impl_mut(implementation) };
        let queue_handle = implementation_metal.execution.insert_command_queue(queue);
        implementation_metal
            .metal
            .set_command_queue(&mut implementation_metal.execution, Some(queue_handle));
        implementation_metal.execution.retire_handle(queue_handle);
        let target_texture_handle = implementation_metal
            .execution
            .insert_texture(target_texture);
        let target_metal = implementation_metal.metal.make_render_target(
            &mut implementation_metal.execution,
            format,
            width,
            height,
        );
        self.target = Some(MechanicalRenderTargetOwner::new(
            target_metal,
            Some(target_texture_handle),
            width,
            height,
            &mut implementation_metal.execution,
        ));
        Ok(())
    }

    /// Exact nullable `setCommandQueue` transition. The pinned setter assigns
    /// immediately, including between beginFrame and the first flush.
    pub(super) fn set_command_queue(
        &mut self,
        queue: Option<Retained<ProtocolObject<dyn MTLCommandQueue>>>,
    ) {
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        unsafe { metal_impl_mut(context) }.replace_command_queue(queue.clone());
        if self.active_frame {
            self.frame_queue = queue;
        }
    }

    pub(super) fn render_context_mut(&mut self) -> Pin<&mut RenderContext> {
        self.render_context.as_mut()
    }

    /// Selected native source-factory seam. NativeMetalFrame's later wholesale
    /// routing pass consumes this contract so Gradient/Path/Paint creation is
    /// inherited from this exact RenderContext -> RiveRenderFactory -> Factory
    /// owner rather than a detached helper object.
    pub(super) fn factory_mut(&mut self) -> &mut dyn FactoryContract {
        // SAFETY: projecting to the non-pinned source Factory base does not
        // move RenderContext or any self-referential logical-flush allocation.
        unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) }
    }

    pub(super) fn make_render_buffer_handle(
        &mut self,
        buffer_type: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType,
        flags: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags,
        size: usize,
    ) -> Option<RiveRenderBufferHandle> {
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        Some(context.makeRenderBufferHandle(buffer_type, flags, size)?)
    }

    pub(super) fn make_linear_gradient_handle(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[u32],
        stops: &[f32],
    ) -> Option<GradientShader> {
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        context
            .riveRenderFactoryMut()
            .makeLinearGradientHandle(sx, sy, ex, ey, colors, stops)
    }

    pub(super) fn make_radial_gradient_handle(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[u32],
        stops: &[f32],
    ) -> Option<GradientShader> {
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        context
            .riveRenderFactoryMut()
            .makeRadialGradientHandle(cx, cy, radius, colors, stops)
    }

    pub(super) fn make_render_path_handle(
        &mut self,
        path: &mut nuxie_render_api::RawPath,
        fill_rule: nuxie_render_api::FillRule,
    ) -> Option<RiveRenderPathHandle> {
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        context
            .riveRenderFactoryMut()
            .makeRenderPathHandle(path, fill_rule)
    }

    pub(super) fn make_empty_render_path_handle(&mut self) -> Option<RiveRenderPathHandle> {
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        context.riveRenderFactoryMut().makeEmptyRenderPathHandle()
    }

    pub(super) fn make_render_paint_handle(&mut self) -> Option<RiveRenderPaintHandle> {
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        context.riveRenderFactoryMut().makeRenderPaintHandle()
    }

    pub(super) fn decode_image_handle(&mut self, bytes: &[u8]) -> Option<RiveRenderImageHandle> {
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        context.decodeImageHandle(bytes)
    }

    pub(super) fn adopt_image_handle(
        &mut self,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
        width: u32,
        height: u32,
    ) -> Option<RiveRenderImageHandle> {
        let implementation = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        let implementation_metal = unsafe { metal_impl_mut(implementation) };
        let handle = implementation_metal
            .execution
            .insert_texture(texture.clone());
        let Some(native) = implementation_metal.metal.adopt_image_texture(
            &mut implementation_metal.execution,
            Some(handle),
            width,
            height,
        ) else {
            unsafe { metal_impl_mut(implementation) }
                .execution
                .retire_handle(handle);
            return None;
        };
        let texture = unsafe { rcp::from_ptr(Box::into_raw(Box::new(native)).cast::<Texture>()) };
        let image = make_rcp(|| unsafe { RiveRenderImage::new(texture) });
        RiveRenderImageHandle::from_exact(image)
    }

    pub(super) fn begin_frame(&mut self, clear_color: u32) -> Result<(), RendererError> {
        if self.active_frame {
            return Err(RendererError::NativeMetal(
                "mechanical RenderContext already has an active frame".into(),
            ));
        }
        let mut descriptor = FrameDescriptor {
            renderTargetWidth: self.width,
            renderTargetHeight: self.height,
            clearColor: clear_color,
            ..FrameDescriptor::default()
        };
        match self.mode {
            RenderMode::RasterOrdering => {}
            RenderMode::Msaa => descriptor.msaaSampleCount = 4,
            RenderMode::ClockwiseAtomic => {
                descriptor.disableRasterOrdering = true;
                descriptor.clockwiseFillOverride = true;
            }
        }
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        unsafe { metal_impl_mut(context) }
            .execution
            .reset_execution_inventory();
        context.beginFrameExecutable(&descriptor);
        let context_metal = unsafe { metal_impl_mut(context) };
        self.frame_queue = context_metal.retained_command_queue();
        self.frame_number = self.frame_number.wrapping_add(1);
        self.active_frame = true;
        Ok(())
    }

    pub(super) fn abandon_frame(&mut self) {
        // Exact source inverse of beginFrameExecutable: release pending draw
        // owners, rewind frame arenas, and clear the debug frame guard.
        unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) }.abortFrameExecutable();
        self.active_frame = false;
        self.frame_queue = None;
    }

    pub(super) fn retained_target_texture(
        &self,
    ) -> Option<Retained<ProtocolObject<dyn MTLTexture>>> {
        self.target.as_ref()?.retained_target_texture()
    }

    pub(super) fn target_matches(
        &self,
        texture: &ProtocolObject<dyn MTLTexture>,
        width: u32,
        height: u32,
    ) -> bool {
        self.width == width
            && self.height == height
            && self
                .target
                .as_ref()
                .and_then(|target| target.retained_target_texture())
                .is_some_and(|current| Retained::as_ptr(&current) == std::ptr::from_ref(texture))
    }

    pub(super) fn dimensions(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub(super) fn mode(&self) -> RenderMode {
        self.mode
    }

    pub(super) fn source_capabilities(&self) -> MetalCapabilitySelection {
        let context = self.render_context.as_ref().get_ref();
        let (platform, metal) = unsafe { metal_impl_ref(context) }
            .metal
            .source_capability_snapshot();
        let atomic_barrier_type = metal.atomicBarrierType;
        MetalCapabilitySelection {
            max_texture_size: platform.maxTextureSize,
            supports_raster_ordering: platform.supportsRasterOrderingMode,
            supports_atomic_mode: platform.supportsAtomicMode,
            path_id_granularity: u32::from(platform.pathIDGranularity),
            supports_texture_compression_etc2: platform.supportsTextureCompressionETC2,
            supports_texture_compression_astc: platform.supportsTextureCompressionASTC,
            supports_texture_compression_bc: platform.supportsTextureCompressionBC,
            atomic_barrier_type,
        }
    }

    pub(super) fn set_mode(&mut self, mode: RenderMode) {
        self.mode = mode;
    }

    pub(super) fn is_active_frame(&self) -> bool {
        self.active_frame
    }

    pub(super) fn current_frame_number(&self) -> u64 {
        self.frame_number
    }

    pub(super) fn resource_domain(&self) -> RenderResourceDomain {
        self.resource_domain.clone()
    }

    #[cfg(feature = "native-ore-metal-experimental")]
    pub(super) fn with_ore_context<R>(
        &mut self,
        callback: impl FnOnce(&mut OreContextMetal) -> R,
    ) -> Option<R> {
        if self.active_frame {
            return None;
        }
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        let context_metal = unsafe { metal_impl_mut(context) };
        if context_metal.metal.command_queue().is_none() {
            return None;
        }
        let ore = context.oreExecutable();
        if ore.is_null() {
            return None;
        }
        match unsafe { &mut *ore } {
            OreContext::Metal(context) => Some(callback(context)),
            #[cfg(feature = "native-ore-vulkan-experimental")]
            OreContext::Vulkan(_) => None,
        }
    }

    pub(super) fn execution_inventory(&self) -> ActualMetalExecutionInventory {
        unsafe { metal_impl_ref(self.render_context.as_ref().get_ref()) }
            .execution
            .snapshot_execution_inventory()
    }

    /// Replace only the target generation. The pinned RenderContext, source
    /// factory, rings, caches, and command queue remain installed for the
    /// lifetime of this product context.
    pub(super) fn replace_target(
        &mut self,
        texture: Retained<ProtocolObject<dyn MTLTexture>>,
        width: u32,
        height: u32,
    ) -> Result<(), RendererError> {
        if self.active_frame {
            return Err(RendererError::NativeMetal(
                "cannot replace a mechanical target during an active frame".into(),
            ));
        }
        let format = target_pixel_format(texture.pixelFormat()).ok_or_else(|| {
            RendererError::NativeMetal("unsupported mechanical Metal target format".into())
        })?;
        let implementation = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        let implementation_metal = unsafe { metal_impl_mut(implementation) };
        let texture_handle = implementation_metal
            .execution
            .insert_texture(texture.clone());
        let target_metal = implementation_metal.metal.make_render_target(
            &mut implementation_metal.execution,
            format,
            width,
            height,
        );
        self.target = Some(MechanicalRenderTargetOwner::new(
            target_metal,
            Some(texture_handle),
            width,
            height,
            &mut implementation_metal.execution,
        ));
        self.width = width;
        self.height = height;
        Ok(())
    }

    pub(super) fn finish(
        &mut self,
        current_frame_number: u64,
        safe_frame_number: u64,
    ) -> Result<MechanicalCompletionToken, RendererError> {
        self.finish_impl(current_frame_number, safe_frame_number)
    }

    pub(super) fn finish_present(
        &mut self,
        current_frame_number: u64,
        safe_frame_number: u64,
        drawable: &ProtocolObject<dyn objc2_metal::MTLDrawable>,
    ) -> Result<MechanicalCompletionToken, RendererError> {
        let queue = self.frame_queue.clone().ok_or_else(|| {
            RendererError::NativeMetal("mechanical frame lost its queue snapshot".into())
        })?;
        let completion = self.finish_impl(current_frame_number, safe_frame_number)?;
        let mut committed_guard = CommittedFrameWaitGuard::new(completion.clone());
        let presentation_result = match make_command_buffer_on_queue(&queue) {
            Ok(command_buffer) => {
                command_buffer.presentDrawable(drawable);
                let presentation = NativeMetalSubmissionCompletion::commit(&command_buffer);
                presentation.wait()
            }
            Err(error) => Err(error),
        };
        // Always observe the source completion before disarming the guard.
        // If both submissions fail, the source error is authoritative because
        // it covers the committed callback/ring lifetime that this method
        // owns; presentation failure is only returned when source work was
        // successful.
        let source_result = completion.wait();
        committed_guard.disarm();
        finish_present_result(source_result, presentation_result).map(|()| completion)
    }

    fn finish_impl(
        &mut self,
        current_frame_number: u64,
        safe_frame_number: u64,
    ) -> Result<MechanicalCompletionToken, RendererError> {
        if !self.active_frame {
            return Err(RendererError::NativeMetal(
                "mechanical RenderContext has no active frame".into(),
            ));
        }
        let Some(target) = self.target.as_mut() else {
            return Err(RendererError::NativeMetal(
                "mechanical RenderContext has no installed target".into(),
            ));
        };
        let render_target = core::ptr::from_mut(&mut *target.metal.base);
        let context = unsafe { Pin::get_unchecked_mut(self.render_context.as_mut()) };
        let command_buffer =
            RenderContextImplContract::makeCommandBuffer(unsafe { metal_impl_mut(context) });
        if command_buffer.is_null() {
            return Err(RendererError::NativeMetal(
                "failed to allocate mechanical Metal command buffer".into(),
            ));
        }
        let command_handle = unsafe { *command_buffer.cast::<Handle>() };
        let command_wait = unsafe { metal_impl_mut(context) }
            .execution
            .retained_command_buffer(command_handle)
            .expect("source command-buffer handle has a live native owner");
        let resources = FlushResources {
            renderTarget: render_target,
            externalCommandBuffer: command_buffer,
            currentFrameNumber: current_frame_number,
            safeFrameNumber: safe_frame_number,
        };
        let completion = MechanicalCompletionToken::default();
        *self
            .completion_slot
            .lock()
            .unwrap_or_else(|poison| poison.into_inner()) = Some(completion.clone());
        let flush_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| unsafe {
            context.flushExecutable(&resources)
        }));
        if let Err(payload) = flush_result {
            // `postFlush` takes the slot at the instant it arms the callback.
            // If the token is still present, the raw ring callback does not
            // exist and the Rust adapter must restore the selected source
            // ring synchronously. Otherwise commit the exact retained source
            // command and wait until that callback has released its pointer
            // before allowing the pinned context to unwind.
            let unarmed = self
                .completion_slot
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .take()
                .is_some();
            if unarmed {
                unsafe { metal_impl_mut(context) }
                    .metal
                    .abort_unarmed_flush_after_unwind();
                completion.mark_complete(Err(
                    "source flush unwound before completion callback installation".into(),
                ));
            }
            unsafe {
                RenderContextImplContract::commitCommandBuffer(
                    metal_impl_mut(context),
                    command_buffer,
                );
            }
            if !unarmed {
                command_wait.waitUntilCompleted();
            }
            // `flushExecutable` normally clears the source per-frame arenas
            // and debug frame bit after `postFlush`. A Rust unwind can leave
            // that authored tail unexecuted, so restore the same quiescent
            // owner state only after any armed raw callback has completed.
            context.abortFrameExecutable();
            self.active_frame = false;
            self.frame_queue = None;
            std::panic::resume_unwind(payload);
        }

        // The source callback is armed now. Establish the wait guard before
        // entering the commit bridge so a Rust unwind after native commit can
        // never release the pinned context ahead of the raw ring pointer.
        let mut source_guard =
            CommittedFrameWaitGuard::with_command(completion.clone(), command_wait);
        unsafe {
            RenderContextImplContract::commitCommandBuffer(metal_impl_mut(context), command_buffer);
        }
        source_guard.disarm();
        self.active_frame = false;
        self.frame_queue = None;
        Ok(completion)
    }
}
