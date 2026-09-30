//! renderer/include/rive/renderer/cmd/deferred_canvas.hpp at 57dddb3727306e284773ec20c653cf686c45abee.
use std::any::Any;
use std::sync::Mutex;

/// Retained until the snapshot carrying it is dropped by its replay owner.
pub struct RetiredCanvasBacking {
    // Erase the concrete intrusive owners at the optional-backend boundary.
    // Each box holds the original rcp, not a native handle or a cloned backing.
    pub texture: Option<Box<dyn Any>>,
    pub render_target: Option<Box<dyn Any>>,
}
impl Drop for RetiredCanvasBacking {
    fn drop(&mut self) {
        // C++ member destruction is reverse declaration order.
        drop(self.render_target.take());
        drop(self.texture.take());
    }
}

#[derive(Default)]
struct RetirerState {
    retired: Vec<RetiredCanvasBacking>,
    closed: bool,
}

/// Shared by the session and its canvases, including canvases outliving it.
/// Existing Rust frame/image handles remain thread-affine; synchronization does
/// not grant Send to otherwise non-Send resource owners.
#[derive(Default)]
pub struct CanvasRetirer {
    state: Mutex<RetirerState>,
}

impl CanvasRetirer {
    pub fn retire(&self, texture: Option<Box<dyn Any>>, render_target: Option<Box<dyn Any>>) {
        if texture.is_none() && render_target.is_none() {
            return;
        }
        let mut state = self.state.lock().unwrap();
        if state.closed {
            return;
        }
        state.retired.push(RetiredCanvasBacking {
            texture,
            render_target,
        });
    }

    pub fn take(&self) -> Vec<RetiredCanvasBacking> {
        std::mem::take(&mut self.state.lock().unwrap().retired)
    }

    pub fn close(&self) {
        let mut state = self.state.lock().unwrap();
        state.closed = true;
        state.retired.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::Cell, rc::Rc, sync::Arc};

    struct DropCount(Rc<Cell<usize>>);
    impl Drop for DropCount {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }

    #[test]
    fn canvas_retirer_take_and_close_preserve_lifetime() {
        let retirer = Arc::new(CanvasRetirer::default());
        let shared = retirer.clone();
        let drops = Rc::new(Cell::new(0));
        retirer.retire(None, None);
        assert!(retirer.take().is_empty());
        retirer.retire(Some(Box::new(DropCount(drops.clone()))), None);
        let taken = shared.take();
        assert_eq!(taken.len(), 1);
        assert!(retirer.take().is_empty());
        assert_eq!(drops.get(), 0);
        drop(taken);
        assert_eq!(drops.get(), 1);
        shared.retire(None, Some(Box::new(DropCount(drops.clone()))));
        retirer.close();
        assert_eq!(drops.get(), 2);
        shared.retire(Some(Box::new(DropCount(drops.clone()))), None);
        assert_eq!(drops.get(), 3);
        assert!(shared.take().is_empty());
    }

