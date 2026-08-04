//! Scene-API coverage for authored ScrollConstraints: creation, clamping,
//! occurrence isolation, observation, coherent draw/hit geometry, and the
//! scroll-composed read seams.
//!
//! Behavior is pinned against C++ `ScrollConstraint` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`: offsets store raw and
//! unclamped (`m_offsetX/Y`), reads clamp live against settled
//! content/viewport extents (`scroll_constraint.cpp:141-181`), and scroll is
//! a post-layout world composition over the content's layout children that
//! never mutates the settled layout solve (`scroll_constraint.cpp:182-230`).

use anyhow::Result;
use nuxie::{
    Aabb, ArtboardId, ArtboardSpec, EditReason, ExportedObjectKind, ExportedProperty, FillSpec,
    InstanceId, LayoutComponentSpec, LayoutComponentStyleSpec, NodeSpec, ObjectId, Parent,
    RecordingFactory, RectangleSpec, ResolveError, Scene, SceneLayoutFlexDirection,
    SceneLayoutScale, SceneLayoutUnit, ScrollConstraintDirection, ScrollConstraintId,
    ScrollConstraintSpec, ScrollProperty, ShapeSpec, SolidColorSpec, Vec2D,
};

fn canonical_draw_stream(scene: &mut Scene, instance: InstanceId) -> Result<String> {
    scene.reset_renderer(instance)?;
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_draw_token(instance)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    Ok(factory.canonical_recording().stream().to_owned())
}

fn layout(name: &str, width: f32, height: f32) -> NodeSpec {
    NodeSpec::LayoutComponent(LayoutComponentSpec {
        name: name.into(),
        x: 0.0,
        y: 0.0,
        opacity: 1.0,
        rotation: 0.0,
        scale_x: 1.0,
        scale_y: 1.0,
        clip: false,
        width,
        height,
        fractional_width: 1.0,
        fractional_height: 1.0,
        style: LayoutComponentStyleSpec {
            layout_width_scale: SceneLayoutScale::Fixed,
            layout_height_scale: SceneLayoutScale::Fixed,
            ..LayoutComponentStyleSpec::default()
        },
    })
}

struct ScrollFixture {
    artboard: ArtboardId,
    viewport: ObjectId,
    content: ObjectId,
    child1: ObjectId,
    child2: ObjectId,
    probe: ObjectId,
    constraint: ScrollConstraintId,
}

/// Viewport 100x100 (clip) > content 100x140 column > 100x60 + 100x80 rows.
/// Vertical scroll extents match the editor ScrollView shape:
/// `maxOffsetY = min(0, 100 - 140 - 0) = -40`.
fn author_scroll_scene(scene: &mut Scene, initial: ScrollConstraintSpec) -> Result<ScrollFixture> {
    let (fixture, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Screen".into(),
            width: 200.0,
            height: 200.0,
        })?;
        let mut viewport_node = layout("Viewport", 100.0, 100.0);
        let NodeSpec::LayoutComponent(viewport_spec) = &mut viewport_node else {
            unreachable!("layout helper always returns a LayoutComponent")
        };
        viewport_spec.clip = true;
        let viewport = tx.create(Parent::Artboard(artboard), viewport_node)?;
        let mut content_node = layout("Content", 100.0, 140.0);
        let NodeSpec::LayoutComponent(content_spec) = &mut content_node else {
            unreachable!("layout helper always returns a LayoutComponent")
        };
        content_spec.style.flex_direction = SceneLayoutFlexDirection::Column;
        let content = tx.create(Parent::Object(viewport), content_node)?;
        let child1 = tx.create(Parent::Object(content), layout("First", 100.0, 60.0))?;
        let child2 = tx.create(Parent::Object(content), layout("Second", 100.0, 80.0))?;
        let probe = tx.create(
            Parent::Object(child2),
            NodeSpec::Shape(ShapeSpec {
                name: "Probe".into(),
                x: 50.0,
                y: 40.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(probe),
            NodeSpec::Rectangle(RectangleSpec::new("Probe bounds", 100.0, 80.0)),
        )?;
        let fill = tx.create(
            Parent::Object(probe),
            NodeSpec::Fill(FillSpec {
                name: "Probe fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Probe color".into(),
                color: 0xffab_cdef,
            }),
        )?;
        let constraint = tx.create_scroll_constraint(content, initial)?;
        Ok(ScrollFixture {
            artboard,
            viewport,
            content,
            child1,
            child2,
            probe,
            constraint,
        })
    })?;
    Ok(fixture)
}

