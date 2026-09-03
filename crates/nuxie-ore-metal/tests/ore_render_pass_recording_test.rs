//! Upstream tests/unit_tests/renderer/ore_render_pass_recording_test.cpp at 966499ff.
use nuxie_ore_metal::{
    ore_cmd::{
        ore_command_buffer::*, ore_command_silver::*, ore_commands::*, ore_handle::*,
        ore_render_pass_recording::RenderPassRecording,
    },
    render_pass::RenderPassApi,
    types::*,
};
use std::{cell::RefCell, rc::Rc};
#[test]
fn render_pass_recording_emits_the_expected_command_stream() {
    let mut desc = RenderPassDesc::default();
    desc.colorCount = 1;
    desc.colorAttachments[0].loadOp = LoadOp::clear;
    desc.colorAttachments[0].storeOp = StoreOp::store;
    desc.colorAttachments[0].clearColor = ClearColor {
        r: 0.25,
        g: 0.5,
        b: 0.75,
        a: 1.,
    };
    let recorded = Rc::new(RefCell::new(OreCommandBuffer::default()));
    {
        let mut pass = RenderPassRecording::new(None, recorded.clone(), &desc);
        pass.setPipeline(None);
        pass.setViewport(0., 0., 128., 64., 0., 1.);
        pass.setScissorRect(0, 0, 128, 64);
        pass.draw(6, 1, 0, 0);
        pass.finish();
    }
    let mut expected = OreCommandBuffer::default();
    let mut begin = BeginRenderPassCmd {
        colorCount: 1,
        ..Default::default()
    };
    begin.colors[0] = ColorAttachmentPOD {
        view: INVALID_HANDLE,
        resolveTarget: INVALID_HANDLE,
        clearR: 0.25,
        clearG: 0.5,
        clearB: 0.75,
        clearA: 1.,
        loadOp: LoadOp::clear,
        storeOp: StoreOp::store,
        pad: [0; 2],
    };
    begin.depthStencil = DepthStencilAttachmentPOD {
        view: INVALID_HANDLE,
        depthClearValue: 1.,
        stencilClearValue: 0,
        depthLoadOp: LoadOp::clear,
        depthStoreOp: StoreOp::store,
        stencilLoadOp: LoadOp::clear,
        stencilStoreOp: StoreOp::discard,
    };
    expected.append(CommandType::beginRenderPass, &begin);
    expected.append(
        CommandType::setPipeline,
        &SetPipelineCmd {
            pipeline: INVALID_HANDLE,
        },
    );
    expected.append(
        CommandType::setViewport,
        &SetViewportCmd {
            x: 0.,
            y: 0.,
            width: 128.,
            height: 64.,
            minDepth: 0.,
            maxDepth: 1.,
        },
    );
    expected.append(
        CommandType::setScissorRect,
        &SetScissorRectCmd {
            x: 0,
            y: 0,
            width: 128,
            height: 64,
        },
    );
    expected.append(
        CommandType::draw,
        &DrawCmd {
            vertexCount: 6,
            instanceCount: 1,
            firstVertex: 0,
            firstInstance: 0,
        },
    );
    expected.appendOpcode(CommandType::finish);
    let mut rs = vec![];
    let mut es = vec![];
    serializeSilver(&recorded.borrow(), &mut rs);
    serializeSilver(&expected, &mut es);
    assert!(silverMatch(&es, &rs));
}
#[test]
fn render_pass_recording_finish_is_idempotent() {
    let desc = RenderPassDesc {
        colorCount: 0,
        ..Default::default()
    };
    let recorded = Rc::new(RefCell::new(OreCommandBuffer::default()));
    let mut pass = RenderPassRecording::new(None, recorded.clone(), &desc);
    pass.draw(3, 1, 0, 0);
    pass.finish();
    assert!(pass.isFinished());
    pass.finish();
    let recorded = recorded.borrow();
    let mut r = OreCommandReader::new(recorded.command_bytes(), recorded.blob_bytes());
    let (mut finishes, mut total) = (0, 0);
    while let Some(t) = r.next::<CommandType>() {
        total += 1;
        match t {
            CommandType::finish => finishes += 1,
            CommandType::beginRenderPass => {
                r.read::<BeginRenderPassCmd>();
            }
            CommandType::draw => {
                r.read::<DrawCmd>();
            }
            _ => panic!("unexpected command in stream"),
        }
    }
    assert_eq!(finishes, 1);
    assert_eq!(total, 3);
}
