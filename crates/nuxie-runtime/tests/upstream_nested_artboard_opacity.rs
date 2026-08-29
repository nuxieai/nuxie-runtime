//! Direct ports of all three cases in pinned
//! `tests/unit_tests/runtime/nested_artboard_opacity_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    generated::{
        core_registry::CoreRegistry, nested_artboard_base::NestedArtboardBase,
        shapes::paint::shape_paint_base::ShapePaintBase,
        world_transform_component_base::WorldTransformComponentBase,
    },
    nested_artboard::NestedArtboard,
    shapes::paint::shape_paint::ShapePaint,
};
use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};

fn fixture() -> (
    RuntimeFileHandle,
    PersistentFactory<RecordingFactory>,
    RuntimeArtboardInstanceHandle,
) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/nested_artboard_opacity.riv");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("explicit retained factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(&bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("nested_artboard_opacity.riv imports: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    let source = file.with_file(File::artboard).expect("source artboard");
    let instance = Artboard::instance_from_handle(&source).expect("main artboard instance");
    let parent = instance
        .with_artboard(|instance| instance.find_handle::<Artboard>("Parent Artboard"))
        .expect("Parent Artboard");
    assert_eq!(parent, instance.core_handle());
    (file, factory, instance)
}

fn nested_host(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    artboard
        .with_artboard(|artboard| {
            artboard.find_handle::<NestedArtboard>("Nested artboard container")
        })
        .expect("Nested artboard container")
}

fn nested_instance(host: &CoreHandle) -> RuntimeArtboardInstanceHandle {
    let nested = host
        .with_downcast::<NestedArtboard, _>(|host| host.artboard_instance_handle(0))
        .flatten()
        .expect("mounted nested instance");
    assert!(nested.with_artboard(|instance| {
        instance
            .find_handle::<Artboard>("Nested artboard")
            .is_some()
    }));
    nested
}

fn background_paint(nested: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    let paints = nested.with_artboard(|instance| {
        instance
            .base
            .base
            .base
            .shape_paint_container()
            .shape_paints()
            .to_vec()
    });
    assert_eq!(paints.len(), 1);
    let paint = paints[0].clone();
    assert!(paint.is_type_of(ShapePaintBase::TYPE_KEY));
    paint
}

fn render_opacity(paint: &CoreHandle) -> f32 {
    paint
        .with(|paint| paint.as_shape_paint().map(ShapePaint::render_opacity))
        .flatten()
        .expect("live ShapePaint")
}

fn approx_eq(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 100.0 * f32::EPSILON * expected.abs(),
        "expected {expected}, got {actual}"
    );
}

#[test]
fn nested_artboard_background_renders_with_opacity() {
    let (_file, _factory, artboard) = fixture();
    Artboard::update_components_handle(&artboard.core_handle());
    let nested = nested_instance(&nested_host(&artboard));
    Artboard::update_components_handle(&nested.core_handle());
    // Upstream checks the background paint, not the host drawable's opacity.
    assert_eq!(render_opacity(&background_paint(&nested)), 0.3275);
}

#[test]
fn paused_nested_artboard_still_propagates_host_opacity() {
    let (_file, _factory, artboard) = fixture();
    let host = nested_host(&artboard);
    let nested = nested_instance(&host);
    artboard.advance_default(0.0);
    let paint = background_paint(&nested);
    let baseline = render_opacity(&paint);
    assert!(baseline > 0.0);

    assert!(CoreRegistry::set_bool_handle(
        &host,
        i32::from(NestedArtboardBase::IS_PAUSED_PROPERTY_KEY),
        true
    ));
    let opacity_key = i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY);
    let host_opacity = CoreRegistry::get_double_handle(&host, opacity_key).expect("host opacity");
    assert!(CoreRegistry::set_double_handle(
        &host,
        opacity_key,
        host_opacity * 0.5
    ));
    artboard.advance_default(0.0);

    approx_eq(render_opacity(&paint), baseline * 0.5);
}

#[test]
fn nested_artboard_own_opacity_combines_with_host_opacity() {
    let (_file, factory, artboard) = fixture();
    Artboard::update_components_handle(&artboard.core_handle());
    let nested = nested_instance(&nested_host(&artboard));
    let opacity_key = i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY);

    // Exactly the pinned test: set the mounted instance's own opacity and its
    // separate runtime hostOpacity, without rewriting the container property.
    assert!(CoreRegistry::set_double_handle(
        &nested.core_handle(),
        opacity_key,
        0.4
    ));
    nested.with_artboard_mut(|nested| nested.set_host_opacity(0.5));
    Artboard::update_components_handle(&nested.core_handle());

    assert_eq!(
        CoreRegistry::get_double_handle(&nested.core_handle(), opacity_key),
        Some(0.4)
    );
    approx_eq(nested.with_artboard(|nested| nested.child_opacity()), 0.2);
    approx_eq(render_opacity(&background_paint(&nested)), 0.2);

    // Retain the prior port's additional renderer assertion, now observing the
    // same mounted instance that the upstream numeric assertions inspect.
    let mut renderer = factory.borrow().make_renderer();
    nested.draw(&mut renderer);
    assert!(
        factory.borrow().stream().contains("color=0x33ff0000"),
        "the nested red background receives own 0.4 × host 0.5 opacity"
    );
}
