//! Attachment cases through upstream 115c4862, plus the shared Rust
//! session-handle lifetime regression. Weak references replace raw pointers.
use super::super::{
    deferred_replayer::{DeferredFrameSink, DeferredReplayer, snapshot_frame},
    deferred_session::{DeferredSession, DeferredSessionAttachment},
    render_replay::RendererOwner,
};
use super::TestSink;
use nuxie_render_api::{Factory, OreContextHandle, PersistentFactoryContext, RenderCanvasHandle};
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

struct TestHost {
    session: RefCell<Option<Weak<RefCell<DeferredSession>>>>,
    target: Cell<u64>,
    notices: Cell<u32>,
    identity: RefCell<Option<Weak<dyn DeferredSessionAttachment>>>,
}
impl TestHost {
    fn new() -> Rc<Self> {
        let host = Rc::new(Self {
            session: RefCell::new(None),
            target: Cell::new(0),
            notices: Cell::new(0),
            identity: RefCell::new(None),
        });
        let attachment: Rc<dyn DeferredSessionAttachment> = host.clone();
        *host.identity.borrow_mut() = Some(Rc::downgrade(&attachment));
        host
    }
    fn join(&self, session: &Rc<RefCell<DeferredSession>>) {
        *self.session.borrow_mut() = Some(Rc::downgrade(session));
        self.target
            .set(session.borrow_mut().acquire_screen_target());
        session
            .borrow()
            .attach(self.identity.borrow().as_ref().unwrap().clone());
    }
    fn screen_renderer(&self) -> Option<RendererOwner> {
        let session = self.session.borrow().as_ref()?.upgrade()?;
        let renderer = session.borrow().screen_renderer(self.target.get());
        Some(renderer)
    }
}
impl DeferredSessionAttachment for TestHost {
    fn deferred_session_destroyed(&self) {
        *self.session.borrow_mut() = None;
        self.target.set(0);
        self.notices.set(self.notices.get() + 1);
    }
}
impl Drop for TestHost {
    fn drop(&mut self) {
        if let Some(session) = self.session.get_mut().as_ref().and_then(Weak::upgrade) {
            let mut session = session.borrow_mut();
            session.detach(self.identity.get_mut().as_ref().unwrap());
            session.release_screen_target(self.target.get());
        }
    }
}
fn session() -> Rc<RefCell<DeferredSession>> {
    Rc::new(RefCell::new(DeferredSession::with_caps(Default::default())))
}

#[test]
fn a_dying_session_clears_every_attached_host() {
    let session = session();
    let a = TestHost::new();
    let b = TestHost::new();
    a.join(&session);
    b.join(&session);
    assert_eq!(session.borrow().attachment_count(), 2);
    assert_eq!(session.borrow().attached_target_count(), 2);
    assert_ne!(a.target.get(), b.target.get());
    assert!(a.screen_renderer().is_some());
    assert!(b.screen_renderer().is_some());
    drop(session);
    assert!(a.session.borrow().is_none());
    assert!(b.session.borrow().is_none());
    assert_eq!(a.notices.get(), 1);
    assert_eq!(b.notices.get(), 1);
    assert!(a.screen_renderer().is_none());
    assert!(b.screen_renderer().is_none());
}

#[test]
fn a_host_that_dies_first_leaves_the_session_nothing_to_notify() {
    let session = session();
    {
        let a = TestHost::new();
        a.join(&session);
        assert_eq!(session.borrow().attachment_count(), 1);
        assert_eq!(session.borrow().attached_target_count(), 1);
    }
    assert_eq!(session.borrow().attachment_count(), 0);
    assert_eq!(session.borrow().attached_target_count(), 0);
}

#[test]
fn attach_is_idempotent_and_a_strangers_detach_is_a_no_op() {
    let session = session();
    let a = TestHost::new();
    let stranger = TestHost::new();
    a.join(&session);
    session
        .borrow()
        .attach(a.identity.borrow().as_ref().unwrap().clone());
    assert_eq!(session.borrow().attachment_count(), 1);
    session
        .borrow()
        .detach(stranger.identity.borrow().as_ref().unwrap());
    assert_eq!(session.borrow().attachment_count(), 1);
}

