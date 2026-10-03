//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/ore/ore_context_vulkan.hpp`.

#![allow(non_snake_case)]

pub(crate) fn targetDescFor(
    target: &super::render_target_vulkan_decl::RenderTargetVulkan,
) -> nuxie_ore_metal::context::TargetDesc {
    match target.framebufferFormat() {
        ash::vk::Format::R8G8B8A8_UNORM | ash::vk::Format::B8G8R8A8_UNORM => {
            nuxie_ore_metal::context::TargetDesc::color8(
                target.width(),
                target.height(),
                target.framebufferFormat() == ash::vk::Format::B8G8R8A8_UNORM,
            )
        }
        _ => Default::default(),
    }
}

use super::vulkan_context_decl::VulkanContext;
use ash::vk;
use nuxie_ore_metal::context::Context;
use nuxie_ore_metal::context::FrameDescriptor;
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::render_pass::RenderPassApi;
use nuxie_ore_metal::types::{
    BindGroupDesc, BindGroupLayoutDesc, BufferDesc, LoadOp, PipelineDesc, RenderPassDesc,
    SamplerDesc, ShaderModuleDesc, StoreOp, TextureDesc, TextureFormat, TextureViewDesc,
};
use std::cell::{Cell, RefCell};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

pub(crate) const MAX_DESCRIPTOR_SETS_PER_GENERATION: u32 = 256;
pub(crate) const kVkProfileSlots: u32 = 3;
pub(crate) const kVkProfilePassesPerSlot: u32 = 64;

#[derive(Default)]
pub(crate) struct VkProfileSlot {
    pub(crate) frameNumber: u64,
    pub(crate) pending: bool,
    pub(crate) labels: Vec<String>,
}

pub(super) struct ContextVulkanLifetime {
    live: Cell<bool>,
}

impl ContextVulkanLifetime {
    pub(super) fn new() -> Rc<Self> {
        Rc::new(Self {
            live: Cell::new(true),
        })
    }

    pub(super) fn retire(&self) {
        self.live.set(false);
    }

    pub(super) fn isLive(&self) -> bool {
        self.live.get()
    }
}

/// Refcounted descriptor pool generation. The pool is destroyed in one shot
/// after both ContextVulkan and all cached BindGroupVulkan sets release it.
pub(crate) struct DescriptorPoolGeneration {
    pub(crate) m_vk: Arc<VulkanContext>,
    pub(crate) m_vkPool: vk::DescriptorPool,
    pub(crate) m_setsAllocated: Mutex<u32>,
}

