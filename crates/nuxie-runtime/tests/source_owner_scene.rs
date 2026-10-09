//! Pinned Scene::draw dispatches Artboard::draw without a receiver loan.
use nuxie_render_api as api;
use nuxie_runtime::source::{
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    core::{CoreArena, CoreHandle},
    factory::RuntimeFactoryHandle,
    scene::Scene,
};
use std::{cell::RefCell, rc::Rc};

struct Renderer {
    root: CoreHandle,
    external_owner: Rc<RefCell<Option<RuntimeArtboardInstanceHandle>>>,
    calls: Vec<&'static str>,
}
impl api::Renderer for Renderer {
    fn save(&mut self) {
        self.calls.push("save");
        self.external_owner.borrow_mut().take();
        assert_eq!(
            self.root
                .with_downcast::<Artboard, _>(Artboard::frame_origin),
            Some(true)
        );
        self.root
            .with_downcast_mut::<Artboard, _>(|a| a.set_frame_origin(false))
            .unwrap();
    }
    fn restore(&mut self) {
        self.calls.push("restore");
        assert_eq!(
            self.root
                .with_downcast::<Artboard, _>(Artboard::frame_origin),
            Some(false)
        );
    }
    fn transform(&mut self, _: api::Mat2D) {
        panic!("Artboard must read frameOrigin after the save callback");
    }
    fn draw_path(&mut self, _: &dyn api::RenderPath, _: &dyn api::RenderPaint) {
        unreachable!()
    }
    fn clip_path(&mut self, _: &dyn api::RenderPath) {
        self.calls.push("clip");
        assert_eq!(
            self.root
                .with_downcast::<Artboard, _>(Artboard::frame_origin),
            Some(false)
        );
    }
    fn draw_image(
        &mut self,
        _: Option<&dyn api::RenderImage>,
        _: api::ImageSampler,
        _: api::BlendMode,
        _: f32,
    ) {
        unreachable!()
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
        unreachable!()
    }
    fn modulate_opacity(&mut self, _: f32) {
        unreachable!()
    }
}

#[test]
fn scene_draw_dispatches_artboard_and_retains_it_through_reentrant_callbacks() {
    let arena = CoreArena::default();
    let mut factory = api::PersistentFactory::new(api::NullFactory::new());
    let source = arena.insert(Artboard::with_factory(
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
    ));
    let instance = Artboard::instance_from_handle(&source).unwrap();
    let root = instance.core_handle();
    root.with_mut(|owner| {
        owner
            .as_transform_component_mut()
            .unwrap()
            .update_render_opacity_state(None)
    })
    .unwrap();
    let weak = instance.downgrade();
    let mut scene = Scene::new(weak.clone());
    let external_owner = Rc::new(RefCell::new(Some(instance)));
    let mut renderer = Renderer {
        root: root.clone(),
        external_owner: external_owner.clone(),
        calls: Vec::new(),
    };
    let before = api::artboard_draw_frame_id();
    scene.draw(&mut renderer);
    assert_eq!(api::artboard_draw_frame_id() - before, 1);
    assert_eq!(renderer.calls, ["save", "clip", "restore"]);
    assert!(external_owner.borrow().is_none());
    assert!(
        weak.upgrade().is_none(),
        "the temporary draw owner must be released afterward"
    );
    assert!(!root.is_alive());
}
