//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/webgpu/render_context_webgpu_impl.hpp`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::webgpu_cpp_decl::{
    BackendType, BindGroup, BindGroupLayout, Buffer, Device, Queue, RenderPipeline,
    Sampler, ShaderModule, Texture as WagyuTexture, TextureFormat as WagyuTextureFormat,
    TextureUsage, TextureView,
};
use crate::mechanical_port::source::include::rive::refcnt_hpp::{rcp, RefCntTarget};
use crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    InterlockMode, PlatformFeatures, INTERLOCK_MODE_COUNT,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::RenderContextHelperImpl;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::RenderContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::RenderTarget;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use core::ffi::c_void;
use std::collections::BTreeMap;
use std::mem::ManuallyDrop;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_include_rive_renderer_webgpu_render_context_webgpu_impl.hpp");

pub(crate) unsafe fn dropArrayReverse<T, const N: usize>(array: &mut ManuallyDrop<[T; N]>) {
    let pointer = array.as_mut_ptr();
    for index in (0..N).rev() {
        unsafe { std::ptr::drop_in_place(pointer.add(index)) };
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct ContextOptions {
    pub(crate) compatibilityMode: bool,
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum PixelLocalStorageType {
    #[default]
    none,
    GL_EXT_shader_pixel_local_storage,
    VK_EXT_rasterization_order_attachment_access,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Capabilities {
    pub(crate) backendType: BackendType,
    pub(crate) polyfillVertexStorageBuffers: bool,
    pub(crate) VK_EXT_rasterization_order_attachment_access: bool,
    pub(crate) GL_EXT_shader_pixel_local_storage: bool,
    pub(crate) GL_EXT_shader_pixel_local_storage2: bool,
    pub(crate) plsType: PixelLocalStorageType,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            backendType: BackendType::Undefined,
            polyfillVertexStorageBuffers: false,
            VK_EXT_rasterization_order_attachment_access: false,
            GL_EXT_shader_pixel_local_storage: false,
            GL_EXT_shader_pixel_local_storage2: false,
            plsType: PixelLocalStorageType::none,
        }
    }
}

pub(crate) struct DrawPipelineLayout {
    pub(crate) m_perFlushBindingLayoutEntries: [super::webgpu_decl::WGPUBindGroupLayoutEntry;
        RenderContextWebGPUImpl::DRAW_BINDINGS_COUNT],
    pub(crate) m_bindGroupLayouts: ManuallyDrop<[BindGroupLayout; 4]>,
    pub(crate) m_pipelineLayout: ManuallyDrop<super::webgpu_cpp_decl::PipelineLayout>,
}

impl Drop for DrawPipelineLayout {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_pipelineLayout);
            dropArrayReverse(&mut self.m_bindGroupLayouts);
        }
    }
}

pub(crate) struct LoadStoreEXTPipeline {
    pub(crate) m_framebufferFormat: WagyuTextureFormat,
    pub(crate) m_bindGroupLayout: ManuallyDrop<BindGroupLayout>,
    pub(crate) m_renderPipeline: ManuallyDrop<RenderPipeline>,
}

impl Drop for LoadStoreEXTPipeline {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_renderPipeline);
            ManuallyDrop::drop(&mut self.m_bindGroupLayout);
        }
    }
}

pub(crate) struct ColorRampPipeline {
    pub(crate) m_bindGroupLayout: ManuallyDrop<BindGroupLayout>,
    pub(crate) m_renderPipeline: ManuallyDrop<RenderPipeline>,
}

pub(crate) struct BlitTextureAsDrawPipeline {
    pub(crate) m_perDrawBindGroupLayout: ManuallyDrop<BindGroupLayout>,
    pub(crate) m_renderPipeline: ManuallyDrop<RenderPipeline>,
}

impl Drop for BlitTextureAsDrawPipeline {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_renderPipeline);
            ManuallyDrop::drop(&mut self.m_perDrawBindGroupLayout);
        }
    }
}

impl Drop for ColorRampPipeline {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_renderPipeline);
            ManuallyDrop::drop(&mut self.m_bindGroupLayout);
        }
    }
}