impl DescriptorPoolGeneration {
    pub(crate) fn hasCapacity(&self) -> bool {
        *self
            .m_setsAllocated
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            < MAX_DESCRIPTOR_SETS_PER_GENERATION
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct VKRenderPassKey {
    pub(crate) colorFormats: [TextureFormat; 4],
    pub(crate) colorCount: u32,
    pub(crate) colorLoadOps: [LoadOp; 4],
    pub(crate) colorStoreOps: [StoreOp; 4],
    pub(crate) colorHasResolve: [bool; 4],
    pub(crate) depthFormat: TextureFormat,
    pub(crate) depthLoadOp: LoadOp,
    pub(crate) depthStoreOp: StoreOp,
    pub(crate) hasDepth: bool,
    pub(crate) sampleCount: u32,
}

impl Default for VKRenderPassKey {
    fn default() -> Self {
        Self {
            colorFormats: [TextureFormat::r8unorm; 4],
            colorCount: 0,
            colorLoadOps: [LoadOp::dontCare; 4],
            colorStoreOps: [StoreOp::discard; 4],
            colorHasResolve: [false; 4],
            depthFormat: TextureFormat::depth24plusStencil8,
            depthLoadOp: LoadOp::dontCare,
            depthStoreOp: StoreOp::discard,
            hasDepth: false,
            sampleCount: 1,
        }
    }
}

impl PartialEq for VKRenderPassKey {
    fn eq(&self, other: &Self) -> bool {
        if self.colorCount != other.colorCount
            || self.hasDepth != other.hasDepth
            || self.sampleCount != other.sampleCount
        {
            return false;
        }
        for index in 0..self.colorCount as usize {
            if self.colorFormats[index] != other.colorFormats[index]
                || self.colorLoadOps[index] != other.colorLoadOps[index]
                || self.colorStoreOps[index] != other.colorStoreOps[index]
                || self.colorHasResolve[index] != other.colorHasResolve[index]
            {
                return false;
            }
        }
        if self.hasDepth
            && (self.depthFormat != other.depthFormat
                || self.depthLoadOp != other.depthLoadOp
                || self.depthStoreOp != other.depthStoreOp)
        {
            return false;
        }
        true
    }
}

impl Eq for VKRenderPassKey {}

pub(crate) struct DescriptorSetAllocation {
    pub(crate) set: vk::DescriptorSet,
    pub(crate) pool: Option<Arc<DescriptorPoolGeneration>>,
}

pub(crate) struct VkPendingImageTransition {
    pub(crate) texture: AnyResourceHandle,
    pub(crate) range: vk::ImageSubresourceRange,
    pub(crate) newLayout: vk::ImageLayout,
}

pub(crate) struct VkPendingTextureUpload {
    pub(crate) texture: AnyResourceHandle,
    pub(crate) stagingBuffer: AnyResourceHandle,
    pub(crate) region: vk::BufferImageCopy,
    pub(crate) aspectMask: vk::ImageAspectFlags,
}

/// Full field/lifetime denominator of the pinned ContextVulkan declaration.
/// Copy and assignment are absent, matching the deleted source operations.
#[repr(C)]
pub(crate) struct ContextVulkan {
    pub(super) base: ManuallyDrop<Context>,
    pub(super) m_lifetime: Rc<ContextVulkanLifetime>,
    pub(super) m_vk: ManuallyDrop<Arc<VulkanContext>>,
    pub(super) m_vkQueue: vk::Queue,
    pub(super) m_vkQueueFamily: u32,
    pub(super) m_vkDepth24Stencil8Format: vk::Format,
    pub(super) m_vkCommandPool: vk::CommandPool,
    pub(super) m_vkCommandBuffer: vk::CommandBuffer,
    pub(super) m_vkDescriptorPool: vk::DescriptorPool,
    pub(super) m_vkFrameFence: vk::Fence,
    pub(super) m_vkCmdBufRecording: bool,
    pub(super) m_currentDescriptorPool: Option<Arc<DescriptorPoolGeneration>>,
    pub(super) m_vkEmptyDSL: vk::DescriptorSetLayout,
    pub(super) m_vkRenderPassCache: Vec<(VKRenderPassKey, vk::RenderPass)>,
    pub(super) m_vkPendingInitialTransitions: Vec<VkPendingImageTransition>,
    // Non-owning stable texture addresses. Texture destruction unregisters
    // through a weak registry handle without borrowing the Context again.
    pub(super) m_vkRiveWrapped:
        Rc<RefCell<Vec<NonNull<super::ore_texture_vulkan_decl::TextureVulkan>>>>,
    pub(super) m_vkPendingTextureUploads: Vec<VkPendingTextureUpload>,
    pub(super) m_vkProfilePool: vk::QueryPool,
    pub(super) m_vkProfileUnavailable: bool,
    pub(super) m_vkProfileOverflowWarned: bool,
    pub(super) m_vkTimestampMask: u64,
    pub(super) m_vkProfileSlots: [VkProfileSlot; kVkProfileSlots as usize],
    pub(super) m_vkProfileSlot: u32,
}

impl ContextVulkan {
    pub(crate) fn Make(vk: Arc<VulkanContext>) -> Option<Box<Self>> {
        super::ore_context_vulkan_impl::Make(vk)
    }

    pub(crate) fn vkAllocateDescriptorSet(
        &mut self,
        dsl: vk::DescriptorSetLayout,
    ) -> DescriptorSetAllocation {
        super::ore_context_vulkan_impl::vkAllocateDescriptorSet(self, dsl)
    }

    pub(crate) fn vkQueuePendingTextureUpload(&mut self, pending: VkPendingTextureUpload) {
        super::ore_context_vulkan_impl::vkQueuePendingTextureUpload(self, pending)
    }

    pub(crate) fn vkQueueTransitionToLayout(
        &mut self,
        texture: &AnyResourceHandle,
        range: vk::ImageSubresourceRange,
        newLayout: vk::ImageLayout,
    ) {
        super::ore_context_vulkan_impl::vkQueueTransitionToLayout(self, texture, range, newLayout)
    }

    pub(crate) fn vkFlushPendingInitialTransitions(&mut self) {
        super::ore_context_vulkan_impl::vkFlushPendingInitialTransitions(self)
    }

    // Loading a never-written subresource reads undefined contents. Marks
    // the view's base mip/layer written regardless of the requested load op.
    pub(super) fn firstUseLoadOp(
        view: &super::ore_texture_vulkan_decl::TextureViewVulkan,
        loadOp: LoadOp,
    ) -> LoadOp {
        super::ore_context_vulkan_impl::firstUseLoadOp(view, loadOp)
    }

    pub(crate) fn vkFlushPendingTextureUploads(&mut self) {
        super::ore_context_vulkan_impl::vkFlushPendingTextureUploads(self)
    }

    pub(crate) fn beginFrame(&mut self, desc: &FrameDescriptor) {
        super::ore_context_vulkan_impl::beginFrame(self, desc)
    }

    pub(crate) fn waitForGPU(&mut self) {
        super::ore_context_vulkan_impl::waitForGPU(self)
    }

    pub(crate) fn endFrame(&mut self) {
        super::ore_context_vulkan_impl::endFrame(self)
    }

    pub(crate) fn vkFormatFor(&self, format: TextureFormat) -> vk::Format {
        super::ore_context_vulkan_impl::vkFormatFor(self, format)
    }

    pub(crate) fn vkGetOrCreateEmptyDSL(&mut self) -> vk::DescriptorSetLayout {
        super::ore_context_vulkan_impl::vkGetOrCreateEmptyDSL(self)
    }

    pub(crate) fn getOrCreateRenderPass(&mut self, key: &VKRenderPassKey) -> vk::RenderPass {
        super::ore_context_vulkan_impl::getOrCreateRenderPass(self, key)
    }

    pub(crate) fn makePipeline(
        &mut self,
        desc: &PipelineDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        super::ore_pipeline_vulkan_impl::makePipeline(self, desc, outError)
    }

    pub(crate) fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        super::ore_context_vulkan_impl::makeBuffer(self, desc)
    }

    pub(crate) fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        super::ore_context_vulkan_impl::makeTexture(self, desc)
    }