    #[cfg(any(
        feature = "native-vulkan-experimental",
        feature = "renderer-vulkan",
        feature = "renderer-webgpu",
        feature = "renderer-webgl2",
        feature = "renderer-metal"
    ))]
    mod source_lifecycle {
        use super::*;
        use crate::deferred::cmd::{
            deferred_replayer::snapshot_frame, deferred_session::DeferredSession,
        };
        use crate::mechanical_port::source::{
            include::rive::refcnt_hpp::{make_rcp, rcp},
            renderer::include::rive::renderer::{
                render_canvas_hpp::RenderCanvas, render_target_hpp::RenderTarget,
                texture_hpp::Texture,
            },
        };
        use nuxie_render_api::{DeferredCanvasHost, Factory, PersistentFactory, RecordingFactory};

        #[repr(C)]
        struct CountedTexture {
            base: Texture,
            drops: Rc<Cell<usize>>,
        }
        #[repr(C)]
        struct CountedTarget {
            base: RenderTarget,
            drops: Rc<Cell<usize>>,
        }
        impl Drop for CountedTexture {
            fn drop(&mut self) {
                self.drops.set(self.drops.get() + 1);
            }
        }
        impl Drop for CountedTarget {
            fn drop(&mut self) {
                self.drops.set(self.drops.get() + 1);
            }
        }
        fn backing(drops: &Rc<Cell<usize>>) -> (rcp<Texture>, rcp<RenderTarget>) {
            unsafe fn destroy_texture(base: *mut Texture) {
                unsafe { drop(Box::from_raw(base.cast::<CountedTexture>())) };
            }
            unsafe fn destroy_target(base: *mut RenderTarget) {
                unsafe { drop(Box::from_raw(base.cast::<CountedTarget>())) };
            }
            let mut texture = Texture::new(4, 3);
            texture.destroy_complete = destroy_texture;
            let mut target = RenderTarget::new(4, 3);
            target.destroy_complete = destroy_target;
            let texture = Box::into_raw(Box::new(CountedTexture {
                base: texture,
                drops: drops.clone(),
            }))
            .cast::<Texture>();
            let target = Box::into_raw(Box::new(CountedTarget {
                base: target,
                drops: drops.clone(),
            }))
            .cast::<RenderTarget>();
            // Each pointer is an offset-zero intrusive base with a complete
            // destructor and its initial retain transferred into the rcp.
            unsafe { (rcp::from_ptr(texture), rcp::from_ptr(target)) }
        }

        #[test]
        fn deferred_image_and_target_retire_at_independent_last_drops() {
            let retirer = Arc::new(CanvasRetirer::default());
            let drops = Rc::new(Cell::new(0));
            let canvas = make_rcp(|| RenderCanvas::new_deferred(retirer.clone(), 4, 3));
            let (texture, target) = backing(&drops);
            unsafe {
                (&mut *canvas.get()).setBacking(texture, target);
            }
            let image = unsafe { &*canvas.get() }.ref_render_image();
            drop(canvas);
            let targets = retirer.take();
            assert_eq!(targets.len(), 1);
            assert!(targets[0].texture.is_none());
            assert!(targets[0].render_target.is_some());
            assert_eq!(drops.get(), 0);
            drop(targets);
            assert_eq!(drops.get(), 1);
            drop(image);
            let textures = retirer.take();
            assert_eq!(textures.len(), 1);
            assert!(textures[0].texture.is_some());
            assert!(textures[0].render_target.is_none());
            assert_eq!(drops.get(), 1);
            drop(textures);
            assert_eq!(drops.get(), 2);
        }

        fn session_canvas(
            session: &mut DeferredSession,
            drops: &Rc<Cell<usize>>,
        ) -> nuxie_render_api::RenderCanvasHandle {
            let factory = PersistentFactory::new(RecordingFactory::default());
            session.bind_render_context(factory.persistent_context());
            let canvas = session.make_content_canvas(4, 3).expect("source shell");
            let (texture, target) = backing(drops);
            {
                let mut shell = canvas.borrow_mut();
                let shell = (&mut **shell as &mut dyn Any)
                    .downcast_mut::<crate::exact_source_adapter::ExactSourceRenderCanvas>()
                    .unwrap();
                unsafe {
                    (&mut *shell.source_ptr()).setBacking(texture, target);
                }
            }
            canvas
        }

        #[test]
        fn snapshot_and_inline_or_idle_drain_own_retired_backings() {
            let mut session = DeferredSession::with_caps(Default::default());
            let drops = Rc::new(Cell::new(0));
            drop(session_canvas(&mut session, &drops));
            let frame = snapshot_frame(&mut session);
            assert_eq!(frame.retired_canvas_backings.len(), 2);
            assert_eq!(drops.get(), 0);
            session.reset_frame();
            assert_eq!(drops.get(), 0);
            drop(frame);
            assert_eq!(drops.get(), 2);
            drop(session_canvas(&mut session, &drops));
            session.reset_frame();
            assert_eq!(drops.get(), 4);
            drop(session_canvas(&mut session, &drops));
            session.take_pending_destroys();
            assert_eq!(drops.get(), 6);
        }

        #[test]
        fn frame_canvas_release_queues_backing_for_the_following_drain() {
            let mut session = DeferredSession::with_caps(Default::default());
            let drops = Rc::new(Cell::new(0));
            let canvas = session_canvas(&mut session, &drops);
            session.routing.borrow_mut().register_canvas(canvas.clone());
            let frame = snapshot_frame(&mut session);
            session.reset_frame();
            drop(canvas);
            assert_eq!(drops.get(), 0);
            assert!(frame.retired_canvas_backings.is_empty());
            drop(frame);
            assert_eq!(drops.get(), 0);
            let next = snapshot_frame(&mut session);
            assert_eq!(next.retired_canvas_backings.len(), 2);
            drop(next);
            assert_eq!(drops.get(), 2);
        }

        #[test]
        fn canvas_outliving_last_session_handle_finds_retirement_closed() {
            let mut session = DeferredSession::with_caps(Default::default());
            let drops = Rc::new(Cell::new(0));
            let old = session_canvas(&mut session, &drops);
            let last = session.clone();
            drop(session);
            drop(old);
            assert_eq!(drops.get(), 0);
            drop(last);
            assert_eq!(drops.get(), 2);
            let mut session = DeferredSession::with_caps(Default::default());
            let survivor = session_canvas(&mut session, &drops);
            drop(session);
            assert_eq!(drops.get(), 2);
            drop(survivor);
            assert_eq!(drops.get(), 4);
        }
    }
}