pub(crate) struct TessellatePipeline {
    pub(crate) m_perFlushBindingsLayout: ManuallyDrop<BindGroupLayout>,
    pub(crate) m_renderPipeline: ManuallyDrop<RenderPipeline>,
}

impl Drop for TessellatePipeline {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_renderPipeline);
            ManuallyDrop::drop(&mut self.m_perFlushBindingsLayout);
        }
    }
}

pub(crate) struct FeatherAtlasPipeline {
    pub(crate) m_perFlushBindingsLayout: ManuallyDrop<BindGroupLayout>,
    pub(crate) m_fillPipeline: ManuallyDrop<RenderPipeline>,
    pub(crate) m_strokePipeline: ManuallyDrop<RenderPipeline>,
}

impl Drop for FeatherAtlasPipeline {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_strokePipeline);
            ManuallyDrop::drop(&mut self.m_fillPipeline);
            ManuallyDrop::drop(&mut self.m_perFlushBindingsLayout);
        }
    }
}

pub(crate) struct DrawPipeline {
    pub(crate) m_renderPipelines: ManuallyDrop<[RenderPipeline; 2]>,
}

impl Drop for DrawPipeline {
    fn drop(&mut self) {
        unsafe { dropArrayReverse(&mut self.m_renderPipelines) };
    }
}

#[repr(C)]
pub(crate) struct RenderContextWebGPUImpl {
    pub(crate) base: ManuallyDrop<RenderContextHelperImpl>,
    pub(crate) m_device: ManuallyDrop<Device>,
    pub(crate) m_queue: ManuallyDrop<Queue>,
    pub(crate) m_contextOptions: ContextOptions,
    pub(crate) m_capabilities: Capabilities,
    pub(crate) m_drawPipelineLayouts:
        ManuallyDrop<[Option<Box<DrawPipelineLayout>>; INTERLOCK_MODE_COUNT]>,
    pub(crate) m_loadStoreEXTPipelines: ManuallyDrop<BTreeMap<u32, LoadStoreEXTPipeline>>,
    pub(crate) m_loadStoreEXTVertexShader: ManuallyDrop<ShaderModule>,
    pub(crate) m_loadStoreEXTUniforms:
        ManuallyDrop<Option<Box<dyn super::render_context_webgpu_impl::BufferRingWebGPUApi>>>,
    pub(crate) m_blitTextureAsDrawPipeline:
        ManuallyDrop<Option<Box<BlitTextureAsDrawPipeline>>>,
    pub(crate) m_colorRampPipeline: ManuallyDrop<Option<Box<ColorRampPipeline>>>,
    pub(crate) m_gradientTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_gradientTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_tessellatePipeline: ManuallyDrop<Option<Box<TessellatePipeline>>>,
    pub(crate) m_tessSpanIndexBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_tessVertexTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_tessVertexTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_featherAtlasPipeline: ManuallyDrop<Option<Box<FeatherAtlasPipeline>>>,
    pub(crate) m_featherAtlasTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_featherAtlasTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_drawPipelines: ManuallyDrop<BTreeMap<u64, DrawPipeline>>,
    pub(crate) m_linearSampler: ManuallyDrop<Sampler>,
    pub(crate) m_imageSamplers: ManuallyDrop<[Sampler; ImageSampler::MAX_SAMPLER_PERMUTATIONS]>,
    pub(crate) m_samplerBindings: ManuallyDrop<BindGroup>,
    pub(crate) m_emptyBindingsLayout: ManuallyDrop<BindGroupLayout>,
    pub(crate) m_pathPatchVertexBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_pathPatchIndexBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_imageRectVertexBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_imageRectIndexBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_gaussianIntegralTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_gaussianIntegralTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_atomicPLSBackingBufferSize: u64,
    pub(crate) m_atomicPLSColorBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_atomicPLSClipBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_atomicPLSCoverageBuffer: ManuallyDrop<Buffer>,
    pub(crate) m_nullTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_nullTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_nullStorageBuffer: ManuallyDrop<Buffer>,
}

impl RenderContextWebGPUImpl {
    pub(crate) const COLOR_RAMP_BINDINGS_COUNT: usize = 1;
    pub(crate) const TESS_BINDINGS_COUNT: usize = 6;
    pub(crate) const FEATHER_ATLAS_BINDINGS_COUNT: usize = 7;
    pub(crate) const DRAW_BINDINGS_COUNT: usize = 10;

