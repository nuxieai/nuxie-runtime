//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/vulkan/render_context_vulkan_impl.hpp`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::draw_pipeline_layout_vulkan_decl::DrawPipelineLayoutVulkan;
use super::pipeline_manager_vulkan_decl::{PipelineManagerVulkan, ShaderCompilationMode};
use super::render_pass_vulkan_decl::RenderPassOptionsVulkan;
use super::render_target_vulkan_decl::{RenderTargetVulkan, RenderTargetVulkanImpl};
use super::vkutil_decl::{
    Buffer, BufferPool, Framebuffer, Image, ImageAccess, ImageAccessAction, ImageView, Texture2D,
};
use super::vulkan_context_decl::{VulkanContext, VulkanFeatures};
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    DrawContents, FlushDescriptor, InterlockMode, LoadAction, PlatformFeatures,
    StorageBufferStructure, IAABB,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, RenderContext,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::RenderContextImpl;
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RiveRenderBuffer;
use ash::vk;
use nuxie_ore_metal::gpu_resource::{
    GPUResource, GPUResourcePool, GpuResourcePayload, ResourceHandle,
};
use nuxie_render_api::ColorInt;
use std::mem::ManuallyDrop;
use std::sync::Arc;
use std::time::Instant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ContextOptions {
    pub(crate) forceAtomicMode: bool,
    pub(crate) disableClockwiseFixedFunctionMode: bool,
    pub(crate) shaderCompilationMode: ShaderCompilationMode,
}

