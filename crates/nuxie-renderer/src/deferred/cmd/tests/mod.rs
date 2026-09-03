//! Literal test-body ports from tests/unit_tests/renderer at e949498e.
mod canvas_schedule_test;
mod deferred_canvas_import_test;
#[cfg(all(
    feature = "rive-decoders",
    any(
        feature = "native-vulkan-experimental",
        feature = "renderer-vulkan",
        feature = "renderer-webgpu",
        feature = "renderer-webgl2",
        feature = "renderer-metal"
    )
))]
mod deferred_flush_parity_test;
mod deferred_measure_test;
#[cfg(feature = "with-rive-path-query")]
mod deferred_path_query_test;
mod deferred_replay_order_test;
mod deferred_segment_test;
mod deferred_source_equivalence_test;
mod foreign_image_registry_test;
mod gpu_census_test;
#[cfg(all(
    feature = "rive-decoders",
    any(
        feature = "native-vulkan-experimental",
        feature = "renderer-vulkan",
        feature = "renderer-webgpu",
        feature = "renderer-webgl2",
        feature = "renderer-metal"
    )
))]
mod render_context_null;
use super::{deferred_replayer::DeferredFrameSink, render_replay::RendererOwner};
use nuxie_render_api::*;
use std::{
    any::Any,
    cell::{Cell, RefCell},
    collections::HashMap,
    rc::Rc,
};

pub struct TestSink {
    pub factory: PersistentFactory<SerializingFactory>,
    pub screens: HashMap<u64, RendererOwner>,
    pub opened_canvases: Vec<usize>,
}
impl Default for TestSink {
    fn default() -> Self {
        Self {
            factory: PersistentFactory::new(SerializingFactory::new()),
            screens: HashMap::new(),
            opened_canvases: Vec::new(),
        }
    }
}
impl DeferredFrameSink for TestSink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.factory.persistent_context().unwrap()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, target: u64) -> Option<RendererOwner> {
        self.factory.borrow_mut().frame_size(256, 256);
        self.factory.borrow_mut().add_frame();
        let renderer: RendererOwner = Rc::new(RefCell::new(Box::new(
            self.factory.borrow().make_renderer(),
        )));
        self.screens.insert(target, renderer.clone());
        Some(renderer)
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        _clear: u32,
    ) -> Option<RendererOwner> {
        self.opened_canvases.push(Rc::as_ptr(&canvas) as usize);
        None
    }
}
pub struct ImageInner {
    pub tag: i32,
    pub destroyed: Rc<Cell<bool>>,
}
impl Drop for ImageInner {
    fn drop(&mut self) {
        self.destroyed.set(true);
    }
}
#[derive(Clone)]
pub struct ForeignImage(pub Rc<ImageInner>);
impl ForeignImage {
    pub fn new(tag: i32, destroyed: Rc<Cell<bool>>) -> Self {
        Self(Rc::new(ImageInner { tag, destroyed }))
    }
}
impl RenderImage for ForeignImage {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn width(&self) -> u32 {
        4
    }
    fn height(&self) -> u32 {
        4
    }
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.0) as usize
    }
}
pub struct FakeCanvas {
    image: Rc<dyn RenderImage>,
}
impl RenderCanvas for FakeCanvas {
    fn width(&self) -> u32 {
        8
    }
    fn height(&self) -> u32 {
        8
    }
    fn render_image(&self) -> Rc<dyn RenderImage> {
        self.image.clone()
    }
    fn begin_frame(
        &mut self,
        _clear: u32,
    ) -> Result<Box<dyn RenderCanvasFrame>, RenderCanvasError> {
        Err(RenderCanvasError::unsupported())
    }
}
pub fn fake_canvas() -> RenderCanvasHandle {
    Rc::new(RefCell::new(Box::new(FakeCanvas {
        image: Rc::new(ForeignImage::new(0, Rc::new(Cell::new(false)))),
    })))
}

pub struct RuntimeCase {
    pub _file: nuxie_runtime::RuntimeFileHandle,
    pub artboard: nuxie_runtime::RuntimeArtboardInstanceHandle,
    pub scene: Option<nuxie_runtime::mechanical_port::source::artboard::Scene>,
}
impl RuntimeCase {
    pub fn import(bytes: &[u8], factory: &mut dyn Factory) -> Option<Self> {
        Self::import_result(bytes, factory).ok()
    }
    pub fn import_result(bytes: &[u8], factory: &mut dyn Factory) -> Result<Self, &'static str> {
        let file = nuxie_runtime::File::import(
            bytes,
            nuxie_runtime::RuntimeFactoryHandle::from_factory(factory).ok_or("undecodable")?,
            None,
            None,
            None,
        )
        .ok_or("undecodable")?;
        let artboard = file
            .with_file(nuxie_runtime::File::artboard_default)
            .ok_or("no_artboard")?;
        let scene = artboard.default_scene();
        Ok(Self {
            _file: file,
            artboard,
            scene,
        })
    }
    pub fn advance(&mut self, seconds: f32) {
        use nuxie_runtime::mechanical_port::source::artboard::Scene;
        match &mut self.scene {
            Some(Scene::StateMachine(machine)) => {
                machine.advance_and_apply(seconds);
            }
            Some(Scene::LinearAnimation(animation)) => {
                animation.advance_and_apply(seconds);
            }
            None => {
                self.artboard.advance_default(seconds);
            }
        }
    }
    pub fn draw(&self, renderer: &mut dyn Renderer) {
        renderer.save();
        self.artboard.draw(renderer);
        renderer.restore();
    }
}