    pub(crate) fn device(&self) -> Device {
        (&*self.m_device).clone()
    }

    pub(crate) fn queue(&self) -> Queue {
        (&*self.m_queue).clone()
    }

    pub(crate) fn contextOptions(&self) -> &ContextOptions {
        &self.m_contextOptions
    }

    pub(crate) fn capabilities(&self) -> &Capabilities {
        &self.m_capabilities
    }

    pub(crate) fn makeCommandBuffer(&mut self) -> *mut c_void {
        super::render_context_webgpu_impl::makeCommandBuffer(self)
    }

    pub(crate) unsafe fn commitCommandBuffer(&mut self, commandBuffer: *mut c_void) {
        unsafe { super::render_context_webgpu_impl::commitCommandBuffer(self, commandBuffer) }
    }

    pub(crate) fn platformFeatures(&self) -> &PlatformFeatures {
        self.base.base.platformFeatures()
    }

    pub(crate) fn makeRenderTarget(
        &self,
        format: WagyuTextureFormat,
        width: u32,
        height: u32,
    ) -> rcp<RenderTargetWebGPU> {
        super::render_context_webgpu_impl::makeRenderTarget(self, format, width, height)
    }

    pub(crate) fn makeRenderBuffer(
        &self,
        bufferType: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferType,
        bufferFlags: crate::mechanical_port::source::include::rive::renderer_hpp::RenderBufferFlags,
        sizeInBytes: usize,
    ) -> rcp<crate::mechanical_port::source::include::rive::renderer_hpp::RenderBuffer> {
        super::render_context_webgpu_impl::makeRenderBuffer(
            self,
            bufferType,
            bufferFlags,
            sizeInBytes,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn makeImageTexture(
        &mut self,
        width: u32,
        height: u32,
        mipLevelCount: u32,
        format: crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat,
        imageData: &[u8],
        blockWidth: u8,
        blockHeight: u8,
        srgb: bool,
        generateRemainingMips: bool,
    ) -> rcp<Texture> {
        super::render_context_webgpu_impl::makeImageTexture(
            self,
            width,
            height,
            mipLevelCount,
            format,
            imageData,
            blockWidth,
            blockHeight,
            srgb,
            generateRemainingMips,
        )
    }

    pub(crate) fn makeImageTextureDefault(
        &mut self,
        width: u32,
        height: u32,
        mipLevelCount: u32,
        format: crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat,
        imageData: &[u8],
    ) -> rcp<Texture> {
        self.makeImageTexture(
            width,
            height,
            mipLevelCount,
            format,
            imageData,
            1,
            1,
            false,
            false,
        )
    }

    pub(crate) fn makeRenderCanvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> rcp<crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas>
    {
        super::render_context_webgpu_impl::makeRenderCanvas(self, width, height)
    }

    pub(crate) fn makeOreContext(&self) -> Option<Box<super::ore_context_wgpu_decl::ContextWGPU>> {
        super::render_context_webgpu_impl::makeOreContext(self)
    }
}

pub(crate) fn MakeContext(
    adapter: super::webgpu_cpp_decl::Adapter,
    device: Device,
    queue: Queue,
    options: ContextOptions,
) -> std::pin::Pin<Box<RenderContext>> {
    super::render_context_webgpu_impl::MakeContext(adapter, device, queue, options)
}

impl Drop for RenderContextWebGPUImpl {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_nullStorageBuffer);
            ManuallyDrop::drop(&mut self.m_nullTextureView);
            ManuallyDrop::drop(&mut self.m_nullTexture);
            ManuallyDrop::drop(&mut self.m_atomicPLSCoverageBuffer);
            ManuallyDrop::drop(&mut self.m_atomicPLSClipBuffer);
            ManuallyDrop::drop(&mut self.m_atomicPLSColorBuffer);
            ManuallyDrop::drop(&mut self.m_gaussianIntegralTextureView);
            ManuallyDrop::drop(&mut self.m_gaussianIntegralTexture);
            ManuallyDrop::drop(&mut self.m_imageRectIndexBuffer);
            ManuallyDrop::drop(&mut self.m_imageRectVertexBuffer);
            ManuallyDrop::drop(&mut self.m_pathPatchIndexBuffer);
            ManuallyDrop::drop(&mut self.m_pathPatchVertexBuffer);
            ManuallyDrop::drop(&mut self.m_emptyBindingsLayout);
            ManuallyDrop::drop(&mut self.m_samplerBindings);
            dropArrayReverse(&mut self.m_imageSamplers);
            ManuallyDrop::drop(&mut self.m_linearSampler);
            ManuallyDrop::drop(&mut self.m_drawPipelines);
            ManuallyDrop::drop(&mut self.m_featherAtlasTextureView);
            ManuallyDrop::drop(&mut self.m_featherAtlasTexture);
            ManuallyDrop::drop(&mut self.m_featherAtlasPipeline);
            ManuallyDrop::drop(&mut self.m_tessVertexTextureView);
            ManuallyDrop::drop(&mut self.m_tessVertexTexture);
            ManuallyDrop::drop(&mut self.m_tessSpanIndexBuffer);
            ManuallyDrop::drop(&mut self.m_tessellatePipeline);
            ManuallyDrop::drop(&mut self.m_gradientTextureView);
            ManuallyDrop::drop(&mut self.m_gradientTexture);
            ManuallyDrop::drop(&mut self.m_colorRampPipeline);
            ManuallyDrop::drop(&mut self.m_blitTextureAsDrawPipeline);
            ManuallyDrop::drop(&mut self.m_loadStoreEXTUniforms);
            ManuallyDrop::drop(&mut self.m_loadStoreEXTVertexShader);
            ManuallyDrop::drop(&mut self.m_loadStoreEXTPipelines);
            dropArrayReverse(&mut self.m_drawPipelineLayouts);
            ManuallyDrop::drop(&mut self.m_queue);
            ManuallyDrop::drop(&mut self.m_device);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

#[repr(C)]
pub(crate) struct RenderTargetWebGPU {
    pub(crate) base: ManuallyDrop<RenderTarget>,
    pub(crate) m_device: ManuallyDrop<Device>,
    pub(crate) m_framebufferFormat: WagyuTextureFormat,
    pub(crate) m_transientPLSUsage: TextureUsage,
    pub(crate) m_transientMSAAColorUsage: TextureUsage,
    pub(crate) m_transientMSAADepthStencilUsage: TextureUsage,
    pub(crate) m_targetTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_coverageTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_clipTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_scratchColorTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_msaaColorTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_msaaDepthStencilTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_dstColorTexture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_targetTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_coverageTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_clipTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_scratchColorTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_msaaColorTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_msaaDepthStencilTextureView: ManuallyDrop<TextureView>,
    pub(crate) m_dstColorTextureView: ManuallyDrop<TextureView>,
}

impl RenderTargetWebGPU {
    pub(crate) fn framebufferFormat(&self) -> WagyuTextureFormat {
        self.m_framebufferFormat
    }
    pub(crate) fn targetTexture(&self) -> WagyuTexture {
        (&*self.m_targetTexture).clone()
    }
    pub(crate) fn targetTextureView(&self) -> TextureView {
        (&*self.m_targetTextureView).clone()
    }
    pub(crate) fn setTargetTextureView(&mut self, view: TextureView, texture: WagyuTexture) {
        *self.m_targetTextureView = view;
        *self.m_targetTexture = texture;
    }
    pub(crate) fn width(&self) -> u32 {
        self.base.width()
    }
    pub(crate) fn height(&self) -> u32 {
        self.base.height()
    }
}

unsafe impl RefCntTarget for RenderTargetWebGPU {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) };
    }
}

