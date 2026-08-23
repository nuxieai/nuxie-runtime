//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/render_target_vulkan.cpp`.

#![allow(non_snake_case)]

use super::render_target_vulkan_decl::{
    RenderTargetVulkan, RenderTargetVulkanApi, RenderTargetVulkanImpl,
};
use super::vkutil_decl::{ImageAccess, ImageAccessAction, Texture2D};
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::{
    RenderTarget, IAABB,
};
use ash::vk;
use nuxie_render_api::ColorInt;
use std::ffi::CStr;
use std::mem::ManuallyDrop;
use std::sync::Arc;

pub(crate) unsafe fn destroy_render_target_vulkan_impl(ptr: *mut RenderTarget) {
    unsafe { drop(Box::from_raw(ptr.cast::<RenderTargetVulkanImpl>())) };
}

pub(crate) fn makeRenderTarget(
    vk_context: Arc<VulkanContext>,
    width: u32,
    height: u32,
    framebuffer_format: vk::Format,
    target_usage_flags: vk::ImageUsageFlags,
) -> rcp<RenderTargetVulkanImpl> {
    make_rcp(|| {
        let mut render_target = RenderTarget::new(width, height);
        render_target.destroy_complete = destroy_render_target_vulkan_impl;
        debug_assert!(
            target_usage_flags.contains(vk::ImageUsageFlags::INPUT_ATTACHMENT)
                || target_usage_flags.contains(
                    vk::ImageUsageFlags::TRANSFER_SRC | vk::ImageUsageFlags::TRANSFER_DST,
                )
        );
        RenderTargetVulkanImpl {
            base: ManuallyDrop::new(RenderTargetVulkan {
                base: ManuallyDrop::new(render_target),
                m_vk: ManuallyDrop::new(vk_context),
                m_framebufferFormat: framebuffer_format,
                m_targetUsageFlags: target_usage_flags,
                m_offscreenColorTexture: ManuallyDrop::new(rcp::new()),
                m_msaaColorTexture: ManuallyDrop::new(rcp::new()),
                m_msaaDepthStencilTexture: ManuallyDrop::new(rcp::new()),
            }),
            m_targetImageView: vk::ImageView::null(),
            m_targetImage: vk::Image::null(),
            m_targetLastAccess: ImageAccess::default(),
        }
    })
}

pub(crate) fn accessTargetImage(
    target: &mut RenderTargetVulkanImpl,
    command_buffer: vk::CommandBuffer,
    dst_access: ImageAccess,
    image_access_action: ImageAccessAction,
) -> vk::Image {
    target.m_targetLastAccess = target.base.m_vk.simpleImageMemoryBarrier(
        command_buffer,
        target.m_targetLastAccess,
        dst_access,
        target.m_targetImage,
        image_access_action,
        vk::DependencyFlags::empty(),
    );
    target.m_targetImage
}

pub(crate) fn accessTargetImageView(
    target: &mut RenderTargetVulkanImpl,
    command_buffer: vk::CommandBuffer,
    dst_access: ImageAccess,
    image_access_action: ImageAccessAction,
) -> vk::ImageView {
    accessTargetImage(target, command_buffer, dst_access, image_access_action);
    target.m_targetImageView
}

pub(crate) fn clearTargetImageView(
    target: &mut dyn RenderTargetVulkanApi,
    command_buffer: vk::CommandBuffer,
    clear_color: ColorInt,
    dst_access_after_clear: ImageAccess,
) -> vk::ImageView {
    let transfer_access = ImageAccess {
        pipelineStages: vk::PipelineStageFlags::TRANSFER,
        accessMask: vk::AccessFlags::TRANSFER_WRITE,
        layout: vk::ImageLayout::GENERAL,
    };
    let image = target.accessTargetImage(
        command_buffer,
        transfer_access,
        ImageAccessAction::invalidateContents,
    );
    target.base().m_vk.clearColorImage(
        command_buffer,
        clear_color,
        image,
        vk::ImageLayout::GENERAL,
    );
    target.accessTargetImageView(
        command_buffer,
        dst_access_after_clear,
        ImageAccessAction::preserveContents,
    )
}

