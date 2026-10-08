//! Canvas mode cases from tests/unit_tests/renderer/deferred_inline_host_test.cpp
//! at 10b048fba81b6218101ae6fda7cdc03e110a7b32. Real source frame selection on
//! the null device; no GPU or substituted interlock-mode calculation.
use super::render_context_null::*;
use super::*;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::InterlockMode;

struct CanvasModeSink {
    factory: ObservingFactory,
    mode: RenderCanvasFrameMode,
}

impl CanvasModeSink {
    fn new() -> Self {
        Self {
            factory: observing_factory(64, 64).0,
            mode: RenderCanvasFrameMode {
                clockwise_fill_override: true,
                ..Default::default()
            },
        }
    }
}

impl DeferredFrameSink for CanvasModeSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.factory.persistent_context().unwrap()
    }

    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.factory.persistent_context()
    }

    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }

    fn begin_screen_frame(&mut self, _: u64) -> Option<RendererOwner> {
        None
    }

    fn frame_mode(&self) -> RenderCanvasFrameMode {
        self.mode
    }
}

struct MSAACanvasSink(CanvasModeSink);

impl DeferredFrameSink for MSAACanvasSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.0.factory()
    }

    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.0.render_context()
    }

    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }

    fn begin_screen_frame(&mut self, _: u64) -> Option<RendererOwner> {
        None
    }

    fn frame_mode(&self) -> RenderCanvasFrameMode {
        self.0.frame_mode()
    }

    fn canvas_mode(&self, _: &RenderCanvasHandle) -> RenderCanvasFrameMode {
        RenderCanvasFrameMode {
            msaa_sample_count: 4,
            ..Default::default()
        }
    }
}

// HostFrameSink::beginCanvasContent's Rust ownership seam: keep the actual
// canvas frame until the corresponding endCanvasContent consumes it.
fn begin_canvas_content(sink: &mut impl DeferredFrameSink) -> Box<dyn RenderCanvasFrame> {
    let canvas: RenderCanvasHandle = Rc::new(RefCell::new(
        sink.factory()
            .with_factory(|factory| factory.make_deferred_render_canvas(64, 64))
            .unwrap(),
    ));
    sink.render_context()
        .unwrap()
        .with_factory(|factory| factory.ensure_canvas_backing(&canvas));
    let mode = sink.canvas_mode(&canvas);
    let frame = canvas.borrow_mut().begin_frame_with_mode(0, mode).unwrap();
    frame
}

#[test]
fn a_canvas_frame_opens_in_the_screens_mode() {
    let mut sink = CanvasModeSink::new();
    let frame = begin_canvas_content(&mut sink);
    sink.factory.borrow().with_backend_mut_for_test(|backend| {
        let descriptor = backend.frame_descriptor();
        assert!(descriptor.clockwiseFillOverride);
        assert_eq!(descriptor.msaaSampleCount, 0);
        assert_eq!(backend.frame_interlock_mode(), InterlockMode::clockwise);
    });
    frame.finish().unwrap();
}

#[test]
fn a_sink_can_open_a_canvas_frame_in_its_own_mode() {
    let mut sink = MSAACanvasSink(CanvasModeSink::new());
    let frame = begin_canvas_content(&mut sink);
    sink.0
        .factory
        .borrow()
        .with_backend_mut_for_test(|backend| {
            let descriptor = backend.frame_descriptor();
            assert!(!descriptor.clockwiseFillOverride);
            assert_eq!(descriptor.msaaSampleCount, 4);
            assert_eq!(backend.frame_interlock_mode(), InterlockMode::depthStencil);
        });
    frame.finish().unwrap();
}

// 625454e3: TestingWindow forwards inkbleed independently; RenderContext
// accepts it only for single-sample depth/stencil frames.
#[test]
fn canvas_inkbleed_follows_the_single_sample_frame_policy() {
    for samples in [0, 1, 4] {
        let mut sink = CanvasModeSink::new();
        sink.mode.msaa_sample_count = samples;
        sink.mode.inkbleed = true;
        let frame = begin_canvas_content(&mut sink);
        sink.factory.borrow().with_backend_mut_for_test(|backend| {
            let descriptor = backend.frame_descriptor();
            assert_eq!(descriptor.inkbleedOverride, samples == 1);
            assert_eq!(descriptor.msaaSampleCount, samples);
            assert!(descriptor.clockwiseFillOverride);
        });
        frame.finish().unwrap();
    }
    let mode = crate::RenderMode::ClockwiseInkbleed.canvas_frame_mode();
    assert_eq!(mode.msaa_sample_count, 1);
    assert!(mode.clockwise_fill_override);
    assert!(mode.inkbleed);
    assert!(!mode.disable_raster_ordering);
}
