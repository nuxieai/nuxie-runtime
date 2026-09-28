//! Source: 7098a7c8, runtime/layout_test.cpp, Corner Radius Link Change.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    generated::{
        core_registry::CoreRegistry,
        layout::layout_component_style_base::LayoutComponentStyleBase as Style,
    },
    layout_component::LayoutComponent,
};
use nuxie_runtime::{Artboard, File, ImportResult, RuntimeFactoryHandle};

#[test]
fn layout_component_corner_radius_link_change() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/layout/layout_complex1.riv");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Malformed;
    let file =
        File::import(&bytes, factory, Some(&mut result), None, None).expect("fixture imports");
    assert_eq!(result, ImportResult::Success);
    let artboard = file.with_file(File::artboard).expect("source artboard");
    let child = artboard
        .with_downcast::<Artboard, _>(|artboard| {
            artboard.find_handle::<LayoutComponent>("LayoutLeftChild1")
        })
        .flatten()
        .expect("LayoutLeftChild1");
    let style = child
        .with_downcast::<LayoutComponent, _>(LayoutComponent::style_handle)
        .flatten()
        .expect("layout style");
    let capture = || {
        Artboard::advance_handle(
            &artboard,
            0.0,
            AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
        );
        child
            .with_downcast_mut::<LayoutComponent, _>(|layout| {
                layout
                    .local_path()
                    .map(|path| path.raw_path().points().to_vec())
                    .unwrap_or_default()
            })
            .expect("live layout")
    };

    assert!(CoreRegistry::set_bool_handle(
        &style,
        i32::from(Style::LINK_CORNER_RADIUS_PROPERTY_KEY),
        false,
    ));
    for key in [
        Style::CORNER_RADIUS_TL_PROPERTY_KEY,
        Style::CORNER_RADIUS_TR_PROPERTY_KEY,
        Style::CORNER_RADIUS_BL_PROPERTY_KEY,
        Style::CORNER_RADIUS_BR_PROPERTY_KEY,
    ] {
        assert!(CoreRegistry::set_double_handle(&style, i32::from(key), 4.0));
    }
    let uniform = capture();
    assert!(!uniform.is_empty());

    for (key, value) in [
        (Style::CORNER_RADIUS_TR_PROPERTY_KEY, 12.0),
        (Style::CORNER_RADIUS_BL_PROPERTY_KEY, 20.0),
        (Style::CORNER_RADIUS_BR_PROPERTY_KEY, 28.0),
    ] {
        assert!(CoreRegistry::set_double_handle(
            &style,
            i32::from(key),
            value
        ));
    }
    let distinct = capture();
    assert_eq!(distinct.len(), uniform.len());

    assert!(CoreRegistry::set_bool_handle(
        &style,
        i32::from(Style::LINK_CORNER_RADIUS_PROPERTY_KEY),
        true,
    ));
    let linked = capture();
    assert_eq!(linked.len(), uniform.len());
    // Catch Approx's default relative epsilon is 100 * f32::EPSILON.
    for (actual, expected) in linked.iter().zip(&uniform) {
        for (actual, expected) in [(actual.x, expected.x), (actual.y, expected.y)] {
            let (actual, expected) = (f64::from(actual), f64::from(expected));
            assert!((actual - expected).abs() <= 100.0 * f64::from(f32::EPSILON) * expected.abs());
        }
    }
}