fn settle(scene: &mut Scene, artboard: ArtboardId) -> Result<InstanceId> {
    let instance = scene.instantiate(artboard)?;
    let mut events = Vec::new();
    let _ = scene.frame().advance(instance, 0.0, &mut events);
    Ok(instance)
}

#[test]
fn authored_scroll_constraint_settles_a_live_clamping_occurrence() -> Result<()> {
    let mut scene = Scene::new();
    let fixture = author_scroll_scene(&mut scene, ScrollConstraintSpec::default())?;
    let instance = settle(&mut scene, fixture.artboard)?;

    let snapshot = scene.scroll_constraint_snapshot(instance, fixture.constraint)?;
    assert_eq!(snapshot.offset, (0.0, 0.0));
    assert_eq!(
        snapshot.lower_bound,
        (0.0, -40.0),
        "maxOffsetY = min(0, viewport 100 - content 140 - paddingBottom 0)",
    );
    assert_eq!(snapshot.upper_bound, (0.0, 0.0));
    assert_eq!(snapshot.clamped_offset, (0.0, 0.0));
    assert!(!snapshot.physics_present);
    assert!(!snapshot.physics_running);

    // The authored occurrence resolves through the imported-scroll resolvers
    // exactly like a `.riv`-imported one.
    let occurrences = scene.scroll_constraint_occurrences(instance)?;
    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0], snapshot);
    assert_eq!(
        scene.scroll_constraint_for_content(instance, snapshot.content_local_id)?,
        Some(snapshot),
    );

    // The exported record is a plain ScrollConstraint core object parented
    // to its content.
    let records = scene.export_records();
    let record = records
        .records()
        .iter()
        .find(|record| record.kind == ExportedObjectKind::ScrollConstraint)
        .expect("authored ScrollConstraint is exported");
    let content_local = u32::try_from(snapshot.content_local_id)?;
    assert!(
        record
            .properties
            .contains(&ExportedProperty::ParentId(content_local)),
        "the constraint's parent is the content LayoutComponent",
    );
    assert!(
        record
            .properties
            .contains(&ExportedProperty::ScrollDirectionValue(1)),
        "vertical is DraggableConstraintDirection::vertical == 1",
    );
    Ok(())
}

#[test]
fn authored_initial_offset_imports_like_a_riv_authored_one() -> Result<()> {
    let mut scene = Scene::new();
    let fixture = author_scroll_scene(
        &mut scene,
        ScrollConstraintSpec {
            scroll_offset_y: -10.0,
            ..ScrollConstraintSpec::default()
        },
    )?;
    let instance = settle(&mut scene, fixture.artboard)?;
    let snapshot = scene.scroll_constraint_snapshot(instance, fixture.constraint)?;
    assert_eq!(snapshot.offset, (0.0, -10.0));
    assert_eq!(snapshot.clamped_offset, (0.0, -10.0));
    Ok(())
}

