use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::r#loop::Loop,
    command_path::CommandPath,
    core::CoreType,
    generated::{
        component_base::ComponentBase, core_registry::CoreRegistry,
        layout_component_base::LayoutComponentBase,
    },
    hittest_command_path::HitTestCommandPath,
    layout::layout_node_provider,
    layout_component::LayoutComponent,
    math::{aabb::IAabb, mat2d::Mat2D, path_types::FillRule},
    shapes::shape::Shape,
    static_scene::StaticScene,
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle};

#[path = "support/riv_bytes.rs"]
mod riv_bytes;

#[test]
fn native_area_hits_do_not_clip_root_or_nested_artboard_frames() {
    use nuxie_runtime::source::{
        generated::{
            artboard_base::ArtboardBase,
            backboard_base::BackboardBase,
            nested_artboard_base::NestedArtboardBase,
            node_base::NodeBase,
            shapes::{
                paint::{fill_base::FillBase, solid_color_base::SolidColorBase},
                parametric_path_base::ParametricPathBase,
                rectangle_base::RectangleBase,
                shape_base::ShapeBase,
            },
        },
        hit_info::HitInfo,
    };
    for nested in [false, true] {
        let mut riv = riv_bytes::RivBytes::default();
        riv.object(BackboardBase::TYPE_KEY);
        riv.end();
        let artboard = |riv: &mut riv_bytes::RivBytes| {
            riv.object(ArtboardBase::TYPE_KEY);
            riv.prop_float(LayoutComponentBase::WIDTH_PROPERTY_KEY, 120.0);
            riv.prop_float(LayoutComponentBase::HEIGHT_PROPERTY_KEY, 40.0);
            riv.end();
        };
        artboard(&mut riv);
        riv.object(ShapeBase::TYPE_KEY);
        riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0);
        riv.prop_float(NodeBase::X_PROPERTY_KEY, 125.0);
        riv.prop_float(NodeBase::Y_PROPERTY_KEY, 5.0);
        riv.end();
        riv.object(RectangleBase::TYPE_KEY);
        riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1);
        riv.prop_float(ParametricPathBase::WIDTH_PROPERTY_KEY, 20.0);
        riv.prop_float(ParametricPathBase::HEIGHT_PROPERTY_KEY, 10.0);
        riv.end();
        riv.object(FillBase::TYPE_KEY);
        riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 1);
        riv.end();
        riv.object(SolidColorBase::TYPE_KEY);
        riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 3);
        riv.end();
        if nested {
            artboard(&mut riv);
            riv.object(NestedArtboardBase::TYPE_KEY);
            riv.prop_uint(ComponentBase::PARENT_ID_PROPERTY_KEY, 0);
            riv.prop_uint(NestedArtboardBase::ARTBOARD_ID_PROPERTY_KEY, 0);
            riv.end();
        }
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &riv.bytes(),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let source = file
            .with_file(|file| file.artboard_at_source(usize::from(nested)))
            .unwrap();
        let instance = Artboard::instance_from_handle(&source).unwrap();
        instance.advance_default(0.0);
        assert!(instance.with_artboard(|artboard| artboard.clip()));
        // The native area picker does not add either artboard's draw clip.
        for (x, expected) in [(119, true), (120, true), (121, true), (140, false)] {
            let mut info = HitInfo {
                area: IAabb {
                    left: x,
                    top: 4,
                    right: x + 1,
                    bottom: 6,
                },
                mounts: Vec::new(),
            };
            let hit = instance
                .with_artboard_mut(|artboard| artboard.hit_test(&mut info, &Mat2D::identity()));
            assert_eq!(hit.is_some(), expected, "nested={nested}, x={x}");
            if expected {
                assert_eq!(info.mounts.len(), usize::from(nested));
            }
        }
    }
}

