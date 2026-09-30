//! Runtime cases from tests/unit_tests/runtime/blend_mode_test.cpp at 3d0d3f56.
//! The GPU-paint case lives in the renderer; the animated producer is in Silver.
use nuxie_render_api as render;
use nuxie_runtime::source::{
    artboard::RuntimeArtboardInstanceHandle,
    core::{CoreArena, CoreHandle},
    core_context::{CoreContext, StatusCode},
    generated::{
        core_registry::CoreRegistry, drawable_base::DrawableBase,
        shapes::paint::shape_paint_base::ShapePaintBase,
    },
    shapes::{
        paint::{
            blend_mode::{BLEND_MODE_BIT_COUNT, BlendMode},
            fill::Fill,
        },
        shape::Shape,
    },
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle};

struct RecordingPaint {
    blend: render::BlendMode,
    additive: f32,
}
impl render::RenderPaint for RecordingPaint {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn style(&mut self, _: render::RenderPaintStyle) {}
    fn color(&mut self, _: render::ColorInt) {}
    fn thickness(&mut self, _: f32) {}
    fn join(&mut self, _: render::StrokeJoin) {}
    fn cap(&mut self, _: render::StrokeCap) {}
    fn feather(&mut self, _: f32) {}
    fn blend_mode(&mut self, value: render::BlendMode) {
        self.blend = value;
    }
    fn additiveness(&mut self, value: f32) {
        self.additive = value;
    }
    fn shader(&mut self, _: Option<&dyn render::RenderShader>) {}
    fn invalidate_stroke(&mut self) {}
}

#[derive(Default)]
struct RecordingFactory(render::NullFactory);
impl render::Factory for RecordingFactory {
    fn make_render_buffer(
        &mut self,
        kind: render::RenderBufferType,
        flags: render::RenderBufferFlags,
        size: usize,
    ) -> Box<dyn render::RenderBuffer> {
        self.0.make_render_buffer(kind, flags, size)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[render::ColorInt],
        stops: &[f32],
    ) -> Box<dyn render::RenderShader> {
        self.0.make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[render::ColorInt],
        stops: &[f32],
    ) -> Box<dyn render::RenderShader> {
        self.0.make_radial_gradient(cx, cy, radius, colors, stops)
    }
    fn make_render_path(
        &mut self,
        path: render::RawPath,
        rule: render::FillRule,
    ) -> Box<dyn render::RenderPath> {
        self.0.make_render_path(path, rule)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn render::RenderPath> {
        self.0.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn render::RenderPaint> {
        Box::new(RecordingPaint {
            blend: render::BlendMode::SrcOver,
            additive: -1.0,
        })
    }
    fn decode_image(
        &mut self,
        bytes: &[u8],
    ) -> Result<Box<dyn render::RenderImage>, render::ImageDecodeError> {
        self.0.decode_image(bytes)
    }
}

fn artboard() -> RuntimeArtboardInstanceHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR is required");
    let bytes =
        std::fs::read(std::path::PathBuf::from(root).join("tests/unit_tests/assets/shapetest.riv"))
            .unwrap();
    let mut factory = render::PersistentFactory::new(RecordingFactory::default());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    Artboard::instance_from_handle(&file.with_file(File::artboard).unwrap()).unwrap()
}

fn set(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, i32::from(key), value));
}
fn get(owner: &CoreHandle, key: u16) -> u32 {
    CoreRegistry::get_uint_handle(owner, i32::from(key)).unwrap()
}
fn first_shape(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard
        .with_artboard(|a| a.find_all_handles::<Shape>())
        .into_iter()
        .next()
        .expect("shape")
}

// Like the runtime's RuntimeArtboardObjectContext, release the artboard borrow
// after resolving each handle. onAddedDirty synchronously re-registers children
// on that same artboard, so the context must not hold it borrowed across the call.
struct ArtboardContext {
    arena: CoreArena,
    root: CoreHandle,
}
impl CoreContext for ArtboardContext {
    fn core_arena(&self) -> &CoreArena {
        &self.arena
    }
    fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
        self.root
            .with_downcast::<Artboard, _>(|a| a.resolve_handle(id))
            .flatten()
    }
}
fn paint(fill: &CoreHandle) -> (render::BlendMode, f32) {
    let handle = fill
        .with(|object| {
            object
                .as_shape_paint()
                .unwrap()
                .render_paint_handle()
                .unwrap()
        })
        .unwrap();
    let borrowed = handle.borrow();
    let paint = borrowed
        .as_any()
        .downcast_ref::<RecordingPaint>()
        .expect("recording paint");
    (paint.blend, paint.additive)
}
fn sync(fill: &CoreHandle, mode: render::BlendMode, amount: u8) {
    fill.with_mut(|object| {
        object
            .as_shape_paint_mut()
            .unwrap()
            .blend_mode(mode, amount)
    })
    .unwrap();
}
fn approx(actual: f32, expected: f32) {
    // Catch's default Approx epsilon is 100 times f32 epsilon, with zero scale.
    assert!(
        (actual - expected).abs() <= 100.0 * f32::EPSILON * expected.abs(),
        "{actual} != {expected}"
    );
}

#[test]
fn additive_is_a_valid_blend_mode() {
    assert_eq!(BlendMode::Additive as u32, 12);
    assert!((BlendMode::Additive as u32) < (1 << BLEND_MODE_BIT_COUNT));
}

