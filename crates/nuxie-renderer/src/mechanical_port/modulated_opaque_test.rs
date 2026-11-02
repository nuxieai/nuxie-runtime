//! `opaquePaint` promises the draw emits alpha 1: the depth-stencil prepass and
//! the blend table trust it. Read from the flush's own draw list on a NULL
//! device, against the rule's arithmetic: the flag is on exactly when the
//! paint's own color is opaque and round(255 x color alpha x opacity) is 255.
//!
//! Upstream's modulate_opacity_test.cpp and modulate_color_test.cpp check colors
//! and gradients, never this flag or a depth-stencil frame.

use super::hairline_batch_test::{CapturedBatch, CapturedCounts, CapturingRenderContext};
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
use nuxie_render_api::RenderPaintStyle;
use std::{cell::RefCell, pin::Pin, rc::Rc};

const TARGET_SIZE: u32 = 256;
const OPAQUE: u32 = 0xff33_6699;

#[derive(Clone, Copy, Debug)]
enum Source {
    Solid(u32),
    Gradient,
    Stroke,
}

struct Case {
    what: &'static str,
    source: Source,
    scope: Option<f32>,
    color: Option<u32>,
    flag_after: bool,
}

const CASES: [Case; 10] = [
    Case {
        what: "opaque fill, no scope: 255",
        source: Source::Solid(OPAQUE),
        scope: None,
        color: None,
        flag_after: true,
    },
    Case {
        what: "opaque fill, scope 1: 255",
        source: Source::Solid(OPAQUE),
        scope: Some(1.0),
        color: None,
        flag_after: true,
    },
    Case {
        what: "opaque fill, scope 0.999: 254.7 rounds to 255",
        source: Source::Solid(OPAQUE),
        scope: Some(0.999),
        color: None,
        flag_after: true,
    },
    Case {
        what: "opaque fill, scope 0.998: 254.49 rounds to 254",
        source: Source::Solid(OPAQUE),
        scope: Some(0.998),
        color: None,
        flag_after: false,
    },
    Case {
        what: "opaque fill, scope 0.5: 127.5 rounds to 128",
        source: Source::Solid(OPAQUE),
        scope: Some(0.5),
        color: None,
        flag_after: false,
    },
    Case {
        what: "opaque fill, modulateColor(0x80ffffff) at scope 1: 128",
        source: Source::Solid(OPAQUE),
        scope: Some(1.0),
        color: Some(0x80ff_ffff),
        flag_after: false,
    },
    Case {
        what: "opaque fill, modulateColor(0xff808080), RGB only, at scope 1: 255",
        source: Source::Solid(OPAQUE),
        scope: Some(1.0),
        color: Some(0xff80_8080),
        flag_after: true,
    },
    Case {
        what: "linear gradient, every stop opaque, scope 0.5: 128",
        source: Source::Gradient,
        scope: Some(0.5),
        color: None,
        flag_after: false,
    },
    Case {
        what: "opaque stroke, scope 0.5: 128",
        source: Source::Stroke,
        scope: Some(0.5),
        color: None,
        flag_after: false,
    },
    Case {
        what: "translucent fill 0x80336699, no scope: 128",
        source: Source::Solid(0x8033_6699),
        scope: None,
        color: None,
        flag_after: false,
    },
];