// Supplemental branch coverage for 6a8cf4a3's scripted hit component. These
// direct owners do not substitute for nested scene/state-machine coverage.
// Since upstream e54bc92a a hit, bounded or not, lands only on a drawable
// whose live script handles pointers; these bare drawables have none, so
// they take no hit, and their box is tested through `within_bounds`, the
// test a bounded hit then runs.
#[test]
fn scripted_bounded_hits_use_known_layout_size_and_inclusive_edges() {
    use nuxie_runtime::source::core::CoreArena;
    use nuxie_runtime::source::{
        animation::state_machine_instance::HitComponent,
        layout::layout_enums::{LayoutDirection, LayoutScaleType},
        math::vec2d::Vec2D,
        scripted::{scripted_drawable::HitScriptedDrawable, scripted_layout::ScriptedLayout},
    };

    let arena = CoreArena::default();
    let layout = arena.insert(ScriptedLayout::default());
    let hit = HitScriptedDrawable::new(layout.clone());
    assert_eq!(
        layout.with_downcast::<ScriptedLayout, _>(ScriptedLayout::layout_size),
        Some(None)
    );
    assert!(!hit.within_bounds(Vec2D::new(0.0, 0.0)));
    layout
        .with_downcast_mut::<ScriptedLayout, _>(|layout| {
            layout.control_size(
                Vec2D::new(20.0, 10.0),
                LayoutScaleType::Fixed,
                LayoutScaleType::Fixed,
                LayoutDirection::Inherit,
            );
        })
        .unwrap();
    assert_eq!(
        layout.with_downcast::<ScriptedLayout, _>(ScriptedLayout::layout_size),
        Some(Some(Vec2D::new(20.0, 10.0)))
    );
    for position in [
        Vec2D::new(0.0, 0.0),
        Vec2D::new(20.0, 0.0),
        Vec2D::new(0.0, 10.0),
        Vec2D::new(20.0, 10.0),
        Vec2D::new(10.0, 5.0),
    ] {
        assert!(hit.within_bounds(position));
        // Inside the box, but no pointer handler takes the hit.
        assert!(!hit.hit_test_bounded(position));
    }
    for position in [
        Vec2D::new(-0.1, 5.0),
        Vec2D::new(20.1, 5.0),
        Vec2D::new(10.0, -0.1),
        Vec2D::new(10.0, 10.1),
    ] {
        assert!(!hit.within_bounds(position));
        assert!(!hit.hit_test(position));
    }
    layout
        .with_mut(|layout| {
            layout
                .as_world_transform_component_mut()
                .unwrap()
                .set_world_transform(Mat2D::new(2.0, 0.0, 0.0, 3.0, 100.0, 200.0));
        })
        .unwrap();
    assert!(hit.within_bounds(Vec2D::new(120.0, 215.0)));
    assert!(!hit.within_bounds(Vec2D::new(10.0, 5.0)));
    layout
        .with_mut(|layout| {
            layout
                .as_world_transform_component_mut()
                .unwrap()
                .set_world_transform(Mat2D::from_scale(0.0, 1.0));
        })
        .unwrap();
    assert!(!hit.within_bounds(Vec2D::new(0.0, 5.0)));
    assert!(!hit.hit_test(Vec2D::new(0.0, 5.0)));
}

#[test]
fn unboxed_scripted_drawable_box_covers_everywhere() {
    use nuxie_runtime::source::core::CoreArena;
    use nuxie_runtime::source::{
        animation::state_machine_instance::HitComponent,
        math::vec2d::Vec2D,
        scripted::scripted_drawable::{HitScriptedDrawable, ScriptedDrawable},
    };

    let arena = CoreArena::default();
    let drawable = arena.insert(ScriptedDrawable::default());
    drawable
        .with_mut(|drawable| {
            drawable
                .as_world_transform_component_mut()
                .unwrap()
                .set_world_transform(Mat2D::from_scale(0.0, 0.0));
        })
        .unwrap();
    let hit = HitScriptedDrawable::new(drawable);
    for position in [Vec2D::new(-1000.0, -1000.0), Vec2D::new(1000.0, 1000.0)] {
        assert!(hit.within_bounds(position));
        assert!(!hit.hit_test_bounded(position));
        assert!(!hit.hit_test(position));
    }
}

fn add_rectangle(path: &mut HitTestCommandPath, left: f32, top: f32, right: f32, bottom: f32) {
    path.move_to(left, top);
    path.line_to(right, top);
    path.line_to(right, bottom);
    path.line_to(left, bottom);
    path.close();
}

#[test]
fn hittest_basics_direct_port() {
    let mut path = HitTestCommandPath::new(IAabb {
        left: 10,
        top: 10,
        right: 12,
        bottom: 12,
    });
    add_rectangle(&mut path, 0.0, 0.0, 20.0, 20.0);
    assert!(path.was_hit());

    let mut path = HitTestCommandPath::new(IAabb {
        left: 81,
        top: 156,
        right: 84,
        bottom: 159,
    });
    add_rectangle(&mut path, 29.9785, 32.5261, 231.102, 269.898);
    assert!(path.was_hit());
}