#[test]
fn scroll_offset_write_clamps_and_settles_draw_hit_and_reads_in_the_same_frame() -> Result<()> {
    let mut scene = Scene::new();
    let fixture = author_scroll_scene(&mut scene, ScrollConstraintSpec::default())?;
    let instance = settle(&mut scene, fixture.artboard)?;

    let child2_solved = scene.solved_layout_bounds(instance, fixture.child2)?;
    assert_eq!(child2_solved, Aabb::new(0.0, 60.0, 100.0, 140.0));
    assert_eq!(
        scene.scrolled_layout_bounds(instance, fixture.child2)?,
        child2_solved,
        "with a zero offset the scrolled read equals the solved read",
    );
    let probe_before = scene
        .frame()
        .world_bounds(instance, fixture.probe)
        .expect("probe draws");
    let stream_before = canonical_draw_stream(&mut scene, instance)?;
    assert!(
        scene
            .frame()
            .hit_test(instance, Vec2D::new(50.0, 30.0))
            .is_empty(),
        "the unfilled first row does not hit before scrolling",
    );
    assert_eq!(
        scene.frame().hit_test(instance, Vec2D::new(50.0, 95.0)),
        vec![fixture.probe],
        "the second row's probe is visible at the viewport bottom before scrolling",
    );

    // In-range write: raw == clamped == -30.
    let snapshot =
        scene.set_scroll_property(instance, fixture.constraint, ScrollProperty::OffsetY, -30.0)?;
    assert_eq!(snapshot.offset, (0.0, -30.0));
    assert_eq!(snapshot.clamped_offset, (0.0, -30.0));

    // The settled layout solve NEVER reflects scroll (pinned C++
    // `layoutBounds()` is untouched by `constrain`/`constrainChild`);
    // the scrolled read seam composes the same world translate the
    // semantic provider uses.
    assert_eq!(
        scene.solved_layout_bounds(instance, fixture.child2)?,
        child2_solved,
    );
    assert_eq!(
        scene.scrolled_layout_bounds(instance, fixture.child2)?,
        Aabb::new(0.0, 30.0, 100.0, 110.0),
        "the settled second-row box moves by exactly the clamped offset",
    );
    assert_eq!(
        scene.scrolled_layout_bounds(instance, fixture.content)?,
        scene.solved_layout_bounds(instance, fixture.content)?,
        "the content owns the constraint and does not displace itself",
    );
    let child2_transform = scene
        .frame()
        .world_transform_with_scroll(instance, fixture.child2)
        .expect("second row resolves");
    assert_eq!(
        (child2_transform.0[4], child2_transform.0[5]),
        (0.0, 30.0),
        "the scroll-composed transform carries the clamped translate",
    );

    // Channel parity: a hot host write and a cold authored initial offset
    // settle to the same occurrence state and the same draw/hit output.
    // Retained draw/hit recomposition of scrolled solved-layout content is a
    // known tracked divergence vs pinned C++ (the layout-scroll silver
    // family, e.g. `layout_scroll_drag_multiplier_layouts`, is on the
    // burn-down register); both authoring channels must stay equal through
    // that burn-down so neither picks up bespoke behavior.
    let mut cold_scene = Scene::new();
    let cold = author_scroll_scene(
        &mut cold_scene,
        ScrollConstraintSpec {
            scroll_offset_y: -30.0,
            ..ScrollConstraintSpec::default()
        },
    )?;
    let cold_instance = settle(&mut cold_scene, cold.artboard)?;
    let cold_snapshot = cold_scene.scroll_constraint_snapshot(cold_instance, cold.constraint)?;
    assert_eq!(cold_snapshot.offset, snapshot.offset);
    assert_eq!(cold_snapshot.clamped_offset, snapshot.clamped_offset);
    assert_eq!(
        cold_scene.scrolled_layout_bounds(cold_instance, cold.child2)?,
        scene.scrolled_layout_bounds(instance, fixture.child2)?,
    );
    let hot_stream = canonical_draw_stream(&mut scene, instance)?;
    let cold_stream = canonical_draw_stream(&mut cold_scene, cold_instance)?;
    assert_eq!(
        hot_stream, cold_stream,
        "a hot write and a cold authored offset draw identically",
    );
    for point in [Vec2D::new(50.0, 30.0), Vec2D::new(50.0, 95.0)] {
        let hot_hits = scene.frame().hit_test(instance, point);
        let cold_hits = cold_scene.frame().hit_test(cold_instance, point);
        assert_eq!(
            hot_hits.len(),
            cold_hits.len(),
            "hot and cold hit the same target count at {point:?}",
        );
        assert_eq!(
            hot_hits.contains(&fixture.probe),
            cold_hits.contains(&cold.probe),
            "hot and cold agree on the probe at {point:?}",
        );
    }
    let _ = (probe_before, stream_before);

    // Out-of-range write: raw stores unclamped, reads clamp to -40.
    let snapshot = scene.set_scroll_property(
        instance,
        fixture.constraint,
        ScrollProperty::OffsetY,
        -100.0,
    )?;
    assert_eq!(snapshot.offset, (0.0, -100.0), "raw offset stays unclamped");
    assert_eq!(snapshot.clamped_offset, (0.0, -40.0));
    assert_eq!(
        scene.scrolled_layout_bounds(instance, fixture.child2)?,
        Aabb::new(0.0, 20.0, 100.0, 100.0),
    );
    let _ = fixture.child1;
    let _ = fixture.viewport;
    Ok(())
}

