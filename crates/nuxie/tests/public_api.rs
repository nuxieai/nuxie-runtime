// Test code is deliberately outside the panic-freedom lint gate (the crate
// lints table denies these for src/; unwrap/indexing are fine in tests).
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use nuxie::{
    AnimationId, ArtboardSpec, BlendMode, ColorInt, ExportedAnimatableProperty, ExportedObjectKind,
    ExportedProperty, Factory, File, FillRule, ImageDecodeError, LinearAnimationSpec, NodeSpec,
    ObjectId, Parent, RawPath, RecordingFactory, RenderBuffer, RenderBufferFlags, RenderBufferType,
    RenderImage, RenderPaint, RenderPaintStyle, RenderPath, RenderShader, Renderer, Scene,
    ShapeSpec, StateMachineInputKind, StrokeCap, StrokeJoin, props,
};
use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

#[test]
fn scene_animation_authoring_surface_is_typed_key_free_and_upsert_shaped() {
    let mut scene = Scene::new();
    let ((animation, first_key), _) = scene
        .edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Canvas".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let shape = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: "Fader".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            let animation: AnimationId = tx.animations().create_linear(
                artboard,
                LinearAnimationSpec {
                    name: "Fade".into(),
                    fps: 60,
                    duration: 60,
                },
            )?;
            let first_key: ObjectId =
                tx.animations()
                    .set_key(animation, shape, props::WORLD_OPACITY, 30, 0.25)?;
            let upserted_key =
                tx.animations()
                    .set_key(animation, shape, props::WORLD_OPACITY, 30, 0.5)?;
            assert_eq!(upserted_key, first_key);
            Ok((animation, first_key))
        })
        .expect("typed animation authoring must commit");

    let animation_object: ObjectId = animation.into();
    assert_eq!(animation_object, animation.object_id());
    assert_ne!(animation_object, first_key);
    let keyed_property = scene
        .export_records()
        .into_records()
        .into_iter()
        .find(|record| record.kind == ExportedObjectKind::KeyedProperty)
        .expect("typed property record");
    assert_eq!(
        keyed_property.properties,
        vec![ExportedProperty::KeyedProperty(
            ExportedAnimatableProperty::WorldOpacity,
        )],
        "the public record vocabulary remains semantic and exposes no numeric Rive key"
    );
}

struct DropTrackedRenderImage {
    inner: Box<dyn RenderImage>,
    dropped: Rc<Cell<usize>>,
}

impl Drop for DropTrackedRenderImage {
    fn drop(&mut self) {
        self.dropped.set(self.dropped.get() + 1);
    }
}

impl RenderImage for DropTrackedRenderImage {
    fn as_any(&self) -> &dyn std::any::Any {
        self.inner.as_any()
    }

    fn width(&self) -> u32 {
        self.inner.width()
    }

    fn height(&self) -> u32 {
        self.inner.height()
    }

    fn uv_transform(&self) -> nuxie::Mat2D {
        self.inner.uv_transform()
    }
}

struct DropTrackedRenderPaint {
    inner: Box<dyn RenderPaint>,
    dropped: Rc<Cell<usize>>,
}

impl Drop for DropTrackedRenderPaint {
    fn drop(&mut self) {
        self.dropped.set(self.dropped.get() + 1);
    }
}

impl RenderPaint for DropTrackedRenderPaint {
    fn as_any(&self) -> &dyn std::any::Any {
        self.inner.as_any()
    }

    fn style(&mut self, style: RenderPaintStyle) {
        self.inner.style(style);
    }

    fn color(&mut self, value: ColorInt) {
        self.inner.color(value);
    }

    fn thickness(&mut self, value: f32) {
        self.inner.thickness(value);
    }

    fn join(&mut self, value: StrokeJoin) {
        self.inner.join(value);
    }

    fn cap(&mut self, value: StrokeCap) {
        self.inner.cap(value);
    }

    fn feather(&mut self, value: f32) {
        self.inner.feather(value);
    }

    fn blend_mode(&mut self, value: BlendMode) {
        self.inner.blend_mode(value);
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        self.inner.shader(shader);
    }

    fn invalidate_stroke(&mut self) {
        self.inner.invalidate_stroke();
    }
}

struct FailFirstImageDecodeFactory {
    inner: RecordingFactory,
    fail_next_image_decode: bool,
    decode_attempts: usize,
    images_created: Rc<Cell<usize>>,
    images_dropped: Rc<Cell<usize>>,
    paints_created: Rc<Cell<usize>>,
    paints_dropped: Rc<Cell<usize>>,
}

impl FailFirstImageDecodeFactory {
    fn new() -> Self {
        Self {
            inner: RecordingFactory::new(),
            fail_next_image_decode: true,
            decode_attempts: 0,
            images_created: Rc::new(Cell::new(0)),
            images_dropped: Rc::new(Cell::new(0)),
            paints_created: Rc::new(Cell::new(0)),
            paints_dropped: Rc::new(Cell::new(0)),
        }
    }
}

