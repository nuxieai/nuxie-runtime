//! Complete mechanical implementation translation of
//! `renderer/src/ore/vulkan/ore_render_pass_vulkan.cpp`.

#![allow(non_snake_case)]

use super::ore_bind_group_vulkan_decl::BindGroupVulkan;
use super::ore_buffer_vulkan_decl::BufferVulkan;
use super::ore_pipeline_vulkan_decl::PipelineVulkan;
use super::ore_render_pass_vulkan_decl::{RenderPassVulkan, RenderPassVulkanState};
use super::ore_texture_vulkan_decl::TextureVulkan;
use super::vkutil_decl::ImageAccess;
use ash::vk;
use nuxie_ore_metal::context::ActiveRenderPass;
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::render_pass::RenderPassApi;
use nuxie_ore_metal::types::{IndexFormat, TextureFormat};
use std::any::Any;
use std::mem::ManuallyDrop;
use std::rc::Weak as RcWeak;

pub(crate) fn setPipeline(pass: &mut RenderPassVulkanState, inPipeline: &AnyResourceHandle) {
    let pipeline = inPipeline
        .downcast_ref::<PipelineVulkan>()
        .expect("RenderPassVulkan requires a PipelineVulkan");
    if !nuxie_ore_metal::render_pass_check_pipeline_compat(&pass.base, &pipeline.base) {
        return;
    }
    let retained = inPipeline
        .clone()
        .downcast::<PipelineVulkan>()
        .unwrap_or_else(|_| unreachable!("checked PipelineVulkan downcast"));
    let commandBuffer = pass.m_vkCmdBuf;
    let context = pass.context();
    unsafe {
        context.m_vk.m_ashDevice.cmd_bind_pipeline(
            commandBuffer,
            vk::PipelineBindPoint::GRAPHICS,
            pipeline.m_vkPipeline,
        );
        if pipeline.m_vkStencilTestEnabled {
            context.m_vk.m_ashDevice.cmd_set_stencil_reference(
                commandBuffer,
                vk::StencilFaceFlags::FRONT_AND_BACK,
                pass.m_vkStencilRef,
            );
        }
    }
    *pass.m_currentPipeline = Some(retained);
}

pub(crate) fn setVertexBuffer(
    pass: &mut RenderPassVulkanState,
    slot: u32,
    inBuffer: &AnyResourceHandle,
    offset: u32,
) {
    let buffer = inBuffer
        .downcast_ref::<BufferVulkan>()
        .expect("RenderPassVulkan requires a BufferVulkan");
    buffer.markBound();
    let vkBuffer = buffer.current();
    let vkOffset = u64::from(offset);
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_bind_vertex_buffers(
            pass.m_vkCmdBuf,
            slot,
            std::slice::from_ref(&vkBuffer),
            std::slice::from_ref(&vkOffset),
        );
    }
}

pub(crate) fn setIndexBuffer(
    pass: &mut RenderPassVulkanState,
    inBuffer: &AnyResourceHandle,
    format: IndexFormat,
    offset: u32,
) {
    let buffer = inBuffer
        .downcast_ref::<BufferVulkan>()
        .expect("RenderPassVulkan requires a BufferVulkan");
    buffer.markBound();
    pass.m_vkIndexBuffer = buffer.current();
    pass.m_vkIndexType = if format == IndexFormat::uint32 {
        vk::IndexType::UINT32
    } else {
        vk::IndexType::UINT16
    };
    pass.m_vkIndexOffset = offset;
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_bind_index_buffer(
            pass.m_vkCmdBuf,
            pass.m_vkIndexBuffer,
            u64::from(offset),
            pass.m_vkIndexType,
        );
    }
}

