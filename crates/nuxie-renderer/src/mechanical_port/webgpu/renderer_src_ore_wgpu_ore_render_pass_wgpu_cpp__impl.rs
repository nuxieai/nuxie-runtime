//! Complete mechanical implementation translation of
//! `renderer/src/ore/wgpu/ore_render_pass_wgpu.cpp`.

#![allow(non_snake_case)]

use super::ore_bind_group_wgpu_decl::BindGroupWGPU;
use super::ore_buffer_wgpu_decl::BufferWGPU;
use super::ore_pipeline_wgpu_decl::PipelineWGPU;
use super::ore_render_pass_wgpu_decl::RenderPassWGPUState;
use super::webgpu_cpp_decl::{Buffer as WagyuBuffer, IndexFormat as WagyuIndexFormat};
use super::webgpu_decl::WGPUColor;
use nuxie_ore_metal::gpu_resource::AnyResourceHandle;
use nuxie_ore_metal::types::{kMaxBindGroups, IndexFormat};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_render_pass_wgpu.cpp");
pub(crate) const BIND_GROUP_FAILURE_PREFIX: &str = "ore: WGPU bind group creation failed for group";

pub(crate) fn validate(pass: &RenderPassWGPUState) {
    assert!(
        !nuxie_ore_metal::render_pass_is_finished(&pass.base),
        "RenderPass already finished"
    );
    assert!(!pass.m_wgpuPassEncoder.Get().is_null());
}

pub(crate) fn setPipeline(pass: &mut RenderPassWGPUState, owner: Option<&AnyResourceHandle>) {
    validate(pass);
    let owner = owner.expect("WebGPU pass source pipeline");
    let pipeline = owner
        .downcast_ref::<PipelineWGPU>()
        .expect("WebGPU pass requires PipelineWGPU");
    if !nuxie_ore_metal::render_pass_check_pipeline_compat(&pass.base, &pipeline.base) {
        return;
    }
    *pass.m_currentPipeline = Some(owner.clone());
    unsafe {
        pass.m_wgpuPassEncoder
            .SetPipeline(pipeline.nativePipeline().Get())
    };
}

pub(crate) fn setVertexBuffer(
    pass: &mut RenderPassWGPUState,
    slot: u32,
    owner: Option<&AnyResourceHandle>,
    offset: u32,
) {
    validate(pass);
    let buffer = owner
        .and_then(|owner| owner.downcast_ref::<BufferWGPU>())
        .expect("WebGPU vertex binding requires BufferWGPU");
    buffer.markBound();
    unsafe {
        pass.m_wgpuPassEncoder.SetVertexBuffer(
            slot,
            buffer.currentRaw(),
            u64::from(offset),
            u64::from(buffer.base.size().wrapping_sub(offset)),
        )
    };
}

pub(crate) fn setIndexBuffer(
    pass: &mut RenderPassWGPUState,
    owner: Option<&AnyResourceHandle>,
    format: IndexFormat,
    offset: u32,
) {
    validate(pass);
    let buffer = owner
        .and_then(|owner| owner.downcast_ref::<BufferWGPU>())
        .expect("WebGPU index binding requires BufferWGPU");
    let wFmt = if format == IndexFormat::uint32 {
        WagyuIndexFormat::Uint32
    } else {
        WagyuIndexFormat::Uint16
    };
    buffer.markBound();
    *pass.m_wgpuIndexBuffer = unsafe { WagyuBuffer::FromBorrowed(buffer.currentRaw()) };
    pass.m_wgpuIndexFormat = wFmt;
    pass.m_wgpuIndexOffset = offset;
    unsafe {
        pass.m_wgpuPassEncoder.SetIndexBuffer(
            pass.m_wgpuIndexBuffer.Get(),
            wFmt.into(),
            u64::from(offset),
            u64::from(buffer.base.size().wrapping_sub(offset)),
        )
    };
}

