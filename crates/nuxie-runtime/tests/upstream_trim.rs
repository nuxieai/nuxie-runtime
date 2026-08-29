//! Direct ports of both cases in pinned
//! `tests/unit_tests/runtime/trim_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory, RecordingRenderer};
use nuxie_runtime::source::{
    core::CoreHandle,
    generated::{core_registry::CoreRegistry, transform_component_base::TransformComponentBase},
    math::path_types::PathVerb,
    node::Node,
    shapes::{paint::stroke::Stroke, shape::Shape},
};
use nuxie_runtime::{AdvanceFlags, Artboard, File, RuntimeFactoryHandle, RuntimeFileHandle};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn load_fixture(name: &str) -> (RuntimeFileHandle, RecordingRenderer) {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let renderer = factory.borrow().make_renderer();
    let factory =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("explicit retained factory");
    let file = File::import(&pinned_fixture(name), factory, None, None, None)
        .unwrap_or_else(|| panic!("{name} imports"));
    (file, renderer)
}

#[test]
fn a_zero_scale_path_will_trim_with_no_crash() {
    let (file, mut renderer) = load_fixture("trim.riv");
    let artboard = file.with_file(File::artboard).expect("default artboard");
    let node = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<Node>("I"))
        .flatten()
        .expect("node I");
    let scale_x = TransformComponentBase::SCALE_X_PROPERTY_KEY as i32;
    let scale_y = TransformComponentBase::SCALE_Y_PROPERTY_KEY as i32;
    let flags = AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME;
    assert_ne!(CoreRegistry::get_double_handle(&node, scale_x), Some(0.0));
    assert_ne!(CoreRegistry::get_double_handle(&node, scale_y), Some(0.0));

    Artboard::advance_handle(&artboard, 0.0, flags);
    Artboard::draw_handle(&artboard, &mut renderer);

    assert!(CoreRegistry::set_double_handle(&node, scale_x, 0.0));
    assert!(CoreRegistry::set_double_handle(&node, scale_y, 0.0));
    Artboard::advance_handle(&artboard, 0.0, flags);
    Artboard::draw_handle(&artboard, &mut renderer);
}

fn test_raw_path(artboard: &CoreHandle, shape_name: &str, verbs: &[PathVerb]) {
    let shape = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<Shape>(shape_name))
        .flatten()
        .unwrap_or_else(|| panic!("shape {shape_name}"));
    let stroke = shape
        .with_downcast::<Shape, _>(|shape| {
            shape
                .children()
                .iter()
                .find(|child| child.is_type_of(Stroke::TYPE_KEY))
                .cloned()
        })
        .flatten()
        .unwrap_or_else(|| panic!("stroke for {shape_name}"));
    assert!(stroke.is_type_of(Stroke::TYPE_KEY));
    let (effect, provider) = stroke
        .with_downcast::<Stroke, _>(|stroke| {
            (
                stroke.base.base.effects_container.effects.first().cloned(),
                *stroke.base.base.path_provider(),
            )
        })
        .expect("live Stroke");
    let effect = effect.unwrap_or_else(|| panic!("stroke effect for {shape_name}"));
    let effect_path = effect
        .with_mut(|effect| {
            effect
                .as_stroke_effect_mut()
                .expect("StrokeEffect")
                .effect_path(&provider)
        })
        .flatten()
        .unwrap_or_else(|| panic!("stroke effect path for {shape_name}"));
    assert_eq!(effect_path.borrow().raw_path().verbs(), verbs);
}

#[test]
fn different_types_of_trim_paths() {
    let (file, _renderer) = load_fixture("trim_path.riv");
    let artboard = file
        .with_file(|file| file.artboard_named_source("artboard-2"))
        .expect("artboard-2");
    Artboard::update_components_handle(&artboard);

    test_raw_path(
        &artboard,
        "clipped-rect",
        &[PathVerb::Move, PathVerb::Line, PathVerb::Line],
    );
    test_raw_path(
        &artboard,
        "clipped-rect-open",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
        ],
    );
    test_raw_path(
        &artboard,
        "clipped-rect-multi",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
        ],
    );
    test_raw_path(
        &artboard,
        "clipped-rect-multi-sync",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
        ],
    );
    test_raw_path(
        &artboard,
        "pen-shape",
        &[PathVerb::Move, PathVerb::Cubic, PathVerb::Cubic],
    );
    test_raw_path(
        &artboard,
        "pen-shape-close",
        &[
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Close,
        ],
    );
    test_raw_path(
        &artboard,
        "mixed-shapes",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Cubic,
        ],
    );
    test_raw_path(
        &artboard,
        "mixed-shapes-synced",
        &[
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
        ],
    );
    test_raw_path(
        &artboard,
        "mixed-shapes-synced-100",
        &[
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
        ],
    );
    test_raw_path(
        &artboard,
        "mixed-shapes-100",
        &[
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Move,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Line,
            PathVerb::Close,
            PathVerb::Move,
            PathVerb::Cubic,
            PathVerb::Cubic,
            PathVerb::Cubic,
        ],
    );
}
