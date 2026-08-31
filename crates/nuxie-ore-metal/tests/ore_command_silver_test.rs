//! Upstream tests/unit_tests/renderer/ore_command_silver_test.cpp at e949498e.
use nuxie_ore_metal::{
    ore_cmd::{
        ore_command_buffer::*, ore_command_silver::*, ore_commands::*, ore_handle::*,
        ore_make_recording::encodePods,
    },
    types::*,
};
fn representative(buf: &mut OreCommandBuffer) {
    let mut begin = BeginRenderPassCmd {
        colorCount: 2,
        ..Default::default()
    };
    begin.colors[0] = ColorAttachmentPOD {
        view: 0,
        resolveTarget: INVALID_HANDLE,
        loadOp: LoadOp::clear,
        storeOp: StoreOp::store,
        clearR: 0.1,
        clearG: 0.2,
        clearB: 0.3,
        clearA: 1.,
    };
    begin.colors[1] = ColorAttachmentPOD {
        view: 1,
        resolveTarget: 2,
        loadOp: LoadOp::load,
        storeOp: StoreOp::discard,
        clearR: 0.,
        clearG: 0.,
        clearB: 0.,
        clearA: 0.,
    };
    begin.depthStencil = DepthStencilAttachmentPOD {
        view: 3,
        depthLoadOp: LoadOp::clear,
        depthStoreOp: StoreOp::store,
        depthClearValue: 1.,
        stencilLoadOp: LoadOp::clear,
        stencilStoreOp: StoreOp::store,
        stencilClearValue: 0,
    };
    buf.append(CommandType::beginRenderPass, &begin);
    buf.append(CommandType::setPipeline, &SetPipelineCmd { pipeline: 7 });
    buf.append(
        CommandType::setVertexBuffer,
        &SetVertexBufferCmd {
            slot: 0,
            buffer: 4,
            offset: 16,
        },
    );
    buf.append(
        CommandType::setIndexBuffer,
        &SetIndexBufferCmd {
            buffer: 5,
            format: IndexFormat::uint16,
            offset: 0,
        },
    );
    let start = buf.append_blob(&encodePods(&[64u32, 128]));
    buf.append(
        CommandType::setBindGroup,
        &SetBindGroupCmd {
            groupIndex: 1,
            bindGroup: 6,
            dynamicOffsetStart: start,
            dynamicOffsetCount: 2,
            pad: 0,
        },
    );
    buf.append(
        CommandType::setViewport,
        &SetViewportCmd {
            x: 0.,
            y: 0.,
            width: 256.,
            height: 128.,
            minDepth: 0.,
            maxDepth: 1.,
        },
    );
    buf.append(
        CommandType::setScissorRect,
        &SetScissorRectCmd {
            x: 0,
            y: 0,
            width: 256,
            height: 128,
        },
    );
    buf.append(
        CommandType::setStencilReference,
        &SetStencilReferenceCmd { reference: 0x80 },
    );
    buf.append(
        CommandType::setBlendColor,
        &SetBlendColorCmd {
            r: 1.,
            g: 0.5,
            b: 0.,
            a: 1.,
        },
    );
    buf.append(
        CommandType::draw,
        &DrawCmd {
            vertexCount: 6,
            instanceCount: 2,
            firstVertex: 1,
            firstInstance: 0,
        },
    );
    buf.append(
        CommandType::drawIndexed,
        &DrawIndexedCmd {
            indexCount: 12,
            instanceCount: 1,
            firstIndex: 0,
            baseVertex: -3,
            firstInstance: 0,
        },
    );
    buf.appendOpcode(CommandType::finish);
}
#[test]
fn ore_silver_round_trips_and_self_compares_equal() {
    let mut b = OreCommandBuffer::default();
    representative(&mut b);
    let mut s = vec![];
    serializeSilver(&b, &mut s);
    assert!(s.len() > kSilverMagic.len());
    let mut b2 = OreCommandBuffer::default();
    representative(&mut b2);
    let mut s2 = vec![];
    serializeSilver(&b2, &mut s2);
    assert_eq!(s, s2);
    assert!(silverMatch(&s, &s2));
}
#[test]
fn ore_silver_detects_a_diverging_field() {
    let mut e = OreCommandBuffer::default();
    representative(&mut e);
    let mut es = vec![];
    serializeSilver(&e, &mut es);
    let mut a = OreCommandBuffer::default();
    let mut begin = BeginRenderPassCmd {
        colorCount: 1,
        ..Default::default()
    };
    begin.colors[0] = ColorAttachmentPOD {
        view: 0,
        resolveTarget: INVALID_HANDLE,
        loadOp: LoadOp::clear,
        storeOp: StoreOp::store,
        clearR: 0.,
        clearG: 0.,
        clearB: 0.,
        clearA: 1.,
    };
    begin.depthStencil.view = INVALID_HANDLE;
    a.append(CommandType::beginRenderPass, &begin);
    a.append(
        CommandType::draw,
        &DrawCmd {
            vertexCount: 99,
            instanceCount: 1,
            firstVertex: 0,
            firstInstance: 0,
        },
    );
    let mut actual = vec![];
    serializeSilver(&a, &mut actual);
    assert!(!silverMatch(&es, &actual));
}
#[test]
fn ore_silver_tolerates_sub_epsilon_float_drift() {
    let mut a = OreCommandBuffer::default();
    a.append(
        CommandType::setBlendColor,
        &SetBlendColorCmd {
            r: 0.5,
            g: 0.,
            b: 0.,
            a: 1.,
        },
    );
    let mut sa = vec![];
    serializeSilver(&a, &mut sa);
    let mut b = OreCommandBuffer::default();
    b.append(
        CommandType::setBlendColor,
        &SetBlendColorCmd {
            r: 0.5 + kSilverEpsilon * 0.5,
            g: 0.,
            b: 0.,
            a: 1.,
        },
    );
    let mut sb = vec![];
    serializeSilver(&b, &mut sb);
    assert!(silverMatch(&sa, &sb));
}
