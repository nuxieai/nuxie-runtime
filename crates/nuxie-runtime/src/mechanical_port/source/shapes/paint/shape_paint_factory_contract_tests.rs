use super::*;
use crate::source::{artboard::Artboard, core::CoreArena, shapes::paint::fill::Fill};
use nuxie_render_api as api;

struct Context {
    arena: CoreArena,
    root: CoreHandle,
}
impl CoreContext for Context {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        (id == 0).then(|| self.root.clone())
    }
}
fn installed_fill(root: &CoreHandle, arena: &CoreArena) -> CoreHandle {
    let fill = arena.insert(Fill::default());
    let mut context = Context {
        arena: arena.clone(),
        root: root.clone(),
    };
    assert_eq!(
        fill.with_mut(|owner| owner
            .as_component_mut()
            .unwrap()
            .on_added_dirty(&mut context)),
        Some(StatusCode::Ok)
    );
    fill
}
struct FactorySwap {
    root: CoreHandle,
    replacement: RuntimeFactoryHandle,
}
impl api::Renderer for FactorySwap {
    fn save(&mut self) {
        self.root
            .with_downcast_mut::<Artboard, _>(|root| root.set_factory(self.replacement.clone()))
            .unwrap();
    }
    fn restore(&mut self) {}
    fn transform(&mut self, _: api::Mat2D) {}
    fn draw_path(&mut self, _: &dyn api::RenderPath, _: &dyn api::RenderPaint) {}
    fn clip_path(&mut self, _: &dyn api::RenderPath) {}
    fn draw_image(
        &mut self,
        _: Option<&dyn api::RenderImage>,
        _: api::ImageSampler,
        _: api::BlendMode,
        _: f32,
    ) {
    }
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn api::RenderImage>,
        _: api::ImageSampler,
        _: Option<&dyn api::RenderBuffer>,
        _: Option<&dyn api::RenderBuffer>,
        _: Option<&dyn api::RenderBuffer>,
        _: u32,
        _: u32,
        _: api::BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}
#[test]
fn new_path_uses_the_factory_installed_by_the_preceding_renderer_callback_once() {
    let arena = CoreArena::default();
    let mut original = api::PersistentFactory::new(api::RecordingFactory::new());
    let mut replacement = api::PersistentFactory::new(api::RecordingFactory::new());
    let root = arena.insert(Artboard::with_factory(
        RuntimeFactoryHandle::from_factory(&mut original).unwrap(),
    ));
    let fill = installed_fill(&root, &arena);
    let mut renderer = FactorySwap {
        root: root.clone(),
        replacement: RuntimeFactoryHandle::from_factory(&mut replacement).unwrap(),
    };
    let mut path = ShapePaintPath::new(true);
    fill.with_mut(|object| {
        object
            .as_shape_paint_behavior_mut()
            .unwrap()
            .shape_paint_mut()
            .draw_with_fill_rule(
                &mut renderer,
                &mut path,
                Mat2D::default(),
                true,
                None,
                true,
                None,
            )
    })
    .unwrap();
    assert!(!original.borrow().stream().contains("makeEmptyRenderPath"));
    assert_eq!(
        replacement
            .borrow()
            .stream()
            .matches("makeEmptyRenderPath")
            .count(),
        1
    );
    assert!(path.has_render_path());
    // Reuse and dirty rebuild retain the same renderer object after the
    // Artboard selects a different factory; no new allocation is permitted.
    root.with_downcast_mut::<Artboard, _>(|root| {
        root.set_factory(RuntimeFactoryHandle::from_factory(&mut original).unwrap())
    })
    .unwrap();
    for dirty in [false, true] {
        if dirty {
            path.rewind();
        }
        fill.with_mut(|object| {
            object
                .as_shape_paint_behavior_mut()
                .unwrap()
                .shape_paint_mut()
                .render_path_for_draw(&mut path, None);
        })
        .unwrap();
    }
    assert!(!original.borrow().stream().contains("makeEmptyRenderPath"));
    assert_eq!(
        replacement
            .borrow()
            .stream()
            .matches("makeEmptyRenderPath")
            .count(),
        1
    );
}
#[test]
fn existing_path_drawing_does_not_reborrow_the_live_artboard_for_an_unused_factory() {
    let arena = CoreArena::default();
    let mut factory = api::PersistentFactory::new(api::RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let root = arena.insert(Artboard::with_factory(factory.clone()));
    let fill = installed_fill(&root, &arena);
    let mut path = ShapePaintPath::new(true);
    path.render_path(&factory);
    for dirty in [false, true] {
        if dirty {
            path.rewind();
        }
        root.with_downcast_mut::<Artboard, _>(|_root| {
            fill.with_mut(|object| {
                object
                    .as_shape_paint_behavior_mut()
                    .unwrap()
                    .shape_paint_mut()
                    .draw_with_fill_rule(
                        &mut api::NullRenderer,
                        &mut path,
                        Mat2D::default(),
                        true,
                        None,
                        true,
                        None,
                    );
            })
            .unwrap();
        })
        .unwrap();
        assert!(path.has_render_path());
    }
}