pub(crate) fn accessOffscreenColorTexture(
    target: &mut dyn RenderTargetVulkanApi,
    command_buffer: vk::CommandBuffer,
    dst_access: ImageAccess,
    image_access_action: ImageAccessAction,
) -> *mut Texture2D {
    let base = target.baseMut();
    if base.m_offscreenColorTexture.get().is_null() {
        let info = vk::ImageCreateInfo::default()
            .format(base.m_framebufferFormat)
            .extent(vk::Extent3D {
                width: base.width(),
                height: base.height(),
                depth: 0,
            })
            .usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::INPUT_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST,
            );
        let name = CStr::from_bytes_with_nul(b"offscreen color texture\0").unwrap();
        *base.m_offscreenColorTexture = base.m_vk.makeTexture2D(info, Some(name));
    }
    let texture = unsafe { &*base.m_offscreenColorTexture.get() };
    texture.barrier(
        command_buffer,
        dst_access,
        image_access_action,
        vk::DependencyFlags::empty(),
    );
    base.m_offscreenColorTexture.get()
}

pub(crate) fn copyTargetImageToOffscreenColorTexture(
    target: &mut dyn RenderTargetVulkanApi,
    command_buffer: vk::CommandBuffer,
    dst_access_after_copy: ImageAccess,
    copy_bounds: &IAABB,
) -> *mut Texture2D {
    let source = target.accessTargetImage(
        command_buffer,
        ImageAccess {
            pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_READ,
            layout: vk::ImageLayout::GENERAL,
        },
        ImageAccessAction::preserveContents,
    );
    let destination = accessOffscreenColorTexture(
        target,
        command_buffer,
        ImageAccess {
            pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_WRITE,
            layout: vk::ImageLayout::GENERAL,
        },
        ImageAccessAction::invalidateContents,
    );
    let gpu_bounds =
        crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB {
            left: copy_bounds.left,
            top: copy_bounds.top,
            right: copy_bounds.right,
            bottom: copy_bounds.bottom,
        };
    target.base().m_vk.blitSubRect(
        command_buffer,
        source,
        vk::ImageLayout::GENERAL,
        unsafe { (&*destination).vkImage() },
        vk::ImageLayout::GENERAL,
        &gpu_bounds,
    );
    accessOffscreenColorTexture(
        target,
        command_buffer,
        dst_access_after_copy,
        ImageAccessAction::preserveContents,
    )
}

pub(crate) fn msaaColorTexture(target: &mut dyn RenderTargetVulkanApi) -> *mut Texture2D {
    let base = target.baseMut();
    if base.m_msaaColorTexture.get().is_null() {
        let info = vk::ImageCreateInfo::default()
            .format(base.m_framebufferFormat)
            .extent(vk::Extent3D {
                width: base.width(),
                height: base.height(),
                depth: 1,
            })
            .samples(vk::SampleCountFlags::TYPE_4)
            .usage(
                vk::ImageUsageFlags::COLOR_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
                    | vk::ImageUsageFlags::INPUT_ATTACHMENT,
            );
        let name = CStr::from_bytes_with_nul(b"MSAA Color Texture\0").unwrap();
        *base.m_msaaColorTexture = base.m_vk.makeTexture2D(info, Some(name));
    }
    base.m_msaaColorTexture.get()
}

pub(crate) fn msaaDepthStencilTexture(target: &mut dyn RenderTargetVulkanApi) -> *mut Texture2D {
    let base = target.baseMut();
    if base.m_msaaDepthStencilTexture.get().is_null() {
        let format =
            super::vkutil_decl::get_preferred_depth_stencil_format(base.m_vk.supportsD24S8());
        let info = vk::ImageCreateInfo::default()
            .format(format)
            .extent(vk::Extent3D {
                width: base.width(),
                height: base.height(),
                depth: 1,
            })
            .samples(vk::SampleCountFlags::TYPE_4)
            .usage(
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
                    | vk::ImageUsageFlags::TRANSIENT_ATTACHMENT,
            );
        let name = CStr::from_bytes_with_nul(b"MSAA Depth/Stencil Texture\0").unwrap();
        *base.m_msaaDepthStencilTexture = base.m_vk.makeTexture2D(info, Some(name));
    }
    base.m_msaaDepthStencilTexture.get()
}
