//! Android draws every frame in MSAA (`RenderMode::Msaa`: depth-stencil at 4
//! samples, inkbleed off). There a one-pass stroke is one `depthStrokes` batch
//! (upstream draw.cpp's PathDraw draw type), and an inside stroke, which is how
//! a CSS box's border reaches the renderer, is a clip by a centered stroke of
//! twice the width (`depthStrokes` with `clipUpdate`) and then a fill of the
//! path inside that clip (upstream rive_renderer.cpp drawPath). Read from the
//! flush's own draw list on a NULL device, so it runs on any host.
//!
//! `vulkan::msaa_strokes_pipeline_test` hands these batches to the Vulkan
//! pipeline manager.

use super::hairline_batch_test::{
    CapturedBatch, CapturedCounts, CapturedFlush, CapturingRenderContext,
};
use crate::mechanical_port::source::include::rive::{
    refcnt_hpp::make_rcp, renderer_hpp::RendererContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::{
    gpu_hpp::{DrawContents, DrawType, InterlockMode},
    render_context_hpp::{FlushResources, FrameDescriptor, RenderContext},
    render_target_hpp::RenderTarget,
    rive_render_paint_hpp::RiveRenderPaint,
    rive_renderer_hpp::RiveRenderer,
};
use nuxie_render_api::{RenderPaintStyle, StrokePosition};
use std::{cell::RefCell, pin::Pin, rc::Rc};

const TARGET_SIZE: u32 = 256;
pub(super) const STROKE_COLOR: u32 = 0xff33_6699;
pub(super) const BORDER_COLOR: u32 = 0xff99_3366;

/// A 4px centered stroke around (10, 10)-(100, 100) and a 6px inside stroke,
/// a CSS border, around (130, 10)-(220, 100), on Android's frame shape. Returns
/// the one flush and every batch it carried.
pub(super) fn android_stroke_frame(
    supports_pipeline_dynamic_state: bool,
) -> (CapturedFlush, Vec<CapturedBatch>) {
    let counts = Rc::new(RefCell::new(CapturedCounts::default()));
    let mut context = RenderContext::from_impl(Box::new(CapturingRenderContext::new(
        supports_pipeline_dynamic_state,
        counts.clone(),
    )));
    // The context stays pinned and outlives the renderer and the draws.
    let ctx = unsafe { Pin::get_unchecked_mut(context.as_mut()) };
    ctx.beginFrameExecutable(&FrameDescriptor {
        renderTargetWidth: TARGET_SIZE,
        renderTargetHeight: TARGET_SIZE,
        msaaSampleCount: 4,
        ..Default::default()
    });
    assert_eq!(ctx.frameInterlockMode(), InterlockMode::depthStencil);
    let mut renderer = unsafe { RiveRenderer::new(ctx) };
    for (left, color, thickness, position) in [
        (10.0, STROKE_COLOR, 4.0, StrokePosition::Center),
        (130.0, BORDER_COLOR, 6.0, StrokePosition::Inside),
    ] {
        let paint = ctx.makeRenderPaint();
        // RenderContext's factory creates a RiveRenderPaint with the base at
        // offset zero; use its concrete implementation.
        let concrete = unsafe { &mut *paint.get().cast::<RiveRenderPaint>() };
        concrete.color(color);
        concrete.style(RenderPaintStyle::Stroke);
        concrete.thickness(thickness);
        concrete.strokePosition(position);
        let path = ctx.makeEmptyRenderPath();
        unsafe {
            let p = &mut (*path.get()).base;
            p.moveTo(left, 10.0);
            p.lineTo(left + 90.0, 10.0);
            p.lineTo(left + 90.0, 100.0);
            p.lineTo(left, 100.0);
            p.close();
            renderer.drawPath(path.get(), paint.get());
        }
    }
    let target = make_rcp(|| RenderTarget::new(TARGET_SIZE, TARGET_SIZE));
    unsafe {
        ctx.flushExecutable(&FlushResources {
            renderTarget: target.get(),
            ..Default::default()
        });
    }
    let counts = counts.borrow();
    assert_eq!(counts.flushes.len(), 1, "one flush: {:?}", counts.flushes);
    (counts.flushes[0], counts.flushed.clone())
}

pub(super) fn is_stroke(batch: &CapturedBatch) -> bool {
    matches!(
        batch.draw_type,
        DrawType::depthStrokes | DrawType::depthAAStrokes
    ) && batch.ranges > 0
}

pub(super) fn has(batch: &CapturedBatch, contents: DrawContents) -> bool {
    batch.draw_contents.0 & contents.0 != 0
}

fn is_fill(batch: &CapturedBatch) -> bool {
    matches!(
        batch.draw_type,
        DrawType::stencilMidpointFanBorrowedCoverage
            | DrawType::stencilMidpointFans
            | DrawType::stencilMidpointFanReset
            | DrawType::stencilDynamicMidpointFans
            | DrawType::stencilOuterCubicBorrowedCoverage
            | DrawType::stencilOuterCubics
            | DrawType::stencilOuterCubicReset
            | DrawType::stencilDynamicOuterCubics
            | DrawType::stencilMidpointFanWinding
            | DrawType::stencilMidpointFanCover
            | DrawType::stencilOuterCubicWinding
            | DrawType::stencilOuterCubicCover
    ) && batch.ranges > 0
}

#[test]
fn an_msaa_frame_draws_a_stroke_and_an_inside_border_as_depth_strokes() {
    for supports_pipeline_dynamic_state in [true, false] {
        let (flush, batches) = android_stroke_frame(supports_pipeline_dynamic_state);
        let what = format!("dynamic state {supports_pipeline_dynamic_state}");
        eprintln!("{what}: {flush:?}");
        for batch in &batches {
            eprintln!("{what}:   {batch:?}");
        }
        assert_eq!(flush.interlock_mode, InterlockMode::depthStencil, "{what}");
        assert_eq!(flush.msaa_sample_count, 4, "{what}");
        // The centered stroke draws color as one depthStrokes batch.
        let plain = batches
            .iter()
            .position(|b| is_stroke(b) && !has(b, DrawContents::clipUpdate));
        assert!(
            plain.is_some(),
            "{what}: no colored depthStrokes batch: {batches:?}"
        );
        // The border's clip is a depthStrokes batch that updates the clip.
        let clip = batches
            .iter()
            .position(|b| is_stroke(b) && has(b, DrawContents::clipUpdate));
        let Some(clip) = clip else {
            panic!("{what}: no clip-updating depthStrokes batch: {batches:?}");
        };
        // The border's color is a fill inside that clip, drawn after it.
        let fill = batches
            .iter()
            .enumerate()
            .skip(clip + 1)
            .find(|(_, b)| is_fill(b) && has(b, DrawContents::activeClip));
        assert!(
            fill.is_some(),
            "{what}: no fill inside the border's clip after it: {batches:?}"
        );
    }
}
