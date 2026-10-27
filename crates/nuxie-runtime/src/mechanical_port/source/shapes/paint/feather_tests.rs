//! Upstream path_test.cpp: outer-to-inner feather regression at 156e959c.
use crate::source::{
    generated::{core_registry::CoreRegistry, shapes::paint::feather_base::FeatherBase},
    shapes::{
        paint::{feather::Feather, fill::Fill},
        shape::Shape,
    },
};
use crate::{Artboard, File, RuntimeFactoryHandle};
use nuxie_render_api::{NullFactory, PersistentFactory};

#[test]
fn feather_switched_from_outer_to_inner_builds_its_inner_path() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    let bytes = std::fs::read(
        std::path::PathBuf::from(root).join("tests/unit_tests/assets/feather_render_test.riv"),
    )
    .unwrap();
    let mut factory = PersistentFactory::new(NullFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let source = file
        .with_file(|file| file.artboard_named_source("button 2"))
        .unwrap();
    let artboard = Artboard::instance_from_handle(&source).unwrap();
    let shape = artboard
        .with_artboard(|board| board.find_handle::<Shape>("button_shape"))
        .unwrap();
    let fill = shape
        .with_downcast::<Shape, _>(|shape| shape.children()[2].clone())
        .unwrap();
    let feather = fill
        .with_downcast::<Fill, _>(|fill| fill.feather())
        .unwrap()
        .unwrap();
    let set_inner = |value| {
        assert!(CoreRegistry::set_bool_handle(
            &feather,
            FeatherBase::INNER_PROPERTY_KEY.into(),
            value
        ))
    };
    let state = || {
        feather
            .with_downcast::<Feather, _>(|feather| {
                let path = feather.inner_path();
                let path = path.borrow();
                (
                    feather.render_count,
                    path.raw_path().empty(),
                    path.raw_path().bounds().width(),
                )
            })
            .unwrap()
    };

    // Start outer so the inner path is never built.
    set_inner(false);
    artboard.advance_default(0.0);
    assert_eq!(state().0, 0);
    assert!(state().1);

    // Switching to inner alone (no geometry change) must build it.
    set_inner(true);
    artboard.advance_default(0.0);
    assert_eq!(state().0, 1);
    assert!(!state().1);
    let width = state().2;

    // Changes while outer must be picked up, including strength * 1.5 padding.
    set_inner(false);
    artboard.advance_default(0.0);
    let strength = feather
        .with_downcast::<Feather, _>(|feather| feather.base.strength())
        .unwrap();
    assert!(CoreRegistry::set_double_handle(
        &feather,
        FeatherBase::STRENGTH_PROPERTY_KEY.into(),
        strength + 10.0
    ));
    artboard.advance_default(0.0);
    assert_eq!(state().0, 1);
    set_inner(true);
    artboard.advance_default(0.0);
    assert_eq!(state().0, 2);
    let actual = f64::from(state().2);
    let expected = f64::from(width + 30.0);
    // Catch Approx's default epsilon and zero margin/scale.
    assert!((actual - expected).abs() <= 100.0 * f64::from(f32::EPSILON) * expected.abs());
}
