//! renderer/ore/cmd/ore_replay.hpp at 83acdadc.
#![allow(non_snake_case)]
use super::{
    ore_command_buffer::{OreCommandBuffer, OreCommandReader},
    ore_commands::*,
    ore_handle::{INVALID_HANDLE, REAL_RESOURCE_FLAG},
    ore_make_replay::{
        OreKind, OreResident, decodePods, replayOreLifecycle, resolveOre, warn_throttled,
    },
};
use crate::{
    context::{CanvasTextureInfo, ContextApi},
    gpu_resource::AnyResourceHandle,
    render_pass::RenderPassApi,
    types::*,
};

// Returns false for lifecycle opcodes. A failed handle poisons the open
// pass: skip pipeline-dependent state as well as draws until the next pass.
pub fn replayPassCommand(
    ctx: &mut dyn ContextApi,
    pass: &mut Option<Box<dyn RenderPassApi>>,
    dropDraws: &mut bool,
    kind: CommandType,
    reader: &mut OreCommandReader<'_>,
    resolve: &mut dyn FnMut(u32, OreKind) -> Option<AnyResourceHandle>,
    describe: Option<&dyn Fn(u32) -> Option<String>>,
) -> bool {
    let churned = |dropDraws: &mut bool, what: &str, h: u32| {
        *dropDraws = true;
        if let Some(note) = describe.and_then(|describe| describe(h)) {
            warn_throttled!(
                "rive ore replay: {} handle {} dropped pass draws: {}",
                what,
                h,
                note
            );
        } else {
            warn_throttled!(
                "rive ore replay: {} handle {} churned, dropping pass draws",
                what,
                h
            );
        }
    };
    match kind {
        CommandType::beginRenderPass => {
            let c: BeginRenderPassCmd = reader.read();
            let mut views: [Option<AnyResourceHandle>; 4] = Default::default();
            let mut targets: [Option<AnyResourceHandle>; 4] = Default::default();
            for i in 0..c.colorCount.min(4) as usize {
                views[i] = resolve(c.colors[i].view, OreKind::textureView);
                targets[i] = resolve(c.colors[i].resolveTarget, OreKind::textureView);
            }
            let depth = resolve(c.depthStencil.view, OreKind::textureView);
            let mut desc = RenderPassDesc {
                colorCount: c.colorCount,
                ..Default::default()
            };
            for i in 0..c.colorCount.min(4) as usize {
                let src = c.colors[i];
                desc.colorAttachments[i] = ColorAttachment {
                    view: views[i].as_ref(),
                    resolveTarget: targets[i].as_ref(),
                    loadOp: src.loadOp,
                    storeOp: src.storeOp,
                    clearColor: ClearColor {
                        r: src.clearR,
                        g: src.clearG,
                        b: src.clearB,
                        a: src.clearA,
                    },
                };
            }
            let ds = c.depthStencil;
            desc.depthStencil = DepthStencilAttachment {
                view: depth.as_ref(),
                depthLoadOp: ds.depthLoadOp,
                depthStoreOp: ds.depthStoreOp,
                depthClearValue: ds.depthClearValue,
                stencilLoadOp: ds.stencilLoadOp,
                stencilStoreOp: ds.stencilStoreOp,
                stencilClearValue: ds.stencilClearValue,
            };
            *dropDraws = false;
            for i in 0..c.colorCount.min(4) as usize {
                if views[i].is_none() && c.colors[i].view != INVALID_HANDLE {
                    churned(dropDraws, "render pass view", c.colors[i].view);
                    break;
                }
            }
            if !*dropDraws {
                *pass = ctx.beginRenderPass(&desc, None);
            }
        }
        CommandType::setPipeline => {
            let c: SetPipelineCmd = reader.read();
            let p = resolve(c.pipeline, OreKind::pipeline);
            if p.is_none() && c.pipeline != INVALID_HANDLE {
                churned(dropDraws, "pipeline", c.pipeline);
            } else if let Some(pass) = pass.as_mut().filter(|_| !*dropDraws) {
                pass.setPipeline(p.as_ref());
            }
        }
        CommandType::setVertexBuffer => {
            let c: SetVertexBufferCmd = reader.read();
            let b = resolve(c.buffer, OreKind::buffer);
            if b.is_none() && c.buffer != INVALID_HANDLE {
                churned(dropDraws, "vertex buffer", c.buffer);
            } else if let Some(pass) = pass.as_mut().filter(|_| !*dropDraws) {
                pass.setVertexBuffer(c.slot, b.as_ref(), c.offset);
            }
        }
        CommandType::setIndexBuffer => {
            let c: SetIndexBufferCmd = reader.read();
            let b = resolve(c.buffer, OreKind::buffer);
            if b.is_none() && c.buffer != INVALID_HANDLE {
                churned(dropDraws, "index buffer", c.buffer);
            } else if let Some(pass) = pass.as_mut().filter(|_| !*dropDraws) {
                pass.setIndexBuffer(b.as_ref(), c.format, c.offset);
            }
        }
        CommandType::setBindGroup => {
            let c: SetBindGroupCmd = reader.read();
            let offsets = if c.dynamicOffsetCount > 0 {
                Some(decodePods::<u32>(
                    reader.blob_at(c.dynamicOffsetStart, c.dynamicOffsetCount.wrapping_mul(4)),
                    c.dynamicOffsetCount,
                ))
            } else {
                None
            };
            let b = resolve(c.bindGroup, OreKind::bindGroup);
            if b.is_none() && c.bindGroup != INVALID_HANDLE {
                churned(dropDraws, "bind group", c.bindGroup);
            } else if let Some(pass) = pass.as_mut().filter(|_| !*dropDraws) {
                pass.setBindGroup(
                    c.groupIndex,
                    b.as_ref(),
                    offsets.as_deref(),
                    c.dynamicOffsetCount,
                );
            }
        }
        CommandType::setViewport => {
            let c: SetViewportCmd = reader.read();
            if let Some(pass) = pass.as_mut().filter(|_| !*dropDraws) {
                pass.setViewport(c.x, c.y, c.width, c.height, c.minDepth, c.maxDepth);
            }
        }
        CommandType::setScissorRect => {
            let c: SetScissorRectCmd = reader.read();
            if let Some(pass) = pass.as_mut().filter(|_| !*dropDraws) {
                pass.setScissorRect(c.x, c.y, c.width, c.height);
            }
        }
        CommandType::setStencilReference => {
            let c: SetStencilReferenceCmd = reader.read();
            if let Some(pass) = pass.as_mut().filter(|_| !*dropDraws) {
                pass.setStencilReference(c.reference);
            }
        }
        CommandType::setBlendColor => {
            let c: SetBlendColorCmd = reader.read();
            if let Some(pass) = pass.as_mut().filter(|_| !*dropDraws) {
                pass.setBlendColor(c.r, c.g, c.b, c.a);
            }
        }
        CommandType::draw => {
            let c: DrawCmd = reader.read();
            if !*dropDraws {
                if let Some(pass) = pass {
                    pass.draw(
                        c.vertexCount,
                        c.instanceCount,
                        c.firstVertex,
                        c.firstInstance,
                    );
                }
            }
        }
        CommandType::drawIndexed => {
            let c: DrawIndexedCmd = reader.read();
            if !*dropDraws {
                if let Some(pass) = pass {
                    pass.drawIndexed(
                        c.indexCount,
                        c.instanceCount,
                        c.firstIndex,
                        c.baseVertex,
                        c.firstInstance,
                    );
                }
            }
        }
        CommandType::finish => {
            if let Some(mut finished) = pass.take() {
                finished.finish();
            }
        }
        _ => return false,
    }
    true
}
pub fn replayCommandBufferResolved(
    ctx: &mut dyn ContextApi,
    commands: &[u8],
    blobs: &[u8],
    resolveHandle: &mut dyn FnMut(u32) -> Option<AnyResourceHandle>,
) {
    let mut reader = OreCommandReader::new(commands, blobs);
    let mut pass = None;
    let mut dropDraws = false;
    while let Some(kind) = reader.next::<CommandType>() {
        if !replayPassCommand(
            ctx,
            &mut pass,
            &mut dropDraws,
            kind,
            &mut reader,
            &mut |h, _| {
                if h == INVALID_HANDLE {
                    None
                } else {
                    resolveHandle(h)
                }
            },
            None,
        ) {
            debug_assert!(false, "lifecycle opcode in passes-only stream");
            break;
        }
    }
}
pub fn replayOreStream(
    ctx: &mut dyn ContextApi,
    commands: &[u8],
    blobs: &[u8],
    table: &mut OreResident,
    real: &mut dyn FnMut(u32) -> Option<AnyResourceHandle>,
    canvasAt: &mut dyn FnMut(u32) -> Option<CanvasTextureInfo>,
    imageAt: &mut dyn FnMut(u32) -> Option<CanvasTextureInfo>,
) {
    let mut reader = OreCommandReader::new(commands, blobs);
    let mut pass = None;
    let mut dropDraws = false;
    while let Some(kind) = reader.next::<CommandType>() {
        if !replayOreLifecycle(ctx, table, kind, &mut reader, real, canvasAt, imageAt)
            && !replayPassCommand(
                ctx,
                &mut pass,
                &mut dropDraws,
                kind,
                &mut reader,
                &mut |h, k| resolveOre(table, real, h, k),
                Some(&|h| {
                    if h & REAL_RESOURCE_FLAG != 0 {
                        None
                    } else {
                        table.failureNote(h).map(str::to_owned)
                    }
                }),
            )
        {
            debug_assert!(false, "unknown ORE opcode");
            break;
        }
    }
}
pub fn replayCommandBuffer(
    ctx: &mut dyn ContextApi,
    cmd: &OreCommandBuffer,
    mut remap: Option<&mut dyn FnMut(&AnyResourceHandle) -> Option<AnyResourceHandle>>,
) {
    let keep = cmd.keepAlive();
    replayCommandBufferResolved(ctx, cmd.command_bytes(), cmd.blob_bytes(), &mut |h| {
        let r = keep.get(h as usize)?;
        if let Some(remap) = &mut remap {
            remap(r)
        } else {
            Some(r.clone())
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{Context, FrameDescriptor, ShaderTarget};
    use crate::ore_cmd::ore_render_pass_recording::RenderPassRecording;
    use std::{cell::RefCell, rc::Rc};

    struct RecordingContext(Rc<RefCell<OreCommandBuffer>>);
    impl ContextApi for RecordingContext {
        fn contextBase(&self) -> &Context {
            unreachable!()
        }
        fn features(&self) -> Features {
            unreachable!()
        }
        fn lastError(&self) -> String {
            unreachable!()
        }
        fn clearLastError(&self) {
            unreachable!()
        }
        fn setLastError(&self, _: &str) {
            unreachable!()
        }
        fn makeBuffer(&mut self, _: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn makeTexture(&mut self, _: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn makeTextureViewImpl(&mut self, _: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn makeSampler(&mut self, _: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn makeShaderModule(&mut self, _: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn makeBindGroupLayout(
            &mut self,
            _: &BindGroupLayoutDesc<'_>,
        ) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn makePipeline(
            &mut self,
            _: &PipelineDesc<'_>,
            _: Option<&mut String>,
        ) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn makeBindGroup(&mut self, _: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn beginRenderPass(
            &mut self,
            desc: &RenderPassDesc<'_>,
            _: Option<&mut String>,
        ) -> Option<Box<dyn RenderPassApi>> {
            Some(Box::new(RenderPassRecording::new(
                None,
                self.0.clone(),
                desc,
            )))
        }
        fn beginFrame(&mut self, _: &FrameDescriptor) {
            unreachable!()
        }
        fn endFrame(&mut self) {
            unreachable!()
        }
        fn waitForGPU(&mut self) {
            unreachable!()
        }
        unsafe fn wrapCanvasTexture(
            &mut self,
            _: *mut core::ffi::c_void,
        ) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        unsafe fn wrapRiveTexture(
            &mut self,
            _: *mut core::ffi::c_void,
            _: u32,
            _: u32,
        ) -> Option<AnyResourceHandle> {
            unreachable!()
        }
        fn shaderTarget(&self) -> ShaderTarget {
            unreachable!()
        }
    }

    #[test]
    fn poisoned_pass_skips_all_state_but_resolves_handles_and_next_pass_recovers() {
        let mut commands = OreCommandBuffer::default();
        for poison in [true, false] {
            commands.append(
                CommandType::beginRenderPass,
                &BeginRenderPassCmd {
                    depthStencil: DepthStencilAttachmentPOD {
                        view: INVALID_HANDLE,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            );
            if poison {
                commands.append(CommandType::setPipeline, &SetPipelineCmd { pipeline: 7 });
            }
            commands.append(
                CommandType::setPipeline,
                &SetPipelineCmd {
                    pipeline: INVALID_HANDLE,
                },
            );
            commands.append(
                CommandType::setVertexBuffer,
                &SetVertexBufferCmd {
                    buffer: INVALID_HANDLE,
                    ..Default::default()
                },
            );
            commands.append(
                CommandType::setIndexBuffer,
                &SetIndexBufferCmd {
                    buffer: INVALID_HANDLE,
                    ..Default::default()
                },
            );
            commands.append(
                CommandType::setBindGroup,
                &SetBindGroupCmd {
                    bindGroup: INVALID_HANDLE,
                    ..Default::default()
                },
            );
            commands.append(CommandType::setViewport, &SetViewportCmd::default());
            commands.append(CommandType::setScissorRect, &SetScissorRectCmd::default());
            commands.append(
                CommandType::setStencilReference,
                &SetStencilReferenceCmd::default(),
            );
            commands.append(CommandType::setBlendColor, &SetBlendColorCmd::default());
            commands.append(CommandType::draw, &DrawCmd::default());
            commands.append(CommandType::drawIndexed, &DrawIndexedCmd::default());
            commands.appendOpcode(CommandType::finish);
        }
        let output = Rc::new(RefCell::new(OreCommandBuffer::default()));
        let mut context = RecordingContext(output.clone());
        let mut pass = None;
        let mut drop_draws = false;
        let mut resolved = Vec::new();
        let mut reader = OreCommandReader::new(commands.command_bytes(), commands.blob_bytes());
        while let Some(kind) = reader.next::<CommandType>() {
            assert!(replayPassCommand(
                &mut context,
                &mut pass,
                &mut drop_draws,
                kind,
                &mut reader,
                &mut |handle, _| {
                    resolved.push(handle);
                    None
                },
                None,
            ));
        }
        assert!(pass.is_none());
        assert!(!drop_draws);
        // Depth attachment resolution + all four state resource resolutions
        // still happen in each pass, including after the first handle churns.
        assert_eq!(
            resolved,
            [
                INVALID_HANDLE,
                7,
                INVALID_HANDLE,
                INVALID_HANDLE,
                INVALID_HANDLE,
                INVALID_HANDLE,
                INVALID_HANDLE,
                INVALID_HANDLE,
                INVALID_HANDLE,
                INVALID_HANDLE,
                INVALID_HANDLE
            ]
        );
        let output = output.borrow();
        let mut reader = OreCommandReader::new(output.command_bytes(), output.blob_bytes());
        let mut observed = Vec::new();
        while let Some(kind) = reader.next::<CommandType>() {
            observed.push(kind);
            reader.skip(ore_payload_size_of(kind));
        }
        assert_eq!(
            observed,
            [
                CommandType::beginRenderPass,
                CommandType::finish,
                CommandType::beginRenderPass,
                CommandType::setPipeline,
                CommandType::setVertexBuffer,
                CommandType::setIndexBuffer,
                CommandType::setBindGroup,
                CommandType::setViewport,
                CommandType::setScissorRect,
                CommandType::setStencilReference,
                CommandType::setBlendColor,
                CommandType::draw,
                CommandType::drawIndexed,
                CommandType::finish
            ]
        );
    }
}
