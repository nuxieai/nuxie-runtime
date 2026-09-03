//! Upstream tests/unit_tests/renderer/ore_command_buffer_test.cpp at e949498e.
use nuxie_ore_metal::{
    cmd::command_stream::CommandReader,
    ore_cmd::{ore_command_buffer::*, ore_commands::*, ore_handle::*},
    types::*,
};
#[test]
fn ore_command_stream_round_trips_through_the_reader() {
    let mut buf = OreCommandBuffer::default();
    let mut begin = BeginRenderPassCmd {
        colorCount: 1,
        ..Default::default()
    };
    begin.colors[0] = ColorAttachmentPOD {
        view: 0,
        resolveTarget: INVALID_HANDLE,
        clearR: 0.25,
        clearG: 0.5,
        clearB: 0.75,
        clearA: 1.,
        loadOp: LoadOp::clear,
        storeOp: StoreOp::store,
        pad: [0; 2],
    };
    begin.depthStencil.view = INVALID_HANDLE;
    buf.append(CommandType::beginRenderPass, &begin);
    buf.append(CommandType::setPipeline, &SetPipelineCmd { pipeline: 7 });
    buf.append(
        CommandType::setVertexBuffer,
        &SetVertexBufferCmd {
            slot: 0,
            buffer: 3,
            offset: 16,
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
    buf.appendOpcode(CommandType::finish);
    let mut r = OreCommandReader::new(buf.command_bytes(), buf.blob_bytes());
    assert_eq!(r.next(), Some(CommandType::beginRenderPass));
    let b: BeginRenderPassCmd = r.read();
    assert_eq!(b.colorCount, 1);
    assert_eq!(b.colors[0].view, 0);
    assert_eq!(b.colors[0].resolveTarget, INVALID_HANDLE);
    assert_eq!(b.colors[0].loadOp, LoadOp::clear);
    assert_eq!(b.colors[0].clearR, 0.25);
    assert_eq!(b.colors[0].clearB, 0.75);
    assert_eq!(b.depthStencil.view, INVALID_HANDLE);
    assert_eq!(r.next(), Some(CommandType::setPipeline));
    assert_eq!(r.read::<SetPipelineCmd>().pipeline, 7);
    assert_eq!(r.next(), Some(CommandType::setVertexBuffer));
    let vb: SetVertexBufferCmd = r.read();
    assert_eq!(vb.slot, 0);
    assert_eq!(vb.buffer, 3);
    assert_eq!(vb.offset, 16);
    assert_eq!(r.next(), Some(CommandType::draw));
    let d: DrawCmd = r.read();
    assert_eq!(d.vertexCount, 6);
    assert_eq!(d.instanceCount, 2);
    assert_eq!(d.firstVertex, 1);
    assert_eq!(r.next(), Some(CommandType::finish));
    assert!(r.next::<CommandType>().is_none());
}
#[test]
fn ore_command_buffer_reset_keeps_the_buffer_reusable() {
    let mut buf = OreCommandBuffer::default();
    buf.append(
        CommandType::draw,
        &DrawCmd {
            vertexCount: 1,
            instanceCount: 1,
            firstVertex: 0,
            firstInstance: 0,
        },
    );
    assert!(!buf.empty());
    buf.reset();
    assert!(buf.empty());
    assert!(buf.keepAlive().is_empty());
    buf.appendOpcode(CommandType::finish);
    assert!(!buf.empty());
}
#[test]
fn ore_command_buffer_capture_maps_nullptr_to_invalid_handle() {
    let mut buf = OreCommandBuffer::default();
    assert_eq!(buf.capture(None), INVALID_HANDLE);
    assert!(buf.keepAlive().is_empty());
}
#[test]
fn a_truncated_trailing_opcode_latches_overrun() {
    let bytes = [1, 0, 0];
    let mut truncated = CommandReader::new(&bytes, &[]);
    assert!(truncated.next::<u32>().is_none());
    assert!(truncated.overrun());
    let mut clean = CommandReader::new(&[], &[]);
    assert!(clean.next::<u32>().is_none());
    assert!(!clean.overrun());
}