#[test]
fn hit_test_command_path_preserves_fill_rule_and_transform_contracts() {
    let mut path = HitTestCommandPath::new(IAabb {
        left: 4,
        top: 4,
        right: 6,
        bottom: 6,
    });
    path.set_fill_rule(FillRule::EvenOdd);
    add_rectangle(&mut path, 0.0, 0.0, 10.0, 10.0);
    add_rectangle(&mut path, 0.0, 0.0, 10.0, 10.0);
    assert!(
        !path.was_hit(),
        "two identical contours cancel for even-odd"
    );

    path.rewind();
    path.set_fill_rule(FillRule::NonZero);
    add_rectangle(&mut path, 0.0, 0.0, 10.0, 10.0);
    add_rectangle(&mut path, 0.0, 0.0, 10.0, 10.0);
    assert!(
        path.was_hit(),
        "two identical contours retain non-zero winding"
    );

    path.rewind();
    path.set_xform(Mat2D::new(2.0, 0.0, 0.0, 2.0, 20.0, 30.0));
    add_rectangle(&mut path, -10.0, -15.0, -5.0, -10.0);
    assert!(path.was_hit());
}

#[test]
fn static_scene_matches_the_pinned_cpp_api_contract() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let assets = PathBuf::from(root).join("tests/unit_tests/assets");
    let bytes = std::fs::read(assets.join("dependency_test.riv"))
        .expect("pinned static dependency fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("native File");
    let source = file
        .with_file(|file| file.artboard())
        .expect("source artboard");
    let artboard = Artboard::instance_from_handle(&source).expect("actual ArtboardInstance");
    let handle = artboard.core_handle();
    assert!(CoreRegistry::set_string_handle(
        &handle,
        i32::from(ComponentBase::NAME_PROPERTY_KEY),
        "still life".into()
    ));
    artboard.advance_default(0.0);
    assert!(
        !artboard.advance_default(0.0),
        "settled artboard reports no update"
    );

    let mut scene = StaticScene::new(artboard.downgrade());
    assert_eq!(scene.name(), "still life");
    assert!(
        !scene.is_translucent(),
        "StaticScene exposes the opaque dependency fixture's paint state"
    );
    assert_eq!(scene.loop_(), Loop::OneShot);
    assert_eq!(scene.duration_seconds(), 0.0);
    assert!(
        scene.advance_and_apply(12.5),
        "StaticScene ignores the artboard's false advance result"
    );

    // Observe zero elapsed on the real interpolation owner, not a replacement
    // artboard implementation: a nonzero advance below must move the same slot.
    // This separate animated fixture is not expected to settle at zero elapsed.
    let animated_bytes = std::fs::read(assets.join("layout/animated_participant.riv"))
        .expect("pinned animated participant fixture");
    let animated_file = File::import(
        &animated_bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
        None,
        None,
        None,
    )
    .expect("animated File");
    let artboard = animated_file
        .with_file(File::artboard_default)
        .expect("animated Artboard");
    artboard.advance_default(0.0);
    let mut scene = StaticScene::new(artboard.downgrade());
    let shape = artboard
        .with_artboard(|artboard| artboard.find_all_handles::<Shape>())
        .into_iter()
        .next()
        .expect("participant Shape");
    let participant = layout_node_provider::from_component(&shape).expect("actual participant");
    let width = || {
        participant
            .with_mut(|owner| {
                owner
                    .as_layout_node_provider_mut()
                    .expect("LayoutNodeProvider")
                    .layout_bounds()
                    .width()
            })
            .expect("live participant")
    };
    let container = artboard
        .with_artboard(|artboard| artboard.find_all_handles::<LayoutComponent>())
        .into_iter()
        .find(|owner| {
            !owner.is_type_of(<Artboard as CoreType>::TYPE_KEY)
                && owner
                    .with(|owner| {
                        owner
                            .as_layout_component()
                            .unwrap()
                            .style_handle()
                            .is_some()
                    })
                    .unwrap()
        })
        .expect("styled layout");
    assert_eq!(width(), 200.0);
    assert!(CoreRegistry::set_double_handle(
        &container,
        i32::from(LayoutComponentBase::WIDTH_PROPERTY_KEY),
        100.0
    ));
    assert!(scene.advance_and_apply(12.5));
    assert!(scene.advance_and_apply(12.5));
    assert_eq!(
        width(),
        200.0,
        "StaticScene ignores elapsed seconds and advances its artboard at zero"
    );
    for _ in 0..5 {
        artboard.advance_default(0.2);
    }
    assert!(
        width() < 200.0,
        "the same participant advances when elapsed time is supplied directly"
    );
}