#[test]
fn virtualized_clamp_includes_content_padding_at_the_pin() -> Result<()> {
    // The pin-discriminating clamp case: at `4ac7b327` a non-infinite
    // virtualized constraint's content extent includes the content's leading
    // and trailing padding (`scroll_constraint.cpp:26-66`); at the audit ref
    // `d788e8ec` it did not. Children 60 + 80 with padding 5 + 7 must clamp
    // against content 152, not 140: `maxOffsetY = min(0, 100 - 152) = -52`.
    let mut scene = Scene::new();
    let ((artboard, constraint), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Padded".into(),
            width: 200.0,
            height: 200.0,
        })?;
        let mut viewport_node = layout("Viewport", 100.0, 100.0);
        let NodeSpec::LayoutComponent(viewport_spec) = &mut viewport_node else {
            unreachable!("layout helper always returns a LayoutComponent")
        };
        viewport_spec.clip = true;
        let viewport = tx.create(Parent::Artboard(artboard), viewport_node)?;
        let mut content_node = layout("Content", 100.0, 140.0);
        let NodeSpec::LayoutComponent(content_spec) = &mut content_node else {
            unreachable!("layout helper always returns a LayoutComponent")
        };
        content_spec.style.flex_direction = SceneLayoutFlexDirection::Column;
        content_spec.style.padding_top = 5.0;
        content_spec.style.padding_top_units = SceneLayoutUnit::Point;
        content_spec.style.padding_bottom = 7.0;
        content_spec.style.padding_bottom_units = SceneLayoutUnit::Point;
        let content = tx.create(Parent::Object(viewport), content_node)?;
        tx.create(Parent::Object(content), layout("First", 100.0, 60.0))?;
        tx.create(Parent::Object(content), layout("Second", 100.0, 80.0))?;
        let constraint = tx.create_scroll_constraint(
            content,
            ScrollConstraintSpec {
                virtualize: true,
                ..ScrollConstraintSpec::default()
            },
        )?;
        Ok((artboard, constraint))
    })?;
    let instance = settle(&mut scene, artboard)?;

    let snapshot = scene.scroll_constraint_snapshot(instance, constraint)?;
    assert_eq!(
        snapshot.lower_bound,
        (0.0, -52.0),
        "virtualized non-infinite content is padding-inclusive at the pin: \
         min(0, viewport 100 - (children 140 + padding 12))",
    );
    let written =
        scene.set_scroll_property(instance, constraint, ScrollProperty::OffsetY, -200.0)?;
    assert_eq!(written.offset, (0.0, -200.0));
    assert_eq!(
        written.clamped_offset,
        (0.0, -52.0),
        "reads clamp to the padding-inclusive bound",
    );
    Ok(())
}

