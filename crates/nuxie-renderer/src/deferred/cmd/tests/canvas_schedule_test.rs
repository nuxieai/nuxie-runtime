//! tests/unit_tests/renderer/canvas_schedule_test.cpp at e949498e.
use super::super::{
    canvas_schedule::*, command_stream::WirePod, deferred_replayer::*, deferred_session::*,
    render_commands::*, render_handle::*,
};
use super::*;
#[derive(Default)]
struct StreamBuilder {
    bytes: Vec<u8>,
    segments: Vec<DeferredSegment>,
}
impl StreamBuilder {
    fn append<P: WirePod>(&mut self, c: RenderCmd, p: &P) {
        self.bytes.push(c as u8);
        p.encode(&mut self.bytes);
    }
    fn canvas_range(&mut self, id: u64, samples: &[u64]) {
        let begin = self.bytes.len() as u32;
        self.append(RenderCmd::DrawPath, &DrawPathPod::default());
        for &sample in samples {
            self.append(
                RenderCmd::DrawImage,
                &DrawImagePod {
                    image: CANVAS_HANDLE_FLAG | sample as u32,
                    ..Default::default()
                },
            );
        }
        self.segments.push(DeferredSegment {
            target: SegmentTarget::Canvas,
            target_id: id,
            begin,
            end: self.bytes.len() as u32,
        });
    }
    fn schedule(&self) -> CanvasSchedule {
        schedule_canvases(&self.bytes, &self.segments)
    }
}
#[test]
fn in_order_sampler_keeps_record_order() {
    let mut b = StreamBuilder::default();
    b.canvas_range(1, &[]);
    b.canvas_range(2, &[1]);
    let s = b.schedule();
    assert_eq!(s.order, [1, 2]);
    assert!(!s.had_cycle);
    assert!(!s.multi_write_fallback);
}
#[test]
fn reader_recorded_before_writer_reorders() {
    let mut b = StreamBuilder::default();
    b.canvas_range(2, &[1]);
    b.canvas_range(1, &[]);
    let s = b.schedule();
    assert_eq!(s.order, [1, 2]);
    assert!(!s.had_cycle);
}
#[test]
fn reversed_three_canvas_chain() {
    let mut b = StreamBuilder::default();
    b.canvas_range(3, &[2]);
    b.canvas_range(2, &[1]);
    b.canvas_range(1, &[]);
    assert_eq!(b.schedule().order, [1, 2, 3]);
}
#[test]
fn cycle_demotes_to_record_order() {
    let mut b = StreamBuilder::default();
    b.canvas_range(1, &[2]);
    b.canvas_range(2, &[1]);
    let s = b.schedule();
    assert_eq!(s.order, [1, 2]);
    assert!(s.had_cycle);
}
#[test]
fn self_sample_is_demoted_edge() {
    let mut b = StreamBuilder::default();
    b.canvas_range(1, &[1]);
    let s = b.schedule();
    assert_eq!(s.order, [1]);
    assert!(s.had_cycle);
}
#[test]
fn unwritten_id_adds_no_edge() {
    let mut b = StreamBuilder::default();
    let begin = b.bytes.len() as u32;
    b.append(
        RenderCmd::DrawImage,
        &DrawImagePod {
            image: CANVAS_HANDLE_FLAG | 7,
            ..Default::default()
        },
    );
    b.segments.push(DeferredSegment {
        target: SegmentTarget::Canvas,
        target_id: 1,
        begin,
        end: b.bytes.len() as u32,
    });
    b.canvas_range(2, &[]);
    let s = b.schedule();
    assert_eq!(s.order, [1, 2]);
    assert!(!s.had_cycle);
}
#[test]
fn read_between_two_writes_falls_back() {
    let mut b = StreamBuilder::default();
    b.canvas_range(1, &[]);
    b.canvas_range(2, &[1]);
    b.canvas_range(1, &[]);
    let s = b.schedule();
    assert_eq!(s.order, [1, 2]);
    assert!(s.multi_write_fallback);
}
#[test]
fn image_mesh_creates_edges() {
    let mut b = StreamBuilder::default();
    let begin = b.bytes.len() as u32;
    b.append(
        RenderCmd::DrawImageMesh,
        &DrawImageMeshPod {
            image: CANVAS_HANDLE_FLAG | 1,
            ..Default::default()
        },
    );
    b.segments.push(DeferredSegment {
        target: SegmentTarget::Canvas,
        target_id: 2,
        begin,
        end: b.bytes.len() as u32,
    });
    b.canvas_range(1, &[]);
    assert_eq!(b.schedule().order, [1, 2]);
}
#[test]
fn independent_canvases_keep_record_order() {
    let mut b = StreamBuilder::default();
    b.canvas_range(3, &[]);
    b.canvas_range(1, &[5]);
    b.canvas_range(4, &[]);
    b.canvas_range(5, &[]);
    assert_eq!(b.schedule().order, [3, 4, 5, 1]);
}
#[test]
fn replay_opens_sampled_canvas_before_reader() {
    let mut session = DeferredSession::new(None);
    let a = fake_canvas();
    let b = fake_canvas();
    let mut rb = session.begin_canvas_content(b.clone(), 0).unwrap();
    rb.draw_image(
        Some(a.borrow().render_image().as_ref()),
        ImageSampler::default(),
        BlendMode::SrcOver,
        1.0,
    );
    session.end_canvas_content(&b);
    let mut ra = session.begin_canvas_content(a.clone(), 0).unwrap();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();
    ra.draw_path(path.as_ref(), paint.as_ref());
    session.end_canvas_content(&a);
    session.close_open_range();
    let frame = snapshot_frame(&mut session);
    let mut sink = TestSink::default();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(
        sink.opened_canvases,
        [Rc::as_ptr(&a) as usize, Rc::as_ptr(&b) as usize]
    );
}
#[test]
fn canvas_only_frame_opens_screen() {
    let mut session = DeferredSession::new(None);
    let canvas = fake_canvas();
    let mut r = session.begin_canvas_content(canvas.clone(), 0).unwrap();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();
    r.draw_path(path.as_ref(), paint.as_ref());
    session.end_canvas_content(&canvas);
    session.close_open_range();
    let frame = snapshot_frame(&mut session);
    let mut sink = TestSink::default();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(sink.opened_canvases.len(), 1);
    assert_eq!(sink.screens.len(), 1);
}
#[test]
fn canvas_callbacks_can_reborrow_the_replay_factory() {
    struct BorrowingSink {
        factory: PersistentFactory<RecordingFactory>,
        opened_canvases: usize,
        ended_canvases: usize,
        opened_screens: usize,
    }
    impl DeferredFrameSink for BorrowingSink {
        fn factory(&mut self) -> PersistentFactoryContext {
            self.factory.persistent_context().unwrap()
        }
        fn ore_context(&mut self) -> Option<OreContextHandle> {
            None
        }
        fn begin_canvas_content(
            &mut self,
            canvas: RenderCanvasHandle,
            _clear_color: u32,
        ) -> Option<RendererOwner> {
            // Like the source sink, frame setup accesses the same factory
            // used for resource replay; no replay-wide RefMut may remain.
            let mut factory = self.factory.borrow_mut();
            let canvas = canvas.borrow();
            factory.frame_size(canvas.width(), canvas.height());
            factory.add_frame();
            self.opened_canvases += 1;
            Some(Rc::new(RefCell::new(Box::new(factory.make_renderer()))))
        }
        fn end_canvas_content(&mut self) {
            let _factory = self.factory.borrow_mut();
            self.ended_canvases += 1;
        }
        fn begin_screen_frame(&mut self, _target: u64) -> Option<RendererOwner> {
            let mut factory = self.factory.borrow_mut();
            factory.add_frame();
            self.opened_screens += 1;
            Some(Rc::new(RefCell::new(Box::new(factory.make_renderer()))))
        }
    }

    let mut session = DeferredSession::new(None);
    let canvas = fake_canvas();
    let mut renderer = session.begin_canvas_content(canvas.clone(), 0).unwrap();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();
    renderer.draw_path(path.as_ref(), paint.as_ref());
    session.end_canvas_content(&canvas);
    let frame = snapshot_frame(&mut session);
    let mut sink = BorrowingSink {
        factory: PersistentFactory::new(RecordingFactory::new()),
        opened_canvases: 0,
        ended_canvases: 0,
        opened_screens: 0,
    };
    let mut replayer = DeferredReplayer::default();
    replayer.replay_frame(&frame, &mut sink);
    assert_eq!(sink.opened_canvases, 1);
    assert_eq!(sink.ended_canvases, 1);
    assert_eq!(sink.opened_screens, 1);
    assert_eq!(replayer.dropped_draws(), 0);
    assert_eq!(
        sink.factory.borrow().stream().matches("drawPath ").count(),
        1
    );
}
#[test]
fn resource_only_frame_opens_no_screen() {
    let mut session = DeferredSession::new(None);
    let _paint = session.make_render_paint();
    let _path = session.make_empty_render_path();
    session.close_open_range();
    let frame = snapshot_frame(&mut session);
    assert!(!frame.commands.is_empty());
    let mut sink = TestSink::default();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert!(sink.screens.is_empty());
}