pub(crate) fn setBindGroup(
    pass: &mut RenderPassVulkanState,
    groupIndex: u32,
    inBg: &AnyResourceHandle,
    dynamicOffsets: Option<&[u32]>,
    dynamicOffsetCount: u32,
) {
    assert!(
        pass.m_currentPipeline.is_some(),
        "setPipeline must be called before setBindGroup"
    );
    let bg = inBg
        .downcast_ref::<BindGroupVulkan>()
        .expect("RenderPassVulkan requires a BindGroupVulkan");
    let set = bg.resolveDescriptorSet();
    if set == vk::DescriptorSet::null() {
        pass.context().setLastError(format!(
            "ore: Vulkan descriptor set allocation failed for group {groupIndex}"
        ));
        return;
    }
    bg.markUBOsBound();
    let offsets = dynamicOffsets.unwrap_or(&[]);
    let offsets = offsets
        .get(..dynamicOffsetCount as usize)
        .expect("dynamicOffsetCount exceeds its authored span");
    let pipelineLayout = pass
        .m_currentPipeline
        .as_ref()
        .expect("pipeline checked above")
        .m_vkPipelineLayout;
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_bind_descriptor_sets(
            pass.m_vkCmdBuf,
            vk::PipelineBindPoint::GRAPHICS,
            pipelineLayout,
            groupIndex,
            std::slice::from_ref(&set),
            offsets,
        );
    }
    nuxie_ore_metal::render_pass_retain_bound_group(&mut pass.base, groupIndex, inBg.clone());
}

pub(crate) fn setViewport(
    pass: &mut RenderPassVulkanState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    minDepth: f32,
    maxDepth: f32,
) {
    let viewport = vk::Viewport {
        x,
        y,
        width,
        height,
        min_depth: minDepth,
        max_depth: maxDepth,
    };
    let x0 = x.floor();
    let y0 = y.floor();
    let x1 = (x + width).ceil();
    let y1 = (y + height).ceil();
    let scissor = vk::Rect2D {
        offset: vk::Offset2D {
            x: x0.max(0.0) as i32,
            y: y0.max(0.0) as i32,
        },
        extent: vk::Extent2D {
            width: (x1 - x0).max(0.0) as u32,
            height: (y1 - y0).max(0.0) as u32,
        },
    };
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_set_viewport(
            pass.m_vkCmdBuf,
            0,
            std::slice::from_ref(&viewport),
        );
        pass.context().m_vk.m_ashDevice.cmd_set_scissor(
            pass.m_vkCmdBuf,
            0,
            std::slice::from_ref(&scissor),
        );
    }
}

pub(crate) fn setScissorRect(
    pass: &mut RenderPassVulkanState,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    let scissor = vk::Rect2D {
        offset: vk::Offset2D {
            x: x as i32,
            y: y as i32,
        },
        extent: vk::Extent2D { width, height },
    };
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_set_scissor(
            pass.m_vkCmdBuf,
            0,
            std::slice::from_ref(&scissor),
        );
    }
}

pub(crate) fn setStencilReference(pass: &mut RenderPassVulkanState, reference: u32) {
    pass.m_vkStencilRef = reference;
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_set_stencil_reference(
            pass.m_vkCmdBuf,
            vk::StencilFaceFlags::FRONT_AND_BACK,
            reference,
        );
    }
}

pub(crate) fn setBlendColor(pass: &mut RenderPassVulkanState, r: f32, g: f32, b: f32, a: f32) {
    unsafe {
        pass.context()
            .m_vk
            .m_ashDevice
            .cmd_set_blend_constants(pass.m_vkCmdBuf, &[r, g, b, a]);
    }
}

pub(crate) fn draw(
    pass: &mut RenderPassVulkanState,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_draw(
            pass.m_vkCmdBuf,
            vertexCount,
            instanceCount,
            firstVertex,
            firstInstance,
        );
    }
}

pub(crate) fn drawIndexed(
    pass: &mut RenderPassVulkanState,
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    baseVertex: i32,
    firstInstance: u32,
) {
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_draw_indexed(
            pass.m_vkCmdBuf,
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        );
    }
}

