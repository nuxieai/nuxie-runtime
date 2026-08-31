//! renderer/ore/cmd/ore_replay.hpp at e949498e.
#![allow(non_snake_case)]
use super::{
    ore_command_buffer::{OreCommandBuffer, OreCommandReader},
    ore_commands::*,
    ore_handle::INVALID_HANDLE,
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

pub fn replayPassCommand(
    ctx: &mut dyn ContextApi,
    pass: &mut Option<Box<dyn RenderPassApi>>,
    dropDraws: &mut bool,
    kind: CommandType,
    reader: &mut OreCommandReader<'_>,
    resolve: &mut dyn FnMut(u32, OreKind) -> Option<AnyResourceHandle>,
) -> bool {
    fn churned(dropDraws: &mut bool, what: &str, h: u32) {
        *dropDraws = true;
        warn_throttled!(
            "rive ore replay: {} handle {} churned, dropping pass draws",
            what,
            h
        );
    }
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
            } else if let Some(pass) = pass {
                pass.setPipeline(p.as_ref());
            }
        }
        CommandType::setVertexBuffer => {
            let c: SetVertexBufferCmd = reader.read();
            let b = resolve(c.buffer, OreKind::buffer);
            if b.is_none() && c.buffer != INVALID_HANDLE {
                churned(dropDraws, "vertex buffer", c.buffer);
            } else if let Some(pass) = pass {
                pass.setVertexBuffer(c.slot, b.as_ref(), c.offset);
            }
        }
        CommandType::setIndexBuffer => {
            let c: SetIndexBufferCmd = reader.read();
            let b = resolve(c.buffer, OreKind::buffer);
            if b.is_none() && c.buffer != INVALID_HANDLE {
                churned(dropDraws, "index buffer", c.buffer);
            } else if let Some(pass) = pass {
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
            } else if let Some(pass) = pass {
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
            if let Some(pass) = pass {
                pass.setViewport(c.x, c.y, c.width, c.height, c.minDepth, c.maxDepth);
            }
        }
        CommandType::setScissorRect => {
            let c: SetScissorRectCmd = reader.read();
            if let Some(pass) = pass {
                pass.setScissorRect(c.x, c.y, c.width, c.height);
            }
        }
        CommandType::setStencilReference => {
            let c: SetStencilReferenceCmd = reader.read();
            if let Some(pass) = pass {
                pass.setStencilReference(c.reference);
            }
        }
        CommandType::setBlendColor => {
            let c: SetBlendColorCmd = reader.read();
            if let Some(pass) = pass {
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
