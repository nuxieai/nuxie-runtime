//! `tests/unit_tests/runtime/instance_factory_test.cpp` at e949498e.
use nuxie_render_api::*;
use nuxie_runtime::source::{advance_flags::AdvanceFlags, nested_artboard::NestedArtboard};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle};

#[derive(Default)]
struct CountingFactory {
    inner: RecordingFactory,
    paints: usize,
    paths: usize,
    buffers: usize,
    shaders: usize,
}
impl Factory for CountingFactory {
    fn make_render_buffer(
        &mut self,
        kind: RenderBufferType,
        flags: RenderBufferFlags,
        size: usize,
    ) -> Box<dyn RenderBuffer> {
        self.buffers += 1;
        self.inner.make_render_buffer(kind, flags, size)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.shaders += 1;
        self.inner
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.shaders += 1;
        self.inner
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }
    fn make_render_path(&mut self, path: RawPath, rule: FillRule) -> Box<dyn RenderPath> {
        self.paths += 1;
        self.inner.make_render_path(path, rule)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.paths += 1;
        self.inner.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.paints += 1;
        self.inner.make_render_paint()
    }
    fn decode_image(&mut self, bytes: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.inner.decode_image(bytes)
    }
}

fn file() -> nuxie_runtime::source::file::RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        std::path::PathBuf::from(root).join("tests/unit_tests/assets/nested_artboard_opacity.riv"),
    )
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}

#[test]
fn instance_without_override_keeps_file_factory() {
    let file = file();
    let source = file.with_file(File::artboard).unwrap();
    let instance = Artboard::instance_from_handle(&source).unwrap();
    let original = source
        .with_downcast::<Artboard, _>(Artboard::factory)
        .flatten()
        .unwrap();
    assert_eq!(
        instance.with_artboard(|a| a.factory().unwrap().persistent_context().identity()),
        original.persistent_context().identity()
    );
}

#[test]
fn instance_override_reroutes_resource_creation() {
    let file = file();
    let mut facade = PersistentFactory::new(CountingFactory::default());
    let factory = RuntimeFactoryHandle::from_factory(&mut facade).unwrap();
    let instance = Artboard::instance_from_handle_with_factory(
        &file.with_file(File::artboard).unwrap(),
        Some(factory.clone()),
    )
    .unwrap();
    assert_eq!(
        instance.with_artboard(|a| a.factory().unwrap().persistent_context().identity()),
        factory.persistent_context().identity()
    );
    assert!(facade.borrow().paints > 0);
}

#[test]
fn nested_instances_inherit_override_factory() {
    let file = file();
    let mut facade = PersistentFactory::new(CountingFactory::default());
    let factory = RuntimeFactoryHandle::from_factory(&mut facade).unwrap();
    let instance = Artboard::instance_from_handle_with_factory(
        &file.with_file(File::artboard).unwrap(),
        Some(factory.clone()),
    )
    .unwrap();
    let nested = instance
        .with_artboard(|a| a.find_handle::<NestedArtboard>("Nested artboard container"))
        .unwrap();
    let source = nested
        .with_downcast::<NestedArtboard, _>(NestedArtboard::source_artboard)
        .flatten()
        .unwrap();
    let nested_factory = source
        .with_downcast::<Artboard, _>(Artboard::factory)
        .flatten()
        .unwrap();
    assert_eq!(
        nested_factory.persistent_context().identity(),
        factory.persistent_context().identity()
    );
}

#[test]
fn advance_and_draw_allocate_on_override_factory() {
    let file = file();
    let mut facade = PersistentFactory::new(CountingFactory::default());
    let factory = RuntimeFactoryHandle::from_factory(&mut facade).unwrap();
    let instance = Artboard::instance_from_handle_with_factory(
        &file.with_file(File::artboard).unwrap(),
        Some(factory),
    )
    .unwrap();
    instance.advance(
        0.016,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
    instance.draw(&mut NullRenderer);
    assert!(facade.borrow().paths > 0);
}
