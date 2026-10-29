//! Lifetime assertions from upstream deferred_worker_handoff_test.cpp at 7a73c835.
//!
//! Approved Rust adaptation: deferred frames, canvas/image handles, and device
//! resources remain thread-affine (57dddb37). Replay and final frame release run
//! on their owning thread here. These tests cover the three hand-off lifetimes,
//! not Emscripten worker dispatch or cross-thread replay, and grant no Send impl.
use super::super::{
    deferred_replayer::{DeferredFrame, DeferredFrameSink, DeferredReplayer, take_frame},
    deferred_session::{DeferredSession, DeferredSessionAttachment},
    render_replay::RendererOwner,
};
use super::TestSink;
use super::render_context_null::{ObservingFactory, observing_factory};
use crate::exact_source_adapter::ExactSourceRenderCanvas;
use crate::mechanical_port::source::{
    include::rive::refcnt_hpp::{rcp, ref_rcp},
    renderer::include::rive::renderer::render_target_hpp::RenderTarget,
};
use nuxie_render_api::{
    DeferredCanvasHost, Factory, OreContextHandle, PersistentFactoryContext, RenderCanvasHandle,
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

struct ReplayOwnerSink {
    base: TestSink,
    context: ObservingFactory,
}
impl ReplayOwnerSink {
    fn new(context: ObservingFactory) -> Self {
        Self {
            base: TestSink::default(),
            context,
        }
    }
}
impl DeferredFrameSink for ReplayOwnerSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.base.factory()
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.context.persistent_context()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        self.base.begin_screen_frame(target)
    }
    fn begin_canvas_content(&mut self, _: RenderCanvasHandle, _: u32) -> Option<RendererOwner> {
        None
    }
}
fn record_canvas_draw(session: &mut DeferredSession, canvas: &RenderCanvasHandle) {
    let mut content = DeferredCanvasHost::begin_canvas_content(session, canvas.clone(), 0).unwrap();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();
    content.draw_path(path.as_ref(), paint.as_ref());
    DeferredCanvasHost::end_canvas_content(session, canvas);
    session.close_open_range();
}
fn replay_on_owner(
    replayer: &mut DeferredReplayer,
    frame: DeferredFrame,
    sink: &mut dyn DeferredFrameSink,
) {
    replayer.replay_frame(&frame, sink);
    // Match the upstream helper's frame.reset() before returning to the host.
    drop(frame);
}
fn retain_backing(canvas: &RenderCanvasHandle) -> rcp<RenderTarget> {
    let mut canvas = canvas.borrow_mut();
    let canvas = (&mut **canvas as &mut dyn Any)
        .downcast_mut::<ExactSourceRenderCanvas>()
        .unwrap();
    // The owning canvas retains its non-null target throughout this borrow;
    // ref_rcp gives the test its own retain before the canvas can be dropped.
    unsafe { ref_rcp((&mut *canvas.source_ptr()).renderTarget()) }
}
fn backing_refs(backing: &rcp<RenderTarget>) -> i32 {
    assert!(!backing.get().is_null());
    // The rcp keeps its target and intrusive reference counter alive.
    unsafe { (&*backing.get()).base.debugging_refcnt() }
}
#[derive(Default)]
struct LeftoverHost {
    attached: Cell<bool>,
    leftover: RefCell<Option<DeferredFrame>>,
}
impl LeftoverHost {
    fn join(self: &Rc<Self>, session: &DeferredSession) {
        self.attached.set(true);
        let attachment: Rc<dyn DeferredSessionAttachment> = self.clone();
        session.attach(Rc::downgrade(&attachment));
    }
}
impl DeferredSessionAttachment for LeftoverHost {
    fn deferred_session_destroyed(&self, session: &mut DeferredSession) {
        *self.leftover.borrow_mut() = Some(take_frame(session));
        self.attached.set(false);
    }
}

#[test]
fn a_taken_frame_replays_on_its_owner_after_its_session_is_gone() {
    let context = observing_factory(8, 8).0;
    let canvas: RenderCanvasHandle =
        Rc::new(RefCell::new(Box::new(ExactSourceRenderCanvas::new(8, 8))));
    let frame = {
        let mut session = DeferredSession::with_caps(Default::default());
        record_canvas_draw(&mut session, &canvas);
        let paint = session.make_render_paint();
        let path = session.make_empty_render_path();
        session
            .screen_renderer(0)
            .borrow_mut()
            .draw_path(path.as_ref(), paint.as_ref());
        take_frame(&mut session)
    };
    let mut sink = ReplayOwnerSink::new(context);
    let mut replayer = DeferredReplayer::default();
    replay_on_owner(&mut replayer, frame, &mut sink);
    assert!(canvas.borrow().is_backed());
    assert_eq!(sink.base.screens.len(), 1);
}

#[test]
fn a_canvas_dropped_while_recording_frees_its_backing_with_the_next_frame() {
    let context = observing_factory(8, 8).0;
    let mut session = DeferredSession::with_caps(Default::default());
    session.bind_render_context(context.persistent_context());
    let mut sink = ReplayOwnerSink::new(context);
    let mut replayer = DeferredReplayer::default();
    let canvas = session.make_content_canvas(8, 8).unwrap();
    record_canvas_draw(&mut session, &canvas);
    replay_on_owner(&mut replayer, take_frame(&mut session), &mut sink);
    assert!(canvas.borrow().is_backed());
    let backing = retain_backing(&canvas);
    drop(canvas);
    assert!(backing_refs(&backing) > 1);
    let next = take_frame(&mut session);
    assert!(!next.retired_canvas_backings.is_empty());
    replay_on_owner(&mut replayer, next, &mut sink);
    assert_eq!(backing_refs(&backing), 1);
}

#[test]
fn a_dying_session_hands_its_unreplayed_frame_to_the_host() {
    let context = observing_factory(8, 8).0;
    let mut sink = ReplayOwnerSink::new(context.clone());
    let mut replayer = DeferredReplayer::default();
    let host = Rc::new(LeftoverHost::default());
    let retired;
    {
        let mut session = DeferredSession::with_caps(Default::default());
        session.bind_render_context(context.persistent_context());
        host.join(&session);
        let dropped = session.make_content_canvas(8, 8).unwrap();
        record_canvas_draw(&mut session, &dropped);
        replay_on_owner(&mut replayer, take_frame(&mut session), &mut sink);
        assert!(dropped.borrow().is_backed());
        retired = retain_backing(&dropped);
        drop(dropped);
        // Recorded, never replayed. The session owns the pending canvas when
        // the external handle below is dropped before session destruction.
        let pending = session.make_content_canvas(8, 8).unwrap();
        record_canvas_draw(&mut session, &pending);
    }
    assert!(host.leftover.borrow().is_some());
    assert!(!host.attached.get());
    assert_eq!(
        host.leftover
            .borrow()
            .as_ref()
            .unwrap()
            .content_canvases
            .len(),
        1
    );
    assert!(backing_refs(&retired) > 1);
    let frame = host.leftover.borrow_mut().take().unwrap();
    replay_on_owner(&mut replayer, frame, &mut sink);
    assert_eq!(backing_refs(&retired), 1);
}
