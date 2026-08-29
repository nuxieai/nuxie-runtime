// Test code is deliberately outside the panic-freedom lint gate.
#![allow(
    clippy::unwrap_used,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

use std::cell::Cell;
use std::path::PathBuf;
use std::rc::Rc;

use nuxie::{
    BlendMode, ColorInt, Factory, File, FileAssetLoader, FileAssetLoaderRef, FillRule,
    ImageDecodeError, PersistentFactory, RecordingFactory, RenderBuffer, RenderBufferFlags,
    RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle, RenderPath, RenderShader,
    RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle, StrokeCap, StrokeJoin,
    ViewModelInstanceRuntime,
    render_api::{Mat2D as RenderMat2D, RawPath as RenderRawPath},
    runtime::assets::image_asset::ImageAsset,
};

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

    fn uv_transform(&self) -> RenderMat2D {
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
    fail_image_decode_on_attempt: Option<usize>,
    decode_attempts: usize,
    images_created: Rc<Cell<usize>>,
    images_dropped: Rc<Cell<usize>>,
    paints_created: Rc<Cell<usize>>,
    paints_dropped: Rc<Cell<usize>>,
}

impl FailFirstImageDecodeFactory {
    fn failing_on_attempt(attempt: usize) -> Self {
        Self {
            inner: RecordingFactory::new(),
            fail_image_decode_on_attempt: Some(attempt),
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

    fn make_render_path(
        &mut self,
        raw_path: RenderRawPath,
        fill_rule: FillRule,
    ) -> Box<dyn RenderPath> {
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
        if self.fail_image_decode_on_attempt == Some(self.decode_attempts) {
            self.fail_image_decode_on_attempt = None;
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

fn import_with_factory<F: Factory + 'static>(
    bytes: &[u8],
    factory: &mut PersistentFactory<F>,
    loader: Option<FileAssetLoaderRef>,
) -> RuntimeFileHandle {
    File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(factory).expect("retained factory"),
        None,
        loader,
        None,
    )
    .expect("import file")
}

fn recording_file(bytes: &[u8]) -> (PersistentFactory<RecordingFactory>, RuntimeFileHandle) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = import_with_factory(bytes, &mut factory, None);
    (factory, file)
}

fn default_view_model(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
) -> Option<nuxie::RuntimeViewModelInstanceHandle> {
    file.with_file(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
    })
    .map(ViewModelInstanceRuntime::new)
    .map(ViewModelInstanceRuntime::into_handle)
}

fn render_with_view_model(
    bytes: &[u8],
    set: impl FnOnce(&nuxie::RuntimeViewModelInstanceHandle),
) -> String {
    let (factory, file) = recording_file(bytes);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let view_model = default_view_model(&file, &artboard).expect("artboard has a view model");
    set(&view_model);
    artboard.bind_view_model_instance(Some(view_model.instance()));
    artboard.advance_default(0.0);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    factory.borrow().stream()
}

#[test]
fn public_api_imports_lists_instantiates_and_draws() {
    let bytes = external_fixture("shapetest.riv");
    let (factory, file) = recording_file(&bytes);
    let (count, names) = file.with_file(|file| {
        (
            file.artboard_count(),
            (0..file.artboard_count())
                .map(|index| file.artboard_name_at(index))
                .collect::<Vec<_>>(),
        )
    });
    assert!(count >= 1);
    assert_eq!(names.len(), count);

    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    artboard.advance_default(0.0);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    let stream = factory.borrow().stream();
    assert!(stream.contains("rive-golden-stream-v1"));
    assert!(stream.contains("drawPath"));
}

#[test]
fn image_decode_failure_is_scoped_to_one_source_import() {
    let bytes = external_fixture("in_band_asset.riv");
    let mut factory = PersistentFactory::new(FailFirstImageDecodeFactory::failing_on_attempt(1));
    let failed_file = import_with_factory(&bytes, &mut factory, None);
    let failed_image = failed_file
        .with_file(|file| file.asset(0))
        .expect("ImageAsset");
    assert!(
        failed_image
            .with_downcast::<ImageAsset, _>(|asset| asset.render_image().is_none())
            .unwrap_or(false)
    );
    assert_eq!(factory.borrow().decode_attempts, 1);

    let decoded_file = import_with_factory(&bytes, &mut factory, None);
    let decoded_image = decoded_file
        .with_file(|file| file.asset(0))
        .expect("ImageAsset");
    assert!(
        decoded_image
            .with_downcast::<ImageAsset, _>(|asset| asset.render_image().is_some())
            .unwrap_or(false)
    );
    assert_eq!(factory.borrow().decode_attempts, 2);
    drop(failed_file);
    drop(decoded_file);
    assert_eq!(
        factory.borrow().images_created.get(),
        factory.borrow().images_dropped.get()
    );
}

struct ExternalImageLoader {
    walle: Vec<u8>,
    eve: Vec<u8>,
    attempts: Rc<Cell<usize>>,
}

impl FileAssetLoader for ExternalImageLoader {
    fn load_contents(
        &mut self,
        asset: nuxie::CoreHandle,
        in_band_bytes: &[u8],
        factory: &RuntimeFactoryHandle,
    ) -> bool {
        let Some(name) = asset
            .with_downcast::<ImageAsset, _>(|image| image.base.file_asset().base.name().to_owned())
        else {
            return false;
        };
        assert!(in_band_bytes.is_empty());
        let bytes = match name.as_str() {
            "walle.jpg" => &self.walle,
            "eve.png" => &self.eve,
            other => panic!("unexpected external image {other}"),
        };
        self.attempts.set(self.attempts.get() + 1);
        asset
            .with_downcast_mut::<ImageAsset, _>(|image| image.decode(bytes, factory))
            .unwrap_or(false)
    }
}

#[test]
fn external_images_are_supplied_by_the_import_loader() {
    let attempts = Rc::new(Cell::new(0));
    let loader = FileAssetLoaderRef::new(Box::new(ExternalImageLoader {
        walle: external_fixture("out_of_band/walle-370.png"),
        eve: external_fixture("out_of_band/eve-317.png"),
        attempts: attempts.clone(),
    }));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = import_with_factory(
        &external_fixture("out_of_band/walle.riv"),
        &mut factory,
        Some(loader),
    );
    assert!(attempts.get() >= 2);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    artboard.advance_default(0.0);
    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    assert!(factory.borrow().stream().contains("drawImage"));
}

#[test]
fn embedded_image_contents_are_decoded_during_import() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = import_with_factory(&external_fixture("in_band_asset.riv"), &mut factory, None);
    let image = file.with_file(|file| file.asset(0)).expect("ImageAsset");
    assert!(
        image
            .with_downcast::<ImageAsset, _>(|image| image.render_image().is_some())
            .unwrap_or(false)
    );
    assert!(factory.borrow().stream().contains("decodeImage"));
}

