//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/vulkan/render_target_vulkan.hpp`.

#![allow(non_snake_case)]

use super::vkutil_decl::{ImageAccess, ImageAccessAction, Texture2D};
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{rcp, safe_ref, RefCntTarget};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::{
    RenderTarget, IAABB,
};
use ash::vk;
use nuxie_render_api::ColorInt;
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::sync::Arc;

pub(crate) trait RenderTargetVulkanApi {
    fn base(&self) -> &RenderTargetVulkan;
    fn baseMut(&mut self) -> &mut RenderTargetVulkan;
    fn targetImage(&self) -> vk::Image;
    fn targetImageView(&self) -> vk::ImageView;
    fn updateLastAccess(&mut self, _access: ImageAccess) {}
    fn accessTargetImage(
        &mut self,
        command_buffer: vk::CommandBuffer,
        dst_access: ImageAccess,
        action: ImageAccessAction,
    ) -> vk::Image;
    fn accessTargetImageView(
        &mut self,
        command_buffer: vk::CommandBuffer,
        dst_access: ImageAccess,
        action: ImageAccessAction,
    ) -> vk::ImageView;
}

/// The two complete-object layouts represented by the pinned C++
/// `RenderTargetVulkan` virtual interface.
///
/// This Rust-only sidecar is the stable equivalent of the source vtable's
/// concrete dispatch identity. It is initialized only by the two translated
/// complete-object constructors.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RenderTargetVulkanKind {
    External,
    Texture,
}

/// A Vulkan render-target base pointer paired with the intrusive owner that
/// keeps its complete concrete allocation alive.
pub(crate) struct RetainedRenderTargetVulkan {
    target: NonNull<RenderTarget>,
    owner: rcp<RenderTarget>,
}

impl RetainedRenderTargetVulkan {
    /// # Safety
    /// `target` must carry the original allocation provenance for a live,
    /// heap-published intrusive render target whose offset-zero base owns a
    /// tagged Vulkan complete object.
    pub(crate) unsafe fn fromLiveTarget(target: NonNull<RenderTarget>) -> Self {
        let owner = unsafe { rcp::from_ptr(safe_ref(target.as_ptr())) };
        Self { target, owner }
    }

    pub(crate) fn updateLastAccess(&mut self, access: ImageAccess) {
        // SAFETY: every clone retains `owner`, which owns this exact
        // offset-zero target allocation. The narrow dispatch does not return
        // a reference, so cloned wrappers cannot manufacture aliased mutable
        // borrows in safe Rust.
        unsafe {
            super::render_context_vulkan_impl::updateLiveRenderTargetVulkanLastAccess(
                self.target,
                access,
            )
        }
    }
}

impl Clone for RetainedRenderTargetVulkan {
    fn clone(&self) -> Self {
        Self {
            target: self.target,
            owner: self.owner.clone(),
        }
    }
}

#[repr(C)]
pub(crate) struct RenderTargetVulkan {
    pub(crate) base: ManuallyDrop<RenderTarget>,
    pub(crate) m_vk: ManuallyDrop<Arc<VulkanContext>>,
    pub(crate) m_framebufferFormat: vk::Format,
    pub(crate) m_targetUsageFlags: vk::ImageUsageFlags,
    pub(crate) m_offscreenColorTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_msaaColorTexture: ManuallyDrop<rcp<Texture2D>>,
    pub(crate) m_msaaDepthStencilTexture: ManuallyDrop<rcp<Texture2D>>,
    /// Rust-only concrete dispatch identity for the source virtual interface.
    pub(super) rust_complete_kind: RenderTargetVulkanKind,
}

impl RenderTargetVulkan {
    pub(crate) fn framebufferFormat(&self) -> vk::Format {
        self.m_framebufferFormat
    }
    pub(crate) fn targetUsageFlags(&self) -> vk::ImageUsageFlags {
        self.m_targetUsageFlags
    }
    pub(crate) fn width(&self) -> u32 {
        self.base.width()
    }
    pub(crate) fn height(&self) -> u32 {
        self.base.height()
    }
    pub(crate) fn bounds(&self) -> IAABB {
        self.base.bounds()
    }

    pub(crate) fn clearTargetImageView(
        target: &mut dyn RenderTargetVulkanApi,
        command_buffer: vk::CommandBuffer,
        clear_color: ColorInt,
        dst_access_after_clear: ImageAccess,
    ) -> vk::ImageView {
        super::render_target_vulkan_impl::clearTargetImageView(
            target,
            command_buffer,
            clear_color,
            dst_access_after_clear,
        )
    }
}

impl Drop for RenderTargetVulkan {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_msaaDepthStencilTexture);
            ManuallyDrop::drop(&mut self.m_msaaColorTexture);
            ManuallyDrop::drop(&mut self.m_offscreenColorTexture);
            ManuallyDrop::drop(&mut self.m_vk);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

#[repr(C)]
pub(crate) struct RenderTargetVulkanImpl {
    pub(crate) base: ManuallyDrop<RenderTargetVulkan>,
    pub(crate) m_targetImageView: vk::ImageView,
    pub(crate) m_targetImage: vk::Image,
    pub(crate) m_targetLastAccess: ImageAccess,
}

impl RenderTargetVulkanImpl {
    /// # Safety
    /// `image_view` must view `image`, both must belong to this render target's
    /// device, and the pair must remain live in `target_last_access` state for
    /// every use of this target.
    pub(crate) unsafe fn setTargetImageView(
        &mut self,
        image_view: vk::ImageView,
        image: vk::Image,
        target_last_access: ImageAccess,
    ) {
        self.m_targetImageView = image_view;
        self.m_targetImage = image;
        self.m_targetLastAccess = target_last_access;
    }

    pub(crate) fn targetLastAccess(&self) -> &ImageAccess {
        &self.m_targetLastAccess
    }
}

unsafe impl RefCntTarget for RenderTargetVulkanImpl {
    fn r#ref(&self) {
        self.base.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.base.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) };
    }
}

impl RenderTargetVulkanApi for RenderTargetVulkanImpl {
    fn base(&self) -> &RenderTargetVulkan {
        &self.base
    }
    fn baseMut(&mut self) -> &mut RenderTargetVulkan {
        &mut self.base
    }
    fn targetImage(&self) -> vk::Image {
        self.m_targetImage
    }
    fn targetImageView(&self) -> vk::ImageView {
        self.m_targetImageView
    }
    fn updateLastAccess(&mut self, access: ImageAccess) {
        self.m_targetLastAccess = access;
    }
    fn accessTargetImage(
        &mut self,
        command_buffer: vk::CommandBuffer,
        dst_access: ImageAccess,
        action: ImageAccessAction,
    ) -> vk::Image {
        super::render_target_vulkan_impl::accessTargetImage(
            self,
            command_buffer,
            dst_access,
            action,
        )
    }
    fn accessTargetImageView(
        &mut self,
        command_buffer: vk::CommandBuffer,
        dst_access: ImageAccess,
        action: ImageAccessAction,
    ) -> vk::ImageView {
        super::render_target_vulkan_impl::accessTargetImageView(
            self,
            command_buffer,
            dst_access,
            action,
        )
    }
}

impl Drop for RenderTargetVulkanImpl {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}