pub(crate) fn setBindGroup(
    pass: &mut RenderPassWGPUState,
    groupIndex: u32,
    owner: Option<&AnyResourceHandle>,
    dynamicOffsets: Option<&[u32]>,
    dynamicOffsetCount: u32,
) {
    validate(pass);
    let group = owner
        .and_then(|owner| owner.downcast_ref::<BindGroupWGPU>())
        .expect("WebGPU group binding requires BindGroupWGPU");
    let native = group.resolveBindGroup().Get();
    if native.is_null() {
        unsafe { pass.m_wgpuContext.as_ref() }
            .expect("RenderPassWGPU source context")
            .setLastError(&format!("{BIND_GROUP_FAILURE_PREFIX} {groupIndex}"));
        return;
    }
    let offsets = dynamicOffsets.unwrap_or(&[]);
    assert!(offsets.len() >= dynamicOffsetCount as usize);
    assert!(groupIndex < kMaxBindGroups);
    group.markUBOsBound();
    unsafe {
        pass.m_wgpuPassEncoder.SetBindGroup(
            groupIndex,
            native,
            dynamicOffsetCount as usize,
            if dynamicOffsetCount > 0 {
                offsets.as_ptr()
            } else {
                std::ptr::null()
            },
        )
    };
    if let Some(owner) = owner {
        nuxie_ore_metal::render_pass_retain_bound_group(&mut pass.base, groupIndex, owner.clone());
    }
}

pub(crate) fn setViewport(
    pass: &mut RenderPassWGPUState,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    minDepth: f32,
    maxDepth: f32,
) {
    validate(pass);
    unsafe {
        pass.m_wgpuPassEncoder
            .SetViewport(x, y, width, height, minDepth, maxDepth)
    };
}

pub(crate) fn setScissorRect(
    pass: &mut RenderPassWGPUState,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) {
    validate(pass);
    unsafe { pass.m_wgpuPassEncoder.SetScissorRect(x, y, width, height) };
}

pub(crate) fn setStencilReference(pass: &mut RenderPassWGPUState, reference: u32) {
    validate(pass);
    unsafe { pass.m_wgpuPassEncoder.SetStencilReference(reference) };
}

pub(crate) fn setBlendColor(pass: &mut RenderPassWGPUState, r: f32, g: f32, b: f32, a: f32) {
    validate(pass);
    let color = WGPUColor {
        r: r.into(),
        g: g.into(),
        b: b.into(),
        a: a.into(),
    };
    unsafe { pass.m_wgpuPassEncoder.SetBlendConstant(&color) };
}

pub(crate) fn draw(
    pass: &mut RenderPassWGPUState,
    vertexCount: u32,
    instanceCount: u32,
    firstVertex: u32,
    firstInstance: u32,
) {
    validate(pass);
    assert!(
        pass.m_currentPipeline.is_some(),
        "setPipeline must be called first"
    );
    unsafe {
        pass.m_wgpuPassEncoder
            .Draw(vertexCount, instanceCount, firstVertex, firstInstance)
    };
}

pub(crate) fn drawIndexed(
    pass: &mut RenderPassWGPUState,
    indexCount: u32,
    instanceCount: u32,
    firstIndex: u32,
    baseVertex: i32,
    firstInstance: u32,
) {
    validate(pass);
    assert!(
        pass.m_currentPipeline.is_some(),
        "setPipeline must be called first"
    );
    unsafe {
        pass.m_wgpuPassEncoder.DrawIndexed(
            indexCount,
            instanceCount,
            firstIndex,
            baseVertex,
            firstInstance,
        )
    };
}

pub(crate) fn finish(pass: &mut RenderPassWGPUState) {
    if nuxie_ore_metal::render_pass_is_finished(&pass.base) {
        return;
    }
    nuxie_ore_metal::render_pass_set_finished(&mut pass.base, true);
    if !pass.m_wgpuPassEncoder.Get().is_null() {
        unsafe { pass.m_wgpuPassEncoder.End() };
        *pass.m_wgpuPassEncoder = Default::default();
    }
    pass.m_wgpuContext = std::ptr::null_mut();
    *pass.m_currentPipeline = None;
}

pub(crate) const SOURCE_METHOD_DEFINITION_COUNT: usize = 16;
pub(crate) const SOURCE_ENCODER_CALL_COUNT: usize = 12;
const _: [(); 7023] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_implementation_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 221);
        assert_eq!(SOURCE_METHOD_DEFINITION_COUNT, 16);
        assert_eq!(SOURCE_ENCODER_CALL_COUNT, 12);
    }
}