#[test]
fn scrolled_bounds_shift_along_transformed_ancestor_axes() -> Result<()> {
    // Pinned C++ post-multiplies the scroll translate onto the child's world
    // transform (`constrainChild`: `worldTransform * m_scrollTransform`,
    // `scroll_constraint.cpp:215-230` at
    // `4ac7b32798da0482e441ef09304dc3b480ed3ee5`), so under a rotated and
    // scaled ancestor the settled box shifts along the transformed axis:
    // an offset of -24 displaces by `linear * (0, -24)`.
    let rotation = 0.5_f32;
    let scale = 1.5_f32;
    let mut scene = Scene::new();
    let ((artboard, constraint, child2), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Transformed".into(),
            width: 400.0,
            height: 400.0,
        })?;
        let mut container_node = layout("Container", 120.0, 120.0);
        let NodeSpec::LayoutComponent(container_spec) = &mut container_node else {
            unreachable!("layout helper always returns a LayoutComponent")
        };
        container_spec.rotation = rotation;
        container_spec.scale_x = scale;
        container_spec.scale_y = scale;
        let container = tx.create(Parent::Artboard(artboard), container_node)?;
        let mut viewport_node = layout("Viewport", 100.0, 100.0);
        let NodeSpec::LayoutComponent(viewport_spec) = &mut viewport_node else {
            unreachable!("layout helper always returns a LayoutComponent")
        };
        viewport_spec.clip = true;
        let viewport = tx.create(Parent::Object(container), viewport_node)?;
        let mut content_node = layout("Content", 100.0, 140.0);
        let NodeSpec::LayoutComponent(content_spec) = &mut content_node else {
            unreachable!("layout helper always returns a LayoutComponent")
        };
        content_spec.style.flex_direction = SceneLayoutFlexDirection::Column;
        let content = tx.create(Parent::Object(viewport), content_node)?;
        tx.create(Parent::Object(content), layout("First", 100.0, 60.0))?;
        let child2 = tx.create(Parent::Object(content), layout("Second", 100.0, 80.0))?;
        let constraint = tx.create_scroll_constraint(content, ScrollConstraintSpec::default())?;
        Ok((artboard, constraint, child2))
    })?;
    let instance = settle(&mut scene, artboard)?;

    let before = scene.scrolled_layout_bounds(instance, child2)?;
    assert_eq!(
        before,
        scene.solved_layout_bounds(instance, child2)?,
        "a zero offset reads exactly the solved box, transformed ancestors or not",
    );
    let _ = scene.set_scroll_property(instance, constraint, ScrollProperty::OffsetY, -24.0)?;
    let after = scene.scrolled_layout_bounds(instance, child2)?;
    let expected_dx = 24.0 * scale * rotation.sin();
    let expected_dy = -24.0 * scale * rotation.cos();
    assert!(
        ((after.min_x - before.min_x) - expected_dx).abs() < 0.001,
        "dx {} must follow the transformed axis {expected_dx}",
        after.min_x - before.min_x,
    );
    assert!(
        ((after.min_y - before.min_y) - expected_dy).abs() < 0.001,
        "dy {} must follow the transformed axis {expected_dy}",
        after.min_y - before.min_y,
    );
    assert!((after.max_x - after.min_x - (before.max_x - before.min_x)).abs() < 0.001);
    assert!((after.max_y - after.min_y - (before.max_y - before.min_y)).abs() < 0.001);
    Ok(())
}