impl Drop for RenderTargetWebGPU {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_dstColorTextureView);
            ManuallyDrop::drop(&mut self.m_msaaDepthStencilTextureView);
            ManuallyDrop::drop(&mut self.m_msaaColorTextureView);
            ManuallyDrop::drop(&mut self.m_scratchColorTextureView);
            ManuallyDrop::drop(&mut self.m_clipTextureView);
            ManuallyDrop::drop(&mut self.m_coverageTextureView);
            ManuallyDrop::drop(&mut self.m_targetTextureView);
            ManuallyDrop::drop(&mut self.m_dstColorTexture);
            ManuallyDrop::drop(&mut self.m_msaaDepthStencilTexture);
            ManuallyDrop::drop(&mut self.m_msaaColorTexture);
            ManuallyDrop::drop(&mut self.m_scratchColorTexture);
            ManuallyDrop::drop(&mut self.m_clipTexture);
            ManuallyDrop::drop(&mut self.m_coverageTexture);
            ManuallyDrop::drop(&mut self.m_targetTexture);
            ManuallyDrop::drop(&mut self.m_device);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

#[repr(C)]
pub(crate) struct TextureWebGPUImpl {
    pub(crate) base: ManuallyDrop<Texture>,
    pub(crate) m_texture: ManuallyDrop<WagyuTexture>,
    pub(crate) m_textureView: ManuallyDrop<TextureView>,
}