impl Default for ContextOptions {
    fn default() -> Self {
        Self {
            forceAtomicMode: false,
            disableClockwiseFixedFunctionMode: false,
            shaderCompilationMode: ShaderCompilationMode::standard,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DriverWorkarounds {
    pub(crate) maxInstancesPerRenderPass: u32,
    pub(crate) avoidManualMSAAResolves: bool,
    pub(crate) needsManualMSAAResolveAfterDstRead: bool,
}

impl DriverWorkarounds {
    pub(crate) fn needsInterruptibleRenderPasses(&self) -> bool {
        self.maxInstancesPerRenderPass != u32::MAX
    }
}

/// Complete source owner for `RenderBufferVulkanImpl : RiveRenderBuffer`.
#[repr(C)]
pub(crate) struct RenderBufferVulkanImpl {
    pub(crate) base: ManuallyDrop<RiveRenderBuffer>,
    pub(crate) m_bufferPool: ManuallyDrop<ResourceHandle<BufferPool>>,
    pub(crate) m_currentBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
}

pub(crate) struct ResourceTexturePipeline {
    pub(crate) m_vk: Arc<VulkanContext>,
    pub(crate) m_renderPass: vk::RenderPass,
    pub(crate) m_resumingRenderPass: vk::RenderPass,
    pub(crate) m_instanceCountInCurrentRenderPass: u32,
}

pub(crate) struct ColorRampPipeline {
    pub(crate) base: ManuallyDrop<ResourceTexturePipeline>,
    pub(crate) m_pipelineLayout: vk::PipelineLayout,
    pub(crate) m_renderPipeline: vk::Pipeline,
}

pub(crate) struct TessellatePipeline {
    pub(crate) base: ManuallyDrop<ResourceTexturePipeline>,
    pub(crate) m_pipelineLayout: vk::PipelineLayout,
    pub(crate) m_renderPipeline: vk::Pipeline,
}

pub(crate) struct FeatherAtlasPipeline {
    pub(crate) base: ManuallyDrop<ResourceTexturePipeline>,
    pub(crate) m_pipelineLayout: vk::PipelineLayout,
    pub(crate) m_fillPipeline: vk::Pipeline,
    pub(crate) m_strokePipeline: vk::Pipeline,
}

#[repr(C)]
pub(crate) struct DescriptorSetPool {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) m_vk: ManuallyDrop<Arc<VulkanContext>>,
    pub(crate) m_vkDescriptorPool: vk::DescriptorPool,
}

unsafe impl GpuResourcePayload for DescriptorSetPool {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

pub(crate) struct DescriptorSetPoolPool {
    pub(crate) base: GPUResourcePool,
    pub(crate) m_vk: Arc<VulkanContext>,
}

impl DescriptorSetPoolPool {
    pub(crate) const MAX_POOL_SIZE: usize = 64;
}

pub(crate) struct DescriptorSetAllocator {
    pub(crate) m_descriptorSetPoolPool: Arc<DescriptorSetPoolPool>,
    pub(crate) m_descriptorSetPool: Option<ResourceHandle<DescriptorSetPool>>,
    pub(crate) m_perFlushDescriptorSet: vk::DescriptorSet,
    pub(crate) m_perDrawDescriptorSetLayout: vk::DescriptorSetLayout,
    pub(crate) m_imageTextureUpdateCount: u32,
}

pub(crate) struct DrawRenderPass {
    pub(crate) m_impl: *mut RenderContextVulkanImpl,
    pub(crate) m_desc: *const FlushDescriptor,
    pub(crate) m_drawBounds: IAABB,
    pub(crate) m_colorImageView: vk::ImageView,
    pub(crate) m_msaaColorSeedImageView: vk::ImageView,
    pub(crate) m_msaaResolveImageView: vk::ImageView,
    pub(crate) m_pipelineLayout: *const DrawPipelineLayoutVulkan,
    pub(crate) m_renderPassOptions: RenderPassOptionsVulkan,
    pub(crate) m_scissor: IAABB,
    pub(crate) m_patchCountInCurrentDrawPass: u32,
}

#[repr(C)]
pub(crate) struct RenderContextVulkanImpl {
    pub(crate) base: ManuallyDrop<RenderContextImpl>,
    pub(crate) m_vk: ManuallyDrop<Arc<VulkanContext>>,
    pub(crate) m_canvasQueue: vk::Queue,
    pub(crate) m_canvasQueueFamilyIndex: u32,
    pub(crate) m_canvasCommandPool: vk::CommandPool,
    pub(crate) m_workarounds: DriverWorkarounds,
    pub(crate) m_flushUniformBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_pathBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_paintBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_paintAuxBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_contourBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_gradSpanBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_tessSpanBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_triangleBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_imageDrawInstanceBufferPool: ManuallyDrop<BufferPool>,
    pub(crate) m_flushUniformBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_pathBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_paintBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_paintAuxBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_contourBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_gradSpanBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_tessSpanBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_triangleBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_imageDrawInstanceBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_localEpoch: Instant,
    pub(crate) m_nullImageTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_colorRampPipeline: ManuallyDrop<Option<Box<ColorRampPipeline>>>,
    pub(crate) m_gradTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_gradTextureFramebuffer: ManuallyDrop<Option<ResourceHandle<Framebuffer>>>,
    pub(crate) m_tessellatePipeline: ManuallyDrop<Option<Box<TessellatePipeline>>>,
    pub(crate) m_tessSpanIndexBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_tessTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_tesselationSyncIssueWorkaroundTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_tessTextureFramebuffer: ManuallyDrop<Option<ResourceHandle<Framebuffer>>>,
    pub(crate) m_featherAtlasPipeline: ManuallyDrop<Option<Box<FeatherAtlasPipeline>>>,
    pub(crate) m_featherAtlasTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_featherAtlasFramebuffer: ManuallyDrop<Option<ResourceHandle<Framebuffer>>>,
    pub(crate) m_plsTransientUsageFlags: vk::ImageUsageFlags,
    pub(crate) m_plsExtent: vk::Extent3D,
    pub(crate) m_plsTransientPlaneCount: u32,
    pub(crate) m_plsTransientImageArray: ManuallyDrop<Option<ResourceHandle<Image>>>,
    pub(crate) m_plsTransientCoverageView: ManuallyDrop<Option<ResourceHandle<ImageView>>>,
    pub(crate) m_plsTransientClipView: ManuallyDrop<Option<ResourceHandle<ImageView>>>,
    pub(crate) m_plsTransientScratchColorTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_plsBlendStorageTexture_RGB10_A2: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_plsTransientClipTexture_R16F: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_plsOffscreenColorTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_plsAtomicCoverageTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_coverageBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_gaussianIntegralTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_pathPatchVertexBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_pathPatchIndexBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_imageRectVertexBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_imageRectIndexBuffer: ManuallyDrop<Option<ResourceHandle<Buffer>>>,
    pub(crate) m_descriptorSetPoolPool: ManuallyDrop<Arc<DescriptorSetPoolPool>>,
    pub(crate) m_pipelineManager: ManuallyDrop<Option<core::pin::Pin<Box<PipelineManagerVulkan>>>>,
}

impl RenderContextVulkanImpl {
    pub(crate) fn platformFeatures(&self) -> &PlatformFeatures {
        self.base.platformFeatures()
    }
    pub(crate) fn vulkanContext(&self) -> &VulkanContext {
        &self.m_vk
    }

    pub(crate) fn makeRenderTarget(
        &self,
        width: u32,
        height: u32,
        framebufferFormat: vk::Format,
        targetUsageFlags: vk::ImageUsageFlags,
    ) -> rcp<RenderTargetVulkanImpl> {
        super::render_target_vulkan_impl::makeRenderTarget(
            Arc::clone(&self.m_vk),
            width,
            height,
            framebufferFormat,
            targetUsageFlags,
        )
    }

    /// # Safety
    /// `queue` must belong to this context's device and `queueFamilyIndex`
    /// must be its actual family for the complete canvas submission lifetime.
    pub(crate) unsafe fn setCanvasQueue(&mut self, queue: vk::Queue, queueFamilyIndex: u32) {
        unsafe {
            super::render_context_vulkan_impl::setCanvasQueue(self, queue, queueFamilyIndex)
        }
    }
    /// # Safety
    /// `image` must be a live externally owned image from this context's
    /// device, with the supplied extent/format, until the returned texture is
    /// no longer used.
    pub(crate) unsafe fn adoptImageTexture(
        &self,
        image: vk::Image,
        width: u32,
        height: u32,
        format: vk::Format,
    ) -> rcp<Texture2D> {
        unsafe {
            super::render_context_vulkan_impl::adoptImageTexture(
                self, image, width, height, format,
            )
        }
    }
    #[cfg(feature = "native-ore-vulkan-experimental")]
    pub(crate) fn makeRenderCanvas(
        &mut self,
        width: u32,
        height: u32,
    ) -> rcp<crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas>{
        super::render_context_vulkan_impl::makeRenderCanvas(self, width, height)
    }
    #[cfg(feature = "native-ore-vulkan-experimental")]
    pub(crate) fn makeOreContext(
        &self,
    ) -> Option<Box<super::ore_context_vulkan_decl::ContextVulkan>> {
        super::ore_context_vulkan_decl::ContextVulkan::Make(Arc::clone(&self.m_vk))
    }
    pub(crate) fn hotloadShaders(&mut self, spirvData: &[u32]) {
        super::render_context_vulkan_impl::hotloadShaders(self, spirvData)
    }
    pub(crate) fn startAsyncPipelineCreation(
        &self,
        interlockMode: InterlockMode,
        framebufferFormat: vk::Format,
        framebufferUsage: vk::ImageUsageFlags,
        colorLoadAction: LoadAction,
    ) {
        super::render_context_vulkan_impl::startAsyncPipelineCreation(
            self,
            interlockMode,
            framebufferFormat,
            framebufferUsage,
            colorLoadAction,
        )
    }
    pub(crate) fn startAsyncPipelineCreationForRenderTarget(
        &self,
        interlockMode: InterlockMode,
        renderTarget: &RenderTargetVulkan,
        colorLoadAction: LoadAction,
    ) {
        super::render_context_vulkan_impl::startAsyncPipelineCreationForRenderTarget(
            self,
            interlockMode,
            renderTarget,
            colorLoadAction,
        )
    }
    pub(crate) fn waitForAsyncPipelineCreation(&self) {
        super::render_context_vulkan_impl::waitForAsyncPipelineCreation(self)
    }
}

/// # Safety
/// All native handles and `pfnGetInstanceProcAddr` must form one compatible,
/// live Vulkan ownership tuple and outlive the returned render context.
pub(crate) unsafe fn MakeContext(
    instance: vk::Instance,
    physicalDevice: vk::PhysicalDevice,
    device: vk::Device,
    features: VulkanFeatures,
    pfnGetInstanceProcAddr: vk::PFN_vkGetInstanceProcAddr,
    options: ContextOptions,
) -> Option<std::pin::Pin<Box<RenderContext>>> {
    unsafe {
        super::render_context_vulkan_impl::MakeContext(
            instance,
            physicalDevice,
            device,
            features,
            pfnGetInstanceProcAddr,
            options,
        )
    }
}

/// # Safety
/// The native ownership requirements are identical to `MakeContext`.
pub(crate) unsafe fn MakeContextDefault(
    instance: vk::Instance,
    physicalDevice: vk::PhysicalDevice,
    device: vk::Device,
    features: VulkanFeatures,
    pfnGetInstanceProcAddr: vk::PFN_vkGetInstanceProcAddr,
) -> Option<std::pin::Pin<Box<RenderContext>>> {
    unsafe {
        MakeContext(
            instance,
            physicalDevice,
            device,
            features,
            pfnGetInstanceProcAddr,
            ContextOptions::default(),
        )
    }
}