fn transitionColorImage(
    pass: &RenderPassVulkanState,
    image: vk::Image,
    baseMip: u32,
    baseLayer: u32,
    layerCount: u32,
) {
    let barrier = vk::ImageMemoryBarrier::default()
        .old_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
        .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: baseMip,
            level_count: 1,
            base_array_layer: baseLayer,
            layer_count: layerCount,
        })
        .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
        .dst_access_mask(vk::AccessFlags::empty());
    unsafe {
        pass.context().m_vk.m_ashDevice.cmd_pipeline_barrier(
            pass.m_vkCmdBuf,
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::PipelineStageFlags::BOTTOM_OF_PIPE,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&barrier),
        );
    }
}

pub(crate) fn finish(pass: &mut RenderPassVulkanState) {
    if nuxie_ore_metal::render_pass_is_finished(&pass.base) {
        return;
    }
    nuxie_ore_metal::render_pass_set_finished(&mut pass.base, true);
    unsafe {
        pass.context()
            .m_vk
            .m_ashDevice
            .cmd_end_render_pass(pass.m_vkCmdBuf);
    }

    let colorAttachmentWriteAccess = ImageAccess {
        pipelineStages: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        accessMask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };
    for index in 0..pass.m_vkColorCount as usize {
        let image = pass.m_vkColorImages[index];
        if image == vk::Image::null() {
            continue;
        }
        transitionColorImage(
            pass,
            image,
            0,
            pass.m_vkColorBaseLayer[index],
            pass.m_vkColorLayerCount[index],
        );
        if let Some(target) = pass.m_vkColorRenderTargets[index].as_mut() {
            target
                .targetMut()
                .updateLastAccess(colorAttachmentWriteAccess);
        }
        if let Some(texture) = pass.m_vkColorTextures[index]
            .as_ref()
            .and_then(|texture| texture.downcast_ref::<TextureVulkan>())
        {
            texture
                .m_vkLayout
                .set(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        }
    }

    for index in 0..pass.m_vkResolveTargets.len() {
        let (image, baseMip, baseLayer, layerCount) = {
            let resolve = &pass.m_vkResolveTargets[index];
            (
                resolve.image,
                resolve.baseMip,
                resolve.baseLayer,
                resolve.layerCount,
            )
        };
        if image == vk::Image::null() {
            continue;
        }
        transitionColorImage(pass, image, baseMip, baseLayer, layerCount);
        let resolve = &mut pass.m_vkResolveTargets[index];
        if let Some(target) = resolve.renderTarget.as_mut() {
            target
                .targetMut()
                .updateLastAccess(colorAttachmentWriteAccess);
        }
        if let Some(texture) = resolve
            .texture
            .as_ref()
            .and_then(|texture| texture.downcast_ref::<TextureVulkan>())
        {
            texture
                .m_vkLayout
                .set(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        }
    }

    if pass.m_vkDepthImage != vk::Image::null() {
        let mut depthAspect = vk::ImageAspectFlags::DEPTH;
        if matches!(
            nuxie_ore_metal::render_pass_depth_format(&pass.base),
            TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
        ) {
            depthAspect |= vk::ImageAspectFlags::STENCIL;
        }
        let barrier = vk::ImageMemoryBarrier::default()
            .old_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
            .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
            .image(pass.m_vkDepthImage)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: depthAspect,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: pass.m_vkDepthBaseLayer,
                layer_count: pass.m_vkDepthLayerCount,
            })
            .src_access_mask(vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE)
            .dst_access_mask(vk::AccessFlags::empty());
        unsafe {
            pass.context().m_vk.m_ashDevice.cmd_pipeline_barrier(
                pass.m_vkCmdBuf,
                vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                    | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&barrier),
            );
        }
        if let Some(texture) = pass
            .m_vkDepthTexture
            .as_ref()
            .and_then(|texture| texture.downcast_ref::<TextureVulkan>())
        {
            texture
                .m_vkLayout
                .set(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        }
    }
}