impl Factory for FailFirstImageDecodeFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        self.inner
            .make_render_buffer(buffer_type, flags, size_in_bytes)
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
        self.inner
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.inner.make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.inner.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        let paint = self.inner.make_render_paint();
        self.paints_created.set(self.paints_created.get() + 1);
        Box::new(DropTrackedRenderPaint {
            inner: paint,
            dropped: Rc::clone(&self.paints_dropped),
        })
    }

    fn decode_image(
        &mut self,
        data: &[u8],
    ) -> std::result::Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.decode_attempts += 1;
        if std::mem::take(&mut self.fail_next_image_decode) {
            return Err(ImageDecodeError);
        }
        let image = self.inner.decode_image(data)?;
        self.images_created.set(self.images_created.get() + 1);
        Ok(Box::new(DropTrackedRenderImage {
            inner: image,
            dropped: Rc::clone(&self.images_dropped),
        }))
    }
}

fn repo_fixture(relative: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative);
    std::fs::read(&path).expect("read repo fixture")
}

fn external_fixture(name: &str) -> Vec<u8> {
    let path = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets")
    .join(name);
    std::fs::read(&path).expect("read external fixture")
}

/// Instantiate artboard 0, optionally set a view-model property before binding,
/// then advance and draw, returning the recorded stream.
fn render_with_view_model(
    bytes: &[u8],
    set: impl FnOnce(&mut nuxie::ViewModelInstance) -> bool,
) -> String {
    let file = File::import(bytes).expect("import file");
    let mut instance = file
        .default_artboard()
        .expect("default artboard")
        .instantiate()
        .expect("instantiate artboard");
    let mut view_model = instance
        .instantiate_view_model()
        .expect("artboard has a view model");
    let _ = set(&mut view_model);
    instance.bind_view_model(&view_model);
    instance.advance(0.0);

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    instance
        .draw(&mut factory, &mut renderer)
        .expect("draw artboard");
    factory.stream()
}

#[test]
fn public_api_imports_lists_instantiates_and_draws() {
    let fixture = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/shapetest.riv");
    let bytes = std::fs::read(&fixture).expect("read fixture");
    let file = File::import(&bytes).expect("import file");

    assert!(file.artboard_count() >= 1);
    let names = file
        .artboards()
        .map(|artboard| artboard.name().unwrap_or("<unnamed>").to_owned())
        .collect::<Vec<_>>();
    assert_eq!(names.len(), file.artboard_count());

    let artboard = file.default_artboard().expect("default artboard");
    let mut instance = artboard.instantiate().expect("instantiate artboard");
    instance.advance(0.0);

    let mut factory = RecordingFactory::new();
    let mut renderer = factory.make_renderer();
    instance
        .draw(&mut factory, &mut renderer)
        .expect("draw artboard");

    let stream = factory.stream();
    assert!(stream.contains("rive-golden-stream-v1"));
    assert!(stream.contains("drawPath"));
}

#[test]
fn retained_public_render_cache_retries_a_failed_image_decode_without_poisoning() {
    let file = File::import(&external_fixture("in_band_asset.riv")).expect("import image fixture");
    let mut instance = file
        .default_artboard()
        .expect("default image artboard")
        .instantiate()
        .expect("instantiate image artboard");
    let mut factory = FailFirstImageDecodeFactory::new();
    let mut cache = instance.new_render_cache();

    assert_eq!(
        factory.decode_attempts, 0,
        "retained cache creation must be renderer-neutral"
    );
    let mut renderer = factory.inner.make_renderer();
    let first = instance
        .draw_with_render_cache(&mut factory, &mut renderer, &mut cache)
        .expect_err("first image decode fails at draw time");
    assert!(first.downcast_ref::<ImageDecodeError>().is_some());
    assert_eq!(factory.decode_attempts, 1);
    assert_eq!(factory.images_created.get(), factory.images_dropped.get());
    assert!(
        factory.paints_created.get() > 0,
        "the failed candidate should exercise owned render resources"
    );
    assert_eq!(factory.paints_created.get(), factory.paints_dropped.get());

    instance
        .draw_with_render_cache(&mut factory, &mut renderer, &mut cache)
        .expect("the same retained cache retries and draws");
    assert!(factory.decode_attempts >= 2);
    assert!(factory.images_created.get() > factory.images_dropped.get());
    assert!(factory.paints_created.get() > factory.paints_dropped.get());

    drop(cache);
    assert_eq!(factory.images_created.get(), factory.images_dropped.get());
    assert_eq!(factory.paints_created.get(), factory.paints_dropped.get());
}

