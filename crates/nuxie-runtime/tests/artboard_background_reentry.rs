use nuxie_render_api::{
    BlendMode, ImageSampler, Mat2D, RenderBuffer, RenderImage, RenderPaint, RenderPath, Renderer, PersistentFactory, RecordingFactory,
};
use nuxie_runtime::source::core::CoreArena;
use nuxie_runtime::{Artboard, CoreHandle, RuntimeFactoryHandle};

struct RootReader(CoreHandle);
impl Renderer for RootReader {
    fn save(&mut self) {
        assert_eq!(
            self.0.with_downcast::<Artboard, _>(Artboard::frame_origin),
            Some(true)
        );
    }
    fn restore(&mut self) { assert!(self.0.with_downcast::<Artboard, _>(Artboard::frame_origin).is_some()); }
    fn transform(&mut self, _: Mat2D) { assert!(self.0.with_downcast::<Artboard, _>(Artboard::frame_origin).is_some()); }
    fn draw_path(&mut self, _: &dyn RenderPath, _: &dyn RenderPaint) {}
    fn clip_path(&mut self, _: &dyn RenderPath) { self.0.with_downcast_mut::<Artboard, _>(|root| root.set_frame_origin(false)).unwrap(); }
    fn draw_image(&mut self, _: Option<&dyn RenderImage>, _: ImageSampler, _: BlendMode, _: f32) {}
    fn draw_image_mesh(
        &mut self,
        _: Option<&dyn RenderImage>,
        _: ImageSampler,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: Option<&dyn RenderBuffer>,
        _: u32,
        _: u32,
        _: BlendMode,
        _: f32,
    ) {
    }
    fn modulate_opacity(&mut self, _: f32) {}
}

#[test]
fn artboard_renderer_save_can_read_root() {
    let arena = CoreArena::default();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let root = arena.insert(Artboard::with_factory(factory));
    root.with_mut(|root| {
        root.as_transform_component_mut()
            .unwrap()
            .update_render_opacity_state(None);
    })
    .unwrap();
    Artboard::draw_content_handle(&root, &mut RootReader(root.clone()));
}