#[test]
fn scroll_writes_are_occurrence_scoped() -> Result<()> {
    let mut scene = Scene::new();
    let fixture = author_scroll_scene(&mut scene, ScrollConstraintSpec::default())?;
    let instance_a = settle(&mut scene, fixture.artboard)?;
    let instance_b = settle(&mut scene, fixture.artboard)?;

    let before_b = scene.scroll_constraint_snapshot(instance_b, fixture.constraint)?;
    let _ = scene.set_scroll_property(
        instance_a,
        fixture.constraint,
        ScrollProperty::OffsetY,
        -25.0,
    )?;

    let after_a = scene.scroll_constraint_snapshot(instance_a, fixture.constraint)?;
    assert_eq!(after_a.offset, (0.0, -25.0));
    let after_b = scene.scroll_constraint_snapshot(instance_b, fixture.constraint)?;
    assert_eq!(
        (after_b.offset, after_b.clamped_offset),
        (before_b.offset, before_b.clamped_offset),
        "a write on one occurrence must not leak into another",
    );
    assert_eq!(
        scene.scrolled_layout_bounds(instance_b, fixture.child2)?,
        Aabb::new(0.0, 60.0, 100.0, 140.0),
    );
    Ok(())
}

#[test]
fn scroll_reads_equal_plain_reads_without_a_live_constraint() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, container), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Plain".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let container = tx.create(Parent::Artboard(artboard), layout("Box", 40.0, 30.0))?;
        Ok((artboard, container))
    })?;
    let instance = settle(&mut scene, artboard)?;
    assert_eq!(
        scene.scrolled_layout_bounds(instance, container)?,
        scene.solved_layout_bounds(instance, container)?,
    );
    assert_eq!(
        scene
            .frame()
            .world_transform_with_scroll(instance, container),
        scene.frame().world_transform(instance, container),
    );
    Ok(())
}

#[test]
fn scroll_constraint_authoring_rejects_invalid_owners_and_values() -> Result<()> {
    let mut scene = Scene::new();
    let result = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            layout_style: None,
            name: "Invalid".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Not a layout".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let abort = tx
            .create_scroll_constraint(shape, ScrollConstraintSpec::default())
            .expect_err("a Shape cannot own a ScrollConstraint");
        assert_eq!(abort.diagnostic().reason, EditReason::UnknownObject);

        let viewport = tx.create(Parent::Artboard(artboard), layout("Viewport", 100.0, 100.0))?;
        let content = tx.create(Parent::Object(viewport), layout("Content", 100.0, 140.0))?;
        let abort = tx
            .create_scroll_constraint(
                content,
                ScrollConstraintSpec {
                    scroll_offset_y: f32::NAN,
                    ..ScrollConstraintSpec::default()
                },
            )
            .expect_err("non-finite offsets are rejected at author time");
        assert!(matches!(
            abort.diagnostic().reason,
            EditReason::NonFiniteProperty { .. }
        ));

        tx.create_scroll_constraint(content, ScrollConstraintSpec::default())?;
        let abort = tx
            .create_scroll_constraint(content, ScrollConstraintSpec::default())
            .expect_err("one content owns at most one authored ScrollConstraint");
        assert_eq!(abort.diagnostic().reason, EditReason::IdentityCollision);
        Ok(((), ()))
    });
    assert!(result.is_ok());

    let mut scene = Scene::new();
    let fixture = author_scroll_scene(&mut scene, ScrollConstraintSpec::default())?;
    let instance = settle(&mut scene, fixture.artboard)?;
    assert_eq!(
        scene
            .set_scroll_property(
                instance,
                fixture.constraint,
                ScrollProperty::OffsetY,
                f32::INFINITY,
            )
            .unwrap_err(),
        ResolveError::NonFiniteValue,
    );
    Ok(())
}

#[test]
fn scroll_direction_defaults_mirror_the_pinned_schema() {
    let spec = ScrollConstraintSpec::default();
    assert_eq!(spec.direction, ScrollConstraintDirection::Vertical);
    assert!(spec.interactive, "schema default interactive == true");
    assert_eq!(
        spec.drag_multiplier, 1.0,
        "schema default dragMultiplier == 1"
    );
    assert!(!spec.snap);
    assert!(!spec.virtualize);
    assert!(!spec.infinite);
    assert_eq!(spec.threshold, 0.0);
    assert_eq!(spec.scroll_offset_x, 0.0);
    assert_eq!(spec.scroll_offset_y, 0.0);
}