#[test]
fn public_api_exposes_the_default_rust_renderer() {
    let mut factory =
        nuxie::DefaultRendererFactory::new(16, 16).expect("construct the default Rust renderer");
    let mut frame = factory.begin_frame(0xff_12_34_56);
    let mut path = factory.make_empty_render_path();
    path.move_to(2.0, 2.0);
    path.line_to(14.0, 2.0);
    path.line_to(2.0, 14.0);
    path.close();
    let mut paint = factory.make_render_paint();
    paint.color(0xff_ff_00_00);
    frame.draw_path(path.as_ref(), paint.as_ref());

    let pixels = frame.finish().expect("render one default-backend frame");
    assert_eq!(pixels.len(), 16 * 16 * 4);
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel == [255, 0, 0, 255]),
        "the default backend must draw into the frame"
    );
}

#[test]
fn public_api_drives_default_state_machine_and_inputs() {
    let bytes = repo_fixture("fixtures/animation/smi_test.riv");
    let file = File::import(&bytes).expect("import file");

    let artboard = file
        .artboard_named("artboard to nest")
        .expect("artboard to nest");
    assert!(artboard.state_machine_count() >= 1);
    assert_eq!(artboard.state_machine_name(0), Some("State Machine 1"));
    assert_eq!(artboard.state_machine_name(99), None);
    assert_eq!(artboard.default_state_machine_index(), Some(0));

    let mut instance = artboard.instantiate().expect("instantiate artboard");
    let mut state_machine = instance
        .default_state_machine_instance()
        .expect("default state machine instance");

    let bool_index = state_machine.input_index_named("bool").expect("bool input");
    assert_eq!(
        state_machine.input(bool_index).map(|input| input.kind()),
        Some(StateMachineInputKind::Bool)
    );
    assert!(state_machine.set_bool(bool_index, true));

    let number_index = state_machine.input_index_named("num").expect("num input");
    assert!(state_machine.set_number(number_index, 42.0));

    let trigger_index = state_machine.input_index_named("trig").expect("trig input");
    assert!(state_machine.fire_trigger(trigger_index));

    // Wrong-kind writes are rejected.
    assert!(!state_machine.set_number(bool_index, 1.0));

    instance.advance_with_state_machine(&mut state_machine, 0.016);
    instance.advance_with_state_machine(&mut state_machine, 0.016);
}

#[test]
fn public_api_view_model_number_set_changes_stream() {
    // `data_binding_test_2.riv` artboard 0 binds a shape to the view model's
    // `num` number property, so setting it visibly changes the draw stream.
    let bytes = external_fixture("data_binding_test_2.riv");

    let baseline = render_with_view_model(&bytes, |_| false);
    let mutated = render_with_view_model(&bytes, |view_model| {
        assert!(
            view_model.set_number("num", 137.0),
            "num is a settable number property"
        );
        true
    });

    assert!(baseline.contains("rive-golden-stream-v1"));
    assert_ne!(
        baseline, mutated,
        "setting a bound number property must change the draw stream"
    );
    // Setting a property that does not exist reports no change.
    let mut probe = File::import(&bytes)
        .unwrap()
        .default_artboard()
        .unwrap()
        .instantiate()
        .unwrap()
        .instantiate_view_model()
        .unwrap();
    assert!(!probe.set_number("does-not-exist", 1.0));
    assert!(!probe.set_bool("num", true), "wrong-kind write is rejected");
}

#[test]
fn public_api_view_model_string_set_changes_stream() {
    // `relative_data_binding.riv` artboard 0 binds text to the view model's
    // `str` string property.
    let bytes = external_fixture("relative_data_binding.riv");

    let baseline = render_with_view_model(&bytes, |_| false);
    let mutated = render_with_view_model(&bytes, |view_model| {
        assert!(
            view_model.set_string("str", "nuxie view model string"),
            "str is a settable string property"
        );
        true
    });

    assert_ne!(
        baseline, mutated,
        "setting a bound string property must change the draw stream"
    );
}

#[test]
fn public_api_view_model_instance_selection_and_missing() {
    // Artboards without a view model yield no context.
    let bytes = repo_fixture("fixtures/animation/smi_test.riv");
    let file = File::import(&bytes).expect("import file");
    let instance = file
        .default_artboard()
        .unwrap()
        .instantiate()
        .expect("instantiate artboard");
    assert!(instance.view_model_index().is_none());
    assert!(instance.instantiate_view_model().is_none());
    assert!(instance.instantiate_view_model_instance(0).is_none());

    // A databind fixture exposes a view model and a source instance at index 0;
    // an out-of-range instance index yields nothing.
    let bytes = external_fixture("data_binding_test_2.riv");
    let file = File::import(&bytes).expect("import file");
    let instance = file
        .default_artboard()
        .unwrap()
        .instantiate()
        .expect("instantiate artboard");
    assert!(instance.view_model_index().is_some());
    assert!(instance.instantiate_view_model().is_some());
    assert!(instance.instantiate_view_model_instance(0).is_some());
    assert!(instance.instantiate_view_model_instance(9_999).is_none());
}