#[cfg(all(feature = "renderer-metal", target_os = "macos"))]
#[test]
fn public_api_exposes_the_explicit_metal_renderer() {
    let mut factory = nuxie::NativeMetalFactory::new(16, 16)
        .expect("construct the explicit native Metal renderer");
    let mut frame = factory
        .begin_frame(0xff_12_34_56)
        .expect("begin native Metal frame");
    let mut path = factory.make_empty_render_path();
    path.move_to(2.0, 2.0);
    path.line_to(14.0, 2.0);
    path.line_to(2.0, 14.0);
    path.close();
    let mut paint = factory.make_render_paint();
    paint.color(0xff_ff_00_00);
    frame.draw_path(path.as_ref(), paint.as_ref());
    let pixels = frame.finish().expect("render one native Metal frame");
    assert_eq!(pixels.len(), 16 * 16 * 4);
    assert!(
        pixels
            .chunks_exact(4)
            .any(|pixel| pixel == [255, 0, 0, 255])
    );
}

#[cfg(all(feature = "renderer-metal", target_os = "macos"))]
#[test]
fn imported_artboard_renders_pixels_with_the_explicit_metal_renderer() {
    let mut factory = PersistentFactory::new(
        nuxie::NativeMetalFactory::new(4, 3).expect("construct native Metal renderer"),
    );
    let file = import_with_factory(&external_fixture("in_band_asset.riv"), &mut factory, None);
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let mut frame = factory
        .borrow_mut()
        .begin_frame(0xff_12_34_56)
        .expect("begin native Metal frame");
    artboard.draw(&mut frame);
    let output = frame.finish_for_benchmark().expect("finish frame");
    assert!(output.execution_inventory.draw_calls > 0);
    assert_eq!(&output.pixels[..4], &[49, 49, 49, 255]);
}

#[test]
fn public_api_drives_default_state_machine_and_inputs() {
    let (_factory, file) = recording_file(&repo_fixture("fixtures/animation/smi_test.riv"));
    let artboard = file
        .with_file(|file| file.artboard_named("artboard to nest"))
        .expect("artboard to nest");
    assert!(artboard.with_artboard(|artboard| artboard.base.state_machine_count()) >= 1);
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.base.state_machine_name_at(0)),
        "State Machine 1"
    );
    assert_eq!(
        artboard.with_artboard(|artboard| artboard.base.default_state_machine_index()),
        0
    );

    let machine = artboard
        .default_state_machine()
        .expect("default state machine instance");
    assert!(machine.with_instance(|machine| machine.get_bool("bool").is_some()));
    assert!(machine.with_instance(|machine| machine.get_number("num").is_some()));
    assert!(machine.with_instance(|machine| machine.get_trigger("trig").is_some()));
    assert!(machine.with_instance(|machine| machine.get_number("bool").is_none()));
    machine.set_bool("bool", true);
    machine.set_number("num", 42.0);
    machine.with_instance_mut(|machine| {
        machine.get_trigger_mut("trig").expect("trigger").fire();
    });
    machine.advance_and_apply(0.016);
    machine.advance_and_apply(0.016);
}