unsafe fn textureWebGPUNativeHandle(base: *const Texture) -> *mut c_void {
    let texture = unsafe { &*base.cast::<TextureWebGPUImpl>() };
    texture.m_texture.Get().cast()
}

impl TextureWebGPUImpl {
    pub(crate) fn new(width: u32, height: u32, texture: WagyuTexture) -> Box<Self> {
        let mut base = Texture::new(width, height);
        base.destroy_complete =
            |base| unsafe { drop(Box::from_raw(base.cast::<TextureWebGPUImpl>())) };
        base.setNativeHandleDispatch(textureWebGPUNativeHandle);
        let view = unsafe { texture.CreateView(std::ptr::null()) };
        Box::new(Self {
            base: ManuallyDrop::new(base),
            m_texture: ManuallyDrop::new(texture),
            m_textureView: ManuallyDrop::new(view),
        })
    }
    pub(crate) fn texture(&self) -> WagyuTexture {
        (&*self.m_texture).clone()
    }
    pub(crate) fn textureView(&self) -> TextureView {
        (&*self.m_textureView).clone()
    }
}

unsafe impl RefCntTarget for TextureWebGPUImpl {
    fn r#ref(&self) {
        self.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) };
    }
}

impl Drop for TextureWebGPUImpl {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_textureView);
            ManuallyDrop::drop(&mut self.m_texture);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

pub(crate) const SOURCE_TOP_LEVEL_CLASS_COUNT: usize = 3;
pub(crate) const SOURCE_NESTED_CLASS_COUNT: usize = 11;
pub(crate) const SOURCE_RENDER_CONTEXT_FIELD_COUNT: usize = 36;
pub(crate) const SOURCE_RENDER_TARGET_FIELD_COUNT: usize = 19;
const _: [(); 12778] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::mem::offset_of;

    #[test]
    fn complete_header_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 344);
        assert_eq!(SOURCE_TOP_LEVEL_CLASS_COUNT, 3);
        assert_eq!(SOURCE_NESTED_CLASS_COUNT, 11);
        assert_eq!(SOURCE_RENDER_CONTEXT_FIELD_COUNT, 36);
        assert_eq!(SOURCE_RENDER_TARGET_FIELD_COUNT, 19);
    }

    #[test]
    fn source_bases_are_offset_zero() {
        assert_eq!(offset_of!(RenderContextWebGPUImpl, base), 0);
        assert_eq!(offset_of!(RenderTargetWebGPU, base), 0);
        assert_eq!(offset_of!(TextureWebGPUImpl, base), 0);
    }

    #[test]
    fn raii_arrays_destroy_in_cpp_reverse_element_order() {
        struct Probe<'a> {
            index: usize,
            order: &'a RefCell<Vec<usize>>,
        }
        impl Drop for Probe<'_> {
            fn drop(&mut self) {
                self.order.borrow_mut().push(self.index);
            }
        }

        let order = RefCell::new(Vec::new());
        let mut probes: ManuallyDrop<[Probe<'_>; 3]> =
            ManuallyDrop::new(std::array::from_fn(|index| Probe {
                index,
                order: &order,
            }));
        unsafe { dropArrayReverse(&mut probes) };
        assert_eq!(*order.borrow(), [2, 1, 0]);
    }
}