    pub(crate) fn makeTextureView(
        &mut self,
        desc: &TextureViewDesc<'_>,
    ) -> Option<AnyResourceHandle> {
        nuxie_ore_metal::context::ContextApi::makeTextureView(self, desc)
    }

    pub(crate) fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        super::ore_context_vulkan_impl::makeSampler(self, desc)
    }

    pub(crate) fn makeShaderModule(
        &mut self,
        desc: &ShaderModuleDesc<'_>,
    ) -> Option<AnyResourceHandle> {
        super::ore_context_vulkan_impl::makeShaderModule(self, desc)
    }

    pub(crate) fn makeBindGroupLayout(
        &mut self,
        desc: &BindGroupLayoutDesc<'_>,
    ) -> Option<AnyResourceHandle> {
        super::ore_context_vulkan_impl::makeBindGroupLayout(self, desc)
    }

    pub(crate) fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        super::ore_context_vulkan_impl::makeBindGroup(self, desc)
    }

    pub(crate) fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        super::ore_context_vulkan_impl::beginRenderPass(self, desc, outError)
    }

    pub(crate) unsafe fn wrapCanvasTexture(
        &mut self,
        canvas: *mut core::ffi::c_void,
    ) -> Option<AnyResourceHandle> {
        unsafe { super::ore_context_vulkan_impl::wrapCanvasTexture(self, canvas) }
    }

    pub(crate) unsafe fn wrapRiveTexture(
        &mut self,
        texture: *mut core::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        unsafe { super::ore_context_vulkan_impl::wrapRiveTexture(self, texture, width, height) }
    }
}

impl std::ops::Deref for ContextVulkan {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ContextVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_pass_key_ignores_inactive_attachment_slots_exactly() {
        let mut left = VKRenderPassKey::default();
        let mut right = VKRenderPassKey::default();
        left.colorFormats[3] = TextureFormat::rgba8unorm;
        right.colorFormats[3] = TextureFormat::bgra8unorm;
        assert_eq!(left, right);
        left.colorCount = 4;
        right.colorCount = 4;
        assert_ne!(left, right);
    }

    #[test]
    fn render_pass_key_ignores_depth_values_when_depth_is_absent() {
        let mut left = VKRenderPassKey::default();
        let mut right = VKRenderPassKey::default();
        right.depthLoadOp = LoadOp::clear;
        assert_eq!(left, right);
        left.hasDepth = true;
        right.hasDepth = true;
        assert_ne!(left, right);
    }
}