#[test]
fn a_host_outliving_its_session_attaches_a_successor_cleanly() {
    let a = TestHost::new();
    let first = session();
    a.join(&first);
    drop(first);
    assert!(a.session.borrow().is_none());
    let second = session();
    a.join(&second);
    assert!(
        a.session
            .borrow()
            .as_ref()
            .unwrap()
            .ptr_eq(&Rc::downgrade(&second))
    );
    assert_eq!(second.borrow().attachment_count(), 1);
    assert_eq!(second.borrow().attached_target_count(), 1);
    assert!(a.screen_renderer().is_some());
}

#[test]
fn cloned_session_handles_notify_only_on_the_last_drop() {
    let first = DeferredSession::with_caps(Default::default());
    let second = first.clone();
    let host = TestHost::new();
    first.attach(host.identity.borrow().as_ref().unwrap().clone());
    assert_eq!(second.attachment_count(), 1);
    drop(first);
    assert_eq!(host.notices.get(), 0);
    assert_eq!(second.attachment_count(), 1);
    drop(second);
    assert_eq!(host.notices.get(), 1);
}

#[test]
fn abandoning_every_open_frame_lets_the_next_target_close_the_window() {
    let mut session = DeferredSession::with_caps(Default::default());
    session.begin_target_frame(0);
    session.begin_target_frame(1);
    assert_eq!(session.open_target_count(), 2);
    session.abandon_open_target_frames();
    assert_eq!(session.open_target_count(), 0);
    session.begin_target_frame(2);
    assert!(session.end_target_frame(2));
}

#[derive(Default)]
struct TargetSink {
    inner: TestSink,
    opened: Vec<u64>,
}
impl DeferredFrameSink for TargetSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.inner.factory()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        self.inner.ore_context()
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        self.opened.push(target);
        self.inner.begin_screen_frame(target)
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        clear: u32,
    ) -> Option<RendererOwner> {
        self.inner.begin_canvas_content(canvas, clear)
    }
}

#[test]
fn an_abandoned_frames_draws_never_reach_the_next_targets_frame() {
    for abandon_all in [false, true] {
        let mut session = DeferredSession::with_caps(Default::default());
        let a = session.acquire_screen_target();
        let b = session.acquire_screen_target();
        let paint = session.make_render_paint();
        let path = session.make_empty_render_path();

        session.begin_target_frame(a);
        session
            .screen_renderer(a)
            .borrow_mut()
            .draw_path(path.as_ref(), paint.as_ref());
        if abandon_all {
            session.abandon_open_target_frames();
        } else {
            session.abandon_target_frame(a);
        }
        session.release_screen_target(a);

        session.begin_target_frame(b);
        session
            .screen_renderer(b)
            .borrow_mut()
            .draw_path(path.as_ref(), paint.as_ref());
        assert!(session.end_target_frame(b));
        session.close_open_range();

        let frame = snapshot_frame(&mut session);
        let mut sink = TargetSink::default();
        DeferredReplayer::default().replay_frame(&frame, &mut sink);
        assert_eq!(sink.opened, vec![b]);
    }
}

#[test]
fn a_leaving_targets_finished_frame_never_replays_under_its_id() {
    let mut session = DeferredSession::with_caps(Default::default());
    let a = session.acquire_screen_target();
    let b = session.acquire_screen_target();
    let paint = session.make_render_paint();
    let path = session.make_empty_render_path();

    session.begin_target_frame(a);
    session.begin_target_frame(b);
    session
        .screen_renderer(a)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    assert!(!session.end_target_frame(a));
    session.discard_target_frame(a);
    session.release_screen_target(a);
    assert_eq!(session.acquire_screen_target(), a);

    session
        .screen_renderer(b)
        .borrow_mut()
        .draw_path(path.as_ref(), paint.as_ref());
    assert!(session.end_target_frame(b));
    session.close_open_range();

    let frame = snapshot_frame(&mut session);
    let mut sink = TargetSink::default();
    DeferredReplayer::default().replay_frame(&frame, &mut sink);
    assert_eq!(sink.opened, vec![b]);
}
