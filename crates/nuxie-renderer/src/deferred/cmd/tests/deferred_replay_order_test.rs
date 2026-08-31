//! tests/unit_tests/renderer/deferred_replay_order_test.cpp at e949498e.
use super::super::{
    deferred_render_factory::*, deferred_render_resource::DeferredRenderPath, deferred_replayer::*,
    deferred_session::*, render_commands::*, render_handle::*,
};
use super::*;
#[derive(Default)]
struct CountingFactory {
    inner: SerializingFactory,
    paint_count: u32,
}
impl Factory for CountingFactory {
    fn make_render_buffer(
        &mut self,
        t: RenderBufferType,
        f: RenderBufferFlags,
        s: usize,
    ) -> Box<dyn RenderBuffer> {
        self.inner.make_render_buffer(t, f, s)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        c: &[u32],
        s: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner.make_linear_gradient(sx, sy, ex, ey, c, s)
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        r: f32,
        c: &[u32],
        s: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner.make_radial_gradient(cx, cy, r, c, s)
    }
    fn make_render_path(&mut self, p: RawPath, f: FillRule) -> Box<dyn RenderPath> {
        self.inner.make_render_path(p, f)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.inner.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.paint_count += 1;
        self.inner.make_render_paint()
    }
    fn decode_image(&mut self, b: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.inner.decode_image(b)
    }
}
struct CountingSink {
    factory: PersistentFactory<CountingFactory>,
}
impl Default for CountingSink {
    fn default() -> Self {
        Self {
            factory: PersistentFactory::new(CountingFactory::default()),
        }
    }
}
impl DeferredFrameSink for CountingSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.factory.persistent_context().unwrap()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, _: u64) -> Option<RendererOwner> {
        let mut factory = self.factory.borrow_mut();
        factory.inner.frame_size(256, 256);
        factory.inner.add_frame();
        Some(Rc::new(RefCell::new(Box::new(
            factory.inner.make_renderer(),
        ))))
    }
}
fn frame(factory: &DeferredFactory, begin: u32, end: u32, canvas: u64) -> DeferredFrame {
    let b = factory.buffer.lock().unwrap();
    DeferredFrame {
        commands: b.command_bytes().to_vec(),
        blobs: b.blob_bytes().to_vec(),
        segments: vec![
            DeferredSegment {
                target: SegmentTarget::Screen,
                target_id: 0,
                begin: 0,
                end: begin,
            },
            DeferredSegment {
                target: SegmentTarget::Canvas,
                target_id: canvas,
                begin,
                end,
            },
            DeferredSegment {
                target: SegmentTarget::Screen,
                target_id: 0,
                begin: end,
                end: b.command_bytes().len() as u32,
            },
        ],
        ..Default::default()
    }
}
#[test]
fn create_inside_canvas_replays_mint_order() {
    let mut f = DeferredFactory::new();
    let mut renderer = f.make_renderer(None);
    let paint = f.make_render_paint();
    let p1 = f.make_empty_render_path();
    let begin = f.buffer.lock().unwrap().command_bytes().len() as u32;
    let canvas = 7 | CANVAS_HANDLE_FLAG;
    f.buffer.lock().unwrap().append(
        RenderCmd::CanvasContentBegin,
        &CanvasContentPod {
            canvas_id: canvas,
            clear_color: 0xff000000,
        },
    );
    let p2 = f.make_empty_render_path();
    renderer.draw_path(p2.as_ref(), paint.as_ref());
    f.buffer
        .lock()
        .unwrap()
        .append(RenderCmd::CanvasContentEnd, &ResIdPod { id: canvas });
    let end = f.buffer.lock().unwrap().command_bytes().len() as u32;
    renderer.draw_path(p1.as_ref(), paint.as_ref());
    renderer.draw_path(p2.as_ref(), paint.as_ref());
    let frame = frame(&f, begin, end, 7);
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&frame, &mut TestSink::default());
    assert_eq!(replayer.dropped_draws(), 0);
}
#[test]
fn interleaved_ranges_split_per_renderer() {
    let mut s = DeferredSession::new(None);
    let screen = s.screen_renderer(0);
    let mut c1 = DeferredRenderer::new(
        s.command_buffer(),
        Some(s.canvases()),
        Some(s.routing.clone()),
        1,
    );
    let mut c2 = DeferredRenderer::new(
        s.command_buffer(),
        Some(s.canvases()),
        Some(s.routing.clone()),
        2,
    );
    let paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    c1.draw_path(path.as_ref(), paint.as_ref());
    c2.draw_path(path.as_ref(), paint.as_ref());
    c1.draw_path(path.as_ref(), paint.as_ref());
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    s.close_open_range();
    let segments: Vec<_> = s
        .recorded_segments()
        .into_iter()
        .filter(|s| s.target == SegmentTarget::Canvas)
        .collect();
    assert_eq!(segments.len(), 3);
    assert_eq!(
        segments.iter().map(|s| s.target_id).collect::<Vec<_>>(),
        [1, 2, 1]
    );
    for pair in segments.windows(2) {
        assert!(pair[1].begin >= pair[0].end);
    }
    let frame = snapshot_frame(&mut s);
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&frame, &mut TestSink::default());
    assert_eq!(replayer.dropped_draws(), 0);
}
#[test]
fn decoded_image_view_records_image_view_wrap() {
    use nuxie_ore_metal::context::ContextApi;
    use nuxie_ore_metal::ore_cmd::{
        ore_command_buffer::OreCommandReader,
        ore_commands::{CommandType, WrapCanvasViewMode, WrapCanvasViewPOD},
    };
    let s = DeferredSession::new(None);
    let view = s.ore_context.borrow_mut().recordWrapImageView(42, 64, 64);
    assert!(view.is_some());
    let stream = s.ore_context.borrow().stream();
    let stream = stream.borrow();
    let mut reader = OreCommandReader::new(stream.command_bytes(), stream.blob_bytes());
    assert_eq!(
        reader.next::<CommandType>(),
        Some(CommandType::wrapCanvasView)
    );
    let pod: WrapCanvasViewPOD = reader.read();
    assert_eq!(pod.canvasId, 42);
    assert_eq!(pod.mode, WrapCanvasViewMode::imageView as u32);
}
#[test]
fn destroy_in_screen_gap_does_not_starve_canvas() {
    let mut f = DeferredFactory::new();
    let mut renderer = f.make_renderer(None);
    let paint = f.make_render_paint();
    let p1 = f.make_empty_render_path();
    renderer.draw_path(p1.as_ref(), paint.as_ref());
    assert_ne!(
        DeferredRenderPath::id_of_path(p1.as_ref()),
        INVALID_RENDER_HANDLE
    );
    drop(p1);
    f.buffer.lock().unwrap().drain_destroys();
    let begin = f.buffer.lock().unwrap().command_bytes().len() as u32;
    let canvas = 3 | CANVAS_HANDLE_FLAG;
    f.buffer.lock().unwrap().append(
        RenderCmd::CanvasContentBegin,
        &CanvasContentPod {
            canvas_id: canvas,
            clear_color: 0xff000000,
        },
    );
    let p2 = f.make_empty_render_path();
    renderer.draw_path(p2.as_ref(), paint.as_ref());
    f.buffer
        .lock()
        .unwrap()
        .append(RenderCmd::CanvasContentEnd, &ResIdPod { id: canvas });
    let end = f.buffer.lock().unwrap().command_bytes().len() as u32;
    renderer.draw_path(p2.as_ref(), paint.as_ref());
    let frame = frame(&f, begin, end, 3);
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&frame, &mut TestSink::default());
    assert_eq!(replayer.dropped_draws(), 0);
}
#[test]
fn mutation_after_draw_keeps_version() {
    let mut s = DeferredSession::new(None);
    let screen = s.screen_renderer(0);
    let mut canvas = DeferredRenderer::new(
        s.command_buffer(),
        Some(s.canvases()),
        Some(s.routing.clone()),
        1,
    );
    let mut paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    paint.color(0xffff0000);
    canvas.draw_path(path.as_ref(), paint.as_ref());
    paint.color(0xff0000ff);
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    s.close_open_range();
    let frame = snapshot_frame(&mut s);
    let mut sink = CountingSink::default();
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&frame, &mut sink);
    assert_eq!(replayer.dropped_draws(), 0);
    assert_eq!(sink.factory.borrow().paint_count, 2);
}
#[test]
fn mutation_before_draw_stays_one_object() {
    let mut s = DeferredSession::new(None);
    let screen = s.screen_renderer(0);
    let mut paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    paint.color(0xff00ff00);
    paint.thickness(2.0);
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
    s.close_open_range();
    let frame = snapshot_frame(&mut s);
    let mut sink = CountingSink::default();
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&frame, &mut sink);
    assert_eq!(replayer.dropped_draws(), 0);
    assert_eq!(sink.factory.borrow().paint_count, 1);
}
#[test]
fn first_mutation_new_frame_reuses_live_object() {
    let mut s = DeferredSession::new(None);
    let screen = s.screen_renderer(0);
    let mut paint = s.make_render_paint();
    let path = s.make_empty_render_path();
    let mut sink = CountingSink::default();
    let mut replayer = DeferredReplayer::default();
    for color in [0xffff0000, 0xff00ff00, 0xff0000ff] {
        paint.color(color);
        screen.borrow_mut().draw_path(path.as_ref(), paint.as_ref());
        let frame = snapshot_frame(&mut s);
        s.reset_frame();
        replayer.replay_frame(&frame, &mut sink);
        assert_eq!(replayer.dropped_draws(), 0);
    }
    assert_eq!(sink.factory.borrow().paint_count, 1);
}