#[test]
fn shape_paint_carries_its_own_additive_amount() {
    let arena = CoreArena::default();
    let fill = arena.insert(Fill::default());
    assert_eq!(
        get(&fill, ShapePaintBase::ADDITIVE_AMOUNT_PROPERTY_KEY),
        255
    );
    assert_eq!(
        get(&fill, ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY),
        127
    );
    set(
        &fill,
        ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY,
        BlendMode::Additive as u32,
    );
    set(&fill, ShapePaintBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 128);
    assert_eq!(
        get(&fill, ShapePaintBase::ADDITIVE_AMOUNT_PROPERTY_KEY),
        128
    );
    assert_eq!(
        get(&fill, ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY),
        12
    );
    set(
        &fill,
        ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY,
        BlendMode::Multiply as u32,
    );
    assert_eq!(
        get(&fill, ShapePaintBase::ADDITIVE_AMOUNT_PROPERTY_KEY),
        128
    );
}

#[test]
fn drawable_validates_blend_mode_values_on_load() {
    let check = |value| {
        let instance = artboard();
        let shape = first_shape(&instance);
        set(&shape, DrawableBase::BLEND_MODE_VALUE_PROPERTY_KEY, value);
        let root = instance.core_handle();
        let mut context = ArtboardContext {
            arena: root.retain_arena().unwrap(),
            root,
        };
        shape
            .with_downcast_mut::<Shape, _>(|shape| shape.on_added_dirty(&mut context))
            .unwrap()
    };
    assert_eq!(check(3), StatusCode::Ok);
    assert_eq!(check(28), StatusCode::Ok);
    assert_eq!(check(BlendMode::Additive as u32), StatusCode::Ok);
    assert_eq!(check(13), StatusCode::InvalidObject);
    assert_eq!(check(0), StatusCode::InvalidObject);
}

#[test]
fn loaded_drawable_reports_its_blend_mode_and_amount() {
    let instance = artboard();
    let shape = first_shape(&instance);
    set(
        &shape,
        DrawableBase::BLEND_MODE_VALUE_PROPERTY_KEY,
        BlendMode::Additive as u32,
    );
    set(&shape, DrawableBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 200);
    assert!(
        shape
            .with_downcast::<Shape, _>(|s| s.blend_mode() == BlendMode::Additive)
            .unwrap()
    );
    assert_eq!(get(&shape, DrawableBase::ADDITIVE_AMOUNT_PROPERTY_KEY), 200);
    set(&shape, DrawableBase::BLEND_MODE_VALUE_PROPERTY_KEY, 28);
    assert!(
        shape
            .with_downcast::<Shape, _>(|s| s.blend_mode() == BlendMode::Luminosity)
            .unwrap()
    );
    assert_eq!(get(&shape, DrawableBase::ADDITIVE_AMOUNT_PROPERTY_KEY), 200);
}

#[test]
fn shape_paint_syncs_mode_and_amount_onto_render_paint() {
    let instance = artboard();
    let fill = instance
        .with_artboard(|a| a.find_all_handles::<Fill>())
        .into_iter()
        .next()
        .expect("fill");
    set(
        &fill,
        ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY,
        BlendMode::Additive as u32,
    );
    set(&fill, ShapePaintBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 255);
    sync(&fill, render::BlendMode::Multiply, 0);
    assert_eq!(paint(&fill).0, render::BlendMode::Additive);
    approx(paint(&fill).1, 1.0);
    set(&fill, ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY, 127);
    set(&fill, ShapePaintBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 0);
    sync(&fill, render::BlendMode::Additive, 64);
    assert_eq!(paint(&fill).0, render::BlendMode::Additive);
    approx(paint(&fill).1, 64.0 / 255.0);
    sync(&fill, render::BlendMode::Screen, 200);
    assert_eq!(paint(&fill).0, render::BlendMode::Screen);
    assert_eq!(paint(&fill).1, 0.0);
}

#[test]
fn changing_the_amount_resyncs_the_paint_after_load() {
    let instance = artboard();
    let (fill, shape) = instance
        .with_artboard(|a| a.find_all_handles::<Fill>())
        .into_iter()
        .find_map(|fill| {
            let parent = fill
                .with(|f| f.as_component().unwrap().parent_handle())
                .flatten()?;
            parent.with_downcast::<Shape, _>(|_| ())?;
            Some((fill, parent))
        })
        .expect("fill belonging to a shape");
    set(&fill, ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY, 127);
    set(
        &shape,
        DrawableBase::BLEND_MODE_VALUE_PROPERTY_KEY,
        BlendMode::Additive as u32,
    );
    set(&shape, DrawableBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 64);
    assert_eq!(paint(&fill).0, render::BlendMode::Additive);
    approx(paint(&fill).1, 64.0 / 255.0);
    set(&shape, DrawableBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 200);
    approx(paint(&fill).1, 200.0 / 255.0);
    set(
        &fill,
        ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY,
        BlendMode::Additive as u32,
    );
    set(&fill, ShapePaintBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 128);
    assert_eq!(paint(&fill).0, render::BlendMode::Additive);
    approx(paint(&fill).1, 128.0 / 255.0);
    set(&fill, ShapePaintBase::BLEND_MODE_VALUE_PROPERTY_KEY, 127);
    set(&shape, DrawableBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 32);
    approx(paint(&fill).1, 32.0 / 255.0);
    set(&fill, ShapePaintBase::ADDITIVE_AMOUNT_PROPERTY_KEY, 5);
    approx(paint(&fill).1, 32.0 / 255.0);
}