/// Draws the case alone on Android's frame shape, depth-stencil at 4 samples
/// with inkbleed off, and returns what the flush carried.
fn flushed(supports_pipeline_dynamic_state: bool, case: &Case) -> Vec<CapturedBatch> {
    let counts = Rc::new(RefCell::new(CapturedCounts::default()));
    let mut context = RenderContext::from_impl(Box::new(CapturingRenderContext::new(
        supports_pipeline_dynamic_state,
        counts.clone(),
    )));
    // The context stays pinned and outlives the renderer and the draw.
    let ctx = unsafe { Pin::get_unchecked_mut(context.as_mut()) };
    ctx.beginFrameExecutable(&FrameDescriptor {
        renderTargetWidth: TARGET_SIZE,
        renderTargetHeight: TARGET_SIZE,
        msaaSampleCount: 4,
        ..Default::default()
    });
    assert_eq!(ctx.frameInterlockMode(), InterlockMode::depthStencil);
    let paint = ctx.makeRenderPaint();
    // RenderContext's factory creates a RiveRenderPaint with the base at offset
    // zero; use its concrete implementation.
    let concrete = unsafe { &mut *paint.get().cast::<RiveRenderPaint>() };
    match case.source {
        Source::Solid(color) => concrete.color(color),
        Source::Gradient => {
            let shader = ctx.makeLinearGradient(
                10.0,
                10.0,
                100.0,
                10.0,
                &[0xff33_6699, 0xff99_6633],
                &[0.0, 1.0],
            );
            // SAFETY: makeLinearGradient's shader is a Gradient at offset zero.
            unsafe { concrete.shader_source(shader) };
        }
        Source::Stroke => {
            concrete.color(OPAQUE);
            concrete.style(RenderPaintStyle::Stroke);
            concrete.thickness(4.0);
        }
    }
    let mut renderer = unsafe { RiveRenderer::new(ctx) };
    let path = ctx.makeEmptyRenderPath();
    unsafe {
        let p = &mut (*path.get()).base;
        p.moveTo(10.0, 10.0);
        p.lineTo(100.0, 10.0);
        p.lineTo(100.0, 100.0);
        p.lineTo(10.0, 100.0);
        p.close();
        renderer.save();
        if let Some(opacity) = case.scope {
            renderer.modulateOpacity(opacity);
        }
        if let Some(color) = case.color {
            renderer.modulateColor(color, false);
        }
        renderer.drawPath(path.get(), paint.get());
        renderer.restore();
    }
    let target = make_rcp(|| RenderTarget::new(TARGET_SIZE, TARGET_SIZE));
    unsafe {
        ctx.flushExecutable(&FlushResources {
            renderTarget: target.get(),
            ..Default::default()
        });
    }
    let batches = counts.borrow().flushed.clone();
    batches
}

fn carries_opaque(batch: &CapturedBatch) -> bool {
    batch.draw_contents.0 & DrawContents::opaquePaint.0 != 0
}

/// A draw type a path of this source draws as on this frame.
fn is_path_batch(source: Source, draw_type: DrawType) -> bool {
    match source {
        Source::Solid(_) | Source::Gradient => matches!(
            draw_type,
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
        ),
        Source::Stroke => draw_type == DrawType::depthStrokes,
    }
}

#[test]
fn a_scoped_paint_is_flagged_opaque_only_when_it_emits_alpha_255() {
    let mut wrong = Vec::new();
    for supports_pipeline_dynamic_state in [true, false] {
        for case in &CASES {
            let batches = flushed(supports_pipeline_dynamic_state, case);
            let what = format!(
                "dynamic state {supports_pipeline_dynamic_state}, {}",
                case.what
            );
            // A build that drops the draw must not pass a "flag off" row.
            let paths: Vec<_> = batches
                .iter()
                .filter(|batch| is_path_batch(case.source, batch.draw_type) && batch.ranges > 0)
                .collect();
            assert!(
                !paths.is_empty(),
                "{what}: flushed no {:?} path batch with an element range: {batches:?}",
                case.source
            );
            let flag = if case.flag_after {
                paths.iter().any(|batch| carries_opaque(batch))
            } else {
                batches.iter().any(|batch| {
                    !matches!(
                        batch.draw_type,
                        DrawType::renderPassInitialize | DrawType::renderPassResolve
                    ) && carries_opaque(batch)
                })
            };
            eprintln!("{what}: flag {flag}, expected {}", case.flag_after);
            if flag != case.flag_after {
                wrong.push(format!("{what}: flag {flag}, expected {}", case.flag_after));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}
