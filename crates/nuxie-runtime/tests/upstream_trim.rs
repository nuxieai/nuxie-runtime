//! Direct ports of the cases in pinned
//! `tests/unit_tests/runtime/trim_test.cpp`.

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory, RecordingRenderer};
use nuxie_runtime::source::{
    core::CoreHandle,
    generated::{
        core_registry::CoreRegistry, shapes::parametric_path_base::ParametricPathBase,
        transform_component_base::TransformComponentBase,
        world_transform_component_base::WorldTransformComponentBase,
    },
    math::{path_types::PathVerb, vec2d::Vec2D},
    node::Node,
    shapes::{paint::stroke::Stroke, rectangle::Rectangle, shape::Shape},
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

fn trim_points(artboard: &CoreHandle, shape_name: &str) -> Vec<Vec2D> {
    let shape = artboard
        .with_downcast::<Artboard, _>(|artboard| artboard.find_handle::<Shape>(shape_name))
        .flatten()
        .expect("shape");
    let stroke = shape
        .with_downcast::<Shape, _>(|shape| {
            shape
                .children()
                .iter()
                .find(|child| child.is_type_of(Stroke::TYPE_KEY))
                .cloned()
        })
        .flatten();
    let Some(stroke) = stroke else {
        return Vec::new();
    };
    let (effect, provider) = stroke
        .with_downcast::<Stroke, _>(|stroke| {
            (
                stroke.base.base.effects_container.effects.last().cloned(),
                *stroke.base.base.path_provider(),
            )
        })
        .unwrap();
    let effect_path = effect
        .expect("stroke effect")
        .with_mut(|effect| {
            effect
                .as_stroke_effect_mut()
                .unwrap()
                .effect_path(&provider)
        })
        .flatten()
        .expect("effect path");
    effect_path.borrow().raw_path().points().to_vec()
}

/// Complete added trim_test.cpp case at upstream 93e4ce468f0a.
#[test]
fn hidden_stroke_picks_up_path_changes_once_shown() {
    let (file, _renderer) = load_fixture("trim_path.riv");
    let artboard = file
        .with_file(|file| file.artboard_named("artboard-2"))
        .unwrap();
    let control = file
        .with_file(|file| file.artboard_named("artboard-2"))
        .unwrap();
    let find_shape = |root: &CoreHandle| {
        root.with_downcast::<Artboard, _>(|board| board.find_handle::<Shape>("clipped-rect"))
            .flatten()
            .expect("clipped-rect")
    };
    let shape = find_shape(&artboard.core_handle());
    let rect = shape
        .with_downcast::<Shape, _>(|shape| shape.paths()[0].clone())
        .unwrap();
    assert!(rect.is_type_of(Rectangle::TYPE_KEY));
    artboard.advance_default(0.0);
    let before = trim_points(&artboard.core_handle(), "clipped-rect");
    assert!(!before.is_empty());

    let opacity = i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY);
    let width = i32::from(ParametricPathBase::WIDTH_PROPERTY_KEY);
    assert!(CoreRegistry::set_double_handle(&shape, opacity, 0.0));
    artboard.advance_default(0.0);
    let new_width = CoreRegistry::get_double_handle(&rect, width).unwrap() * 2.0;
    assert!(CoreRegistry::set_double_handle(&rect, width, new_width));
    artboard.advance_default(0.0);
    assert!(CoreRegistry::set_double_handle(&shape, opacity, 1.0));
    artboard.advance_default(0.0);

    let control_shape = find_shape(&control.core_handle());
    let control_rect = control_shape
        .with_downcast::<Shape, _>(|shape| shape.paths()[0].clone())
        .unwrap();
    assert!(control_rect.is_type_of(Rectangle::TYPE_KEY));
    let actual_width = CoreRegistry::get_double_handle(&rect, width).unwrap();
    assert!(CoreRegistry::set_double_handle(
        &control_rect,
        width,
        actual_width
    ));
    control.advance_default(0.0);
    let expected = trim_points(&control.core_handle(), "clipped-rect");
    let after = trim_points(&artboard.core_handle(), "clipped-rect");
    assert_eq!(after.len(), expected.len());
    assert_ne!(after, before);
    for (actual, expected) in after.iter().zip(&expected) {
        for (actual, expected) in [(actual.x, expected.x), (actual.y, expected.y)] {
            assert!(
                (f64::from(actual) - f64::from(expected)).abs()
                    <= 100.0 * f64::from(f32::EPSILON) * f64::from(expected).abs()
            );
        }
    }
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
                stroke.base.base.effects_container.effects.last().cloned(),
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