impl Drop for RenderPassVulkanState {
    fn drop(&mut self) {
        if self.contextIsLive() {
            finish(self);
        } else {
            // A safe retained pass may be released after its context. The
            // native graph is already retired, so quarantine native finish
            // work and dismantle only the pass's retained Rust owners.
            nuxie_ore_metal::render_pass_set_finished(&mut self.base, true);
        }
        unsafe {
            ManuallyDrop::drop(&mut self.m_vkDepthTexture);
            ManuallyDrop::drop(&mut self.m_vkResolveTargets);
            ManuallyDrop::drop(&mut self.m_vkColorTextures);
            ManuallyDrop::drop(&mut self.m_framebuffer);
            ManuallyDrop::drop(&mut self.m_currentPipeline);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

pub(crate) fn validate(pass: &RenderPassVulkanState) {
    assert!(
        !nuxie_ore_metal::render_pass_is_finished(&pass.base),
        "RenderPass has already been finished"
    );
}

impl RenderPassApi for RenderPassVulkan {
    fn asAny(&self) -> &dyn Any {
        self
    }
    fn asAnyMut(&mut self) -> &mut dyn Any {
        self
    }
    fn intoAny(self: Box<Self>) -> Box<dyn Any> {
        self
    }
    fn activeToken(&self) -> RcWeak<dyn ActiveRenderPass> {
        self.activeToken()
    }

    fn setPipeline(&mut self, pipeline: Option<&AnyResourceHandle>) {
        if let Some(pipeline) = pipeline {
            setPipeline(&mut self.borrowState(), pipeline);
        }
    }
    fn setVertexBuffer(&mut self, slot: u32, buffer: Option<&AnyResourceHandle>, offset: u32) {
        if let Some(buffer) = buffer {
            setVertexBuffer(&mut self.borrowState(), slot, buffer, offset);
        }
    }
    fn setIndexBuffer(
        &mut self,
        buffer: Option<&AnyResourceHandle>,
        format: IndexFormat,
        offset: u32,
    ) {
        if let Some(buffer) = buffer {
            setIndexBuffer(&mut self.borrowState(), buffer, format, offset);
        }
    }
    fn setBindGroup(
        &mut self,
        groupIndex: u32,
        bindGroup: Option<&AnyResourceHandle>,
        dynamicOffsets: Option<&[u32]>,
        dynamicOffsetCount: u32,
    ) {
        if let Some(bindGroup) = bindGroup {
            setBindGroup(
                &mut self.borrowState(),
                groupIndex,
                bindGroup,
                dynamicOffsets,
                dynamicOffsetCount,
            );
        }
    }
    fn setViewport(
        &mut self,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        minDepth: f32,
        maxDepth: f32,
    ) {
        setViewport(
            &mut self.borrowState(),
            x,
            y,
            width,
            height,
            minDepth,
            maxDepth,
        );
    }
    fn setScissorRect(&mut self, x: u32, y: u32, width: u32, height: u32) {
        setScissorRect(&mut self.borrowState(), x, y, width, height);
    }
    fn setStencilReference(&mut self, reference: u32) {
        setStencilReference(&mut self.borrowState(), reference);
    }
    fn setBlendColor(&mut self, r: f32, g: f32, b: f32, a: f32) {
        setBlendColor(&mut self.borrowState(), r, g, b, a);
    }
    fn draw(&mut self, vertexCount: u32, instanceCount: u32, firstVertex: u32, firstInstance: u32) {
        draw(
            &mut self.borrowState(),
            vertexCount,
            instanceCount,
            firstVertex,
            firstInstance,
        );
    }
    fn drawIndexed(
        &mut self,
        indexCount: u32,
        instanceCount: u32,
        firstIndex: u32,
        baseVertex: i32,
        firstInstance: u32,
    ) {
        drawIndexed(
            &mut self.borrowState(),
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        );
    }
    fn finish(&mut self) {
        finish(&mut self.borrowState());
    }
    fn validate(&self) {
        validate(&self.inner.state.borrow());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn viewport_scissor_rounding_matches_floor_origin_and_ceil_far_edge() {
        let x = -0.25_f32;
        let width = 10.5_f32;
        let x0 = x.floor();
        let x1 = (x + width).ceil();
        assert_eq!(x0.max(0.0) as i32, 0);
        assert_eq!((x1 - x0).max(0.0) as u32, 12);
    }
}