#[test]
fn zero_time_state_machine_advance_keeps_cpp_continuation() {
    let (_factory, file) = recording_file(&external_fixture("smi_test.riv"));
    let artboard = file
        .with_file(|file| file.artboard_named("artboard to nest"))
        .expect("artboard");
    let machine = artboard
        .default_state_machine()
        .expect("default state machine");
    assert!(machine.advance_and_apply(0.0));
    assert!(machine.advance_and_apply(0.0));
}

#[test]
fn public_api_view_model_number_set_changes_stream() {
    let bytes = external_fixture("data_binding_test_2.riv");
    let baseline = render_with_view_model(&bytes, |_| {});
    let mutated = render_with_view_model(&bytes, |view_model| {
        let number = view_model.property_number("num").expect("num property");
        number.set_value(137.0);
        assert_eq!(number.value(), 137.0);
    });
    assert!(baseline.contains("rive-golden-stream-v1"));
    assert_ne!(baseline, mutated);

    let (_factory, file) = recording_file(&bytes);
    let artboard = file.with_file(File::artboard_default).unwrap();
    let probe = default_view_model(&file, &artboard).unwrap();
    assert!(probe.property_number("does-not-exist").is_none());
    assert!(probe.property_boolean("num").is_none());
}

#[test]
fn public_api_view_model_string_set_changes_stream() {
    let bytes = external_fixture("relative_data_binding.riv");
    let baseline = render_with_view_model(&bytes, |_| {});
    let mutated = render_with_view_model(&bytes, |view_model| {
        let string = view_model.property_string("str").expect("str property");
        string.set_value("nuxie view model string".to_owned());
        assert_eq!(string.value(), "nuxie view model string");
    });
    assert_ne!(baseline, mutated);
}

#[test]
fn public_api_view_model_instance_selection_and_missing() {
    let (_factory, file) = recording_file(&repo_fixture("fixtures/animation/smi_test.riv"));
    let artboard = file.with_file(File::artboard_default).unwrap();
    assert!(default_view_model(&file, &artboard).is_none());

    let (_factory, file) = recording_file(&external_fixture("data_binding_test_2.riv"));
    let artboard = file.with_file(File::artboard_default).unwrap();
    assert!(default_view_model(&file, &artboard).is_some());
    let model = file
        .with_file(|file| file.default_artboard_view_model(file.artboard()))
        .expect("artboard view model");
    assert!(model.create_instance_from_index(0).is_some());
    assert!(model.create_instance_from_index(9_999).is_none());
}

#[test]
fn artboard_view_model_binding_leaves_global_completion_to_state_machines() {
    let (_factory, file) = recording_file(&external_fixture("global_viewmodels_test.riv"));
    let artboard = file.with_file(File::artboard_default).unwrap();
    let view_model = default_view_model(&file, &artboard).expect("default view model");
    artboard.bind_view_model_instance(Some(view_model.instance()));
    assert_eq!(
        artboard
            .data_context()
            .expect("retained artboard context")
            .with_context(|context| context.view_model_instances().len()),
        1
    );
}

#[test]
fn bound_view_model_state_machine_zero_seconds_keeps_advancing() {
    let (_factory, file) = recording_file(&external_fixture("global_viewmodels_test.riv"));
    let artboard = file.with_file(File::artboard_default).unwrap();
    let view_model = default_view_model(&file, &artboard).expect("default view model");
    let machine = artboard
        .default_state_machine()
        .expect("default state machine");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    artboard.bind_view_model_instance(Some(view_model.instance()));
    assert!(machine.advance_and_apply(0.0));
    assert!(machine.advance_and_apply(0.0));
}

#[test]
fn artboard_occurrence_clone_is_independent() {
    let (_factory, file) = recording_file(&external_fixture("hosted_font_file.riv"));
    let source = file.with_file(File::artboard_default).unwrap();
    let fork = source.instance().expect("clone artboard occurrence");
    let fork_dimensions = fork.with_artboard(|artboard| (artboard.width(), artboard.height()));
    source.set_size(fork_dimensions.0 + 17.0, fork_dimensions.1 + 23.0);
    let source_dimensions = source.with_artboard(|artboard| (artboard.width(), artboard.height()));
    assert_ne!(source_dimensions, fork_dimensions);
    assert_eq!(
        fork.with_artboard(|artboard| (artboard.width(), artboard.height())),
        fork_dimensions
    );
    assert!(source.with_artboard(|artboard| artboard.file().upgrade().is_some()));
    assert!(fork.with_artboard(|artboard| artboard.file().upgrade().is_some()));
}
