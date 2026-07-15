use anyhow::Result;
use nuxie::{
    ArtboardId, ArtboardSpec, DashPathSpec, DashSpec, EditAbort, EditErrorKind, EditId, EditReason,
    ExportedObjectKind, ExportedProperty, FillSpec, NodeKind, NodeSpec, ObjectId, Parent,
    PropValueKind, RecordingFactory, RectangleCornerRadii, RectangleSpec, ResolveError, Scene,
    SceneStrokeCap, SceneStrokeJoin, SceneTx, ShapeSpec, SolidColorSpec, StaleCursor, StrokeSpec,
    StructureEpoch, props,
};

fn draw_stream(scene: &mut Scene, instance: nuxie::InstanceId) -> Result<String> {
    let mut factory = RecordingFactory::new();
    let mut cache = scene.new_render_cache(instance, &mut factory)?;
    let mut renderer = factory.make_renderer();
    scene
        .frame()
        .draw(instance, &mut factory, &mut renderer, &mut cache)?;
    Ok(factory.stream())
}

fn create_card(
    tx: &mut SceneTx<'_>,
    name: &str,
    color: u32,
) -> std::result::Result<(ArtboardId, ObjectId, ObjectId), EditAbort> {
    let artboard = tx.create_artboard(ArtboardSpec {
        name: name.into(),
        width: 100.0,
        height: 100.0,
    })?;
    let shape = tx.create(
        Parent::Artboard(artboard),
        NodeSpec::Shape(ShapeSpec {
            name: format!("{name} Card"),
            x: 50.0,
            y: 50.0,
            opacity: 1.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        }),
    )?;
    let rectangle = tx.create(
        Parent::Object(shape),
        NodeSpec::Rectangle(RectangleSpec::new(format!("{name} Rectangle"), 80.0, 60.0)),
    )?;
    let fill = tx.create(
        Parent::Object(shape),
        NodeSpec::Fill(FillSpec {
            name: format!("{name} Fill"),
        }),
    )?;
    let color = tx.create(
        Parent::Object(fill),
        NodeSpec::SolidColor(SolidColorSpec {
            name: format!("{name} Color"),
            color,
        }),
    )?;
    Ok((artboard, rectangle, color))
}

#[test]
fn a_removed_subtree_restores_the_same_objects_records_and_draw_in_a_later_edit() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle, _fill, color), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 80.0, 60.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        let color = tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard, shape, rectangle, fill, color))
    })?;
    let instance = scene.instantiate(artboard)?;
    let before_records = scene.export_records();
    let before_draw = draw_stream(&mut scene, instance)?;

    let (removed, remove_receipt) = scene.edit(|tx| tx.remove(shape))?;
    assert!(remove_receipt.created.is_empty());
    assert!(matches!(
        scene.cursor(instance, shape, props::WORLD_OPACITY),
        Err(ResolveError::UnknownObject)
    ));
    assert!(matches!(
        scene.cursor(instance, rectangle, props::PATH_WIDTH),
        Err(ResolveError::UnknownObject)
    ));
    assert!(matches!(
        scene.cursor(instance, color, props::COLOR_VALUE),
        Err(ResolveError::UnknownObject)
    ));
    assert_ne!(draw_stream(&mut scene, instance)?, before_draw);

    let (restored_root, restore_receipt) = scene.edit(|tx| tx.restore(removed))?;
    assert_eq!(restored_root, shape);
    assert!(restore_receipt.created.is_empty());
    assert_eq!(scene.export_records(), before_records);
    assert_eq!(draw_stream(&mut scene, instance)?, before_draw);
    assert!(scene.cursor(instance, shape, props::WORLD_OPACITY).is_ok());
    assert!(scene.cursor(instance, rectangle, props::PATH_WIDTH).is_ok());
    assert!(scene.cursor(instance, color, props::COLOR_VALUE).is_ok());
    Ok(())
}

#[test]
fn restore_reinserts_interleaved_descendants_at_their_original_record_positions() -> Result<()> {
    let mut scene = Scene::new();
    let ((shape_a, _shape_b, _rectangle_a, _rectangle_b), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let mut create_shape = |name: &str, x: f32| {
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: name.into(),
                    x,
                    y: 50.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )
        };
        let shape_a = create_shape("A", 25.0)?;
        let shape_b = create_shape("B", 75.0)?;
        let rectangle_a = tx.create(
            Parent::Object(shape_a),
            NodeSpec::Rectangle(RectangleSpec::new("A Rectangle", 20.0, 20.0)),
        )?;
        let rectangle_b = tx.create(
            Parent::Object(shape_b),
            NodeSpec::Rectangle(RectangleSpec::new("B Rectangle", 30.0, 30.0)),
        )?;
        Ok((shape_a, shape_b, rectangle_a, rectangle_b))
    })?;
    let records_before = scene.export_records();

    let (removed, _) = scene.edit(|tx| tx.remove(shape_a))?;
    scene.edit(|tx| tx.restore(removed))?;

    assert_eq!(scene.export_records(), records_before);
    Ok(())
}

#[test]
fn restoring_an_existing_identity_aborts_the_entire_edit_with_a_collision_diagnostic() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, rectangle, color), _) = scene.edit(|tx| create_card(tx, "Main", 0xff112233))?;
    let instance = scene.instantiate(artboard)?;
    let (removed, _) = scene.edit(|tx| tx.remove(rectangle))?;
    scene.edit(|tx| tx.restore(removed.clone()))?;
    let color_cursor = scene.cursor(instance, color, props::COLOR_VALUE)?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();
    let draw_before = draw_stream(&mut scene, instance)?;

    let error = scene
        .edit(|tx| {
            tx.set(color, props::COLOR_VALUE, 0xff445566)?;
            tx.restore(removed)?;
            Ok(())
        })
        .expect_err("the restored rectangle identity already exists");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 1);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Object(rectangle)]
    );
    assert_eq!(error.diagnostic().reason, EditReason::IdentityCollision);
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    assert_eq!(draw_stream(&mut scene, instance)?, draw_before);
    assert_eq!(scene.frame().get(color_cursor)?, 0xff112233);
    Ok(())
}

#[test]
fn remove_then_restore_in_one_edit_is_an_exact_structural_commit_that_stales_all_cursors()
-> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, rectangle, color), _) = scene.edit(|tx| create_card(tx, "Main", 0xff112233))?;
    let instance = scene.instantiate(artboard)?;
    let old_color_cursor = scene.cursor(instance, color, props::COLOR_VALUE)?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();
    let draw_before = draw_stream(&mut scene, instance)?;

    let (restored, receipt) = scene.edit(|tx| {
        let removed = tx.remove(rectangle)?;
        tx.restore(removed)
    })?;

    assert_eq!(restored, rectangle);
    assert_eq!(receipt.epoch, scene.epoch());
    assert_eq!(scene.epoch().get(), epoch_before.get() + 1);
    assert!(receipt.created.is_empty());
    assert_eq!(scene.export_records(), records_before);
    assert_eq!(draw_stream(&mut scene, instance)?, draw_before);
    assert_eq!(scene.frame().get(old_color_cursor), Err(StaleCursor));
    let fresh_color_cursor = scene.cursor(instance, color, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(fresh_color_cursor)?, 0xff112233);
    Ok(())
}

#[test]
fn edit_receipts_exclude_objects_created_and_removed_before_commit() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    let epoch_before = scene.epoch();

    let ((shape, removed), receipt) = scene.edit(|tx| {
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Transient".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let removed = tx.remove(shape)?;
        Ok((shape, removed))
    })?;

    assert!(receipt.created.is_empty());
    assert_eq!(scene.epoch().get(), epoch_before.get() + 1);
    let (restored, restore_receipt) = scene.edit(|tx| tx.restore(removed))?;
    assert_eq!(restored, shape);
    assert!(
        restore_receipt.created.is_empty(),
        "restoring an existing identity never reports a new allocation"
    );
    Ok(())
}

#[test]
fn restore_rejects_a_missing_original_parent_with_structured_diagnostics() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 80.0, 60.0)),
        )?;
        Ok((artboard, shape, rectangle))
    })?;
    let (removed_rectangle, _) = scene.edit(|tx| tx.remove(rectangle))?;
    let (_removed_shape, _) = scene.edit(|tx| tx.remove(shape))?;
    let epoch_before = scene.epoch();
    let records_before = scene.export_records();

    let error = scene
        .edit(|tx| tx.restore(removed_rectangle))
        .expect_err("the rectangle's original shape parent no longer exists");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Object(rectangle), EditId::Object(shape)]
    );
    assert_eq!(error.diagnostic().reason, EditReason::UnknownObject);
    assert_eq!(scene.epoch(), epoch_before);
    assert_eq!(scene.export_records(), records_before);
    assert!(scene.instantiate(artboard).is_ok());
    Ok(())
}

#[test]
fn restoring_a_subtree_into_another_scene_rejects_its_owner_and_rolls_back_the_edit() -> Result<()>
{
    let mut source = Scene::new();
    let ((source_artboard, source_rectangle, _), _) =
        source.edit(|tx| create_card(tx, "Source", 0xff112233))?;
    let (removed, _) = source.edit(|tx| tx.remove(source_rectangle))?;

    let mut target = Scene::new();
    let ((target_artboard, _, target_color), _) =
        target.edit(|tx| create_card(tx, "Target", 0xff223344))?;
    let target_instance = target.instantiate(target_artboard)?;
    let target_cursor = target.cursor(target_instance, target_color, props::COLOR_VALUE)?;
    let epoch_before = target.epoch();
    let records_before = target.export_records();
    let draw_before = draw_stream(&mut target, target_instance)?;

    let error = target
        .edit(|tx| {
            tx.set(target_color, props::COLOR_VALUE, 0xff556677)?;
            tx.restore(removed)?;
            Ok(())
        })
        .expect_err("a removed subtree remains owned by its original scene artboard");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 1);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![
            EditId::Artboard(source_artboard),
            EditId::Object(source_rectangle)
        ]
    );
    assert_eq!(error.diagnostic().reason, EditReason::UnknownArtboard);
    assert_eq!(target.epoch(), epoch_before);
    assert_eq!(target.export_records(), records_before);
    assert_eq!(draw_stream(&mut target, target_instance)?, draw_before);
    assert_eq!(target.frame().get(target_cursor)?, 0xff223344);
    Ok(())
}

#[test]
fn removing_from_one_artboard_preserves_another_artboards_hot_state_and_held_cache() -> Result<()> {
    let mut scene = Scene::new();
    let ((_, rectangle_a, _, artboard_b, _, color_b), _) = scene.edit(|tx| {
        let (artboard_a, rectangle_a, color_a) = create_card(tx, "A", 0xff112233)?;
        let (artboard_b, rectangle_b, color_b) = create_card(tx, "B", 0xff223344)?;
        Ok((
            artboard_a,
            rectangle_a,
            color_a,
            artboard_b,
            rectangle_b,
            color_b,
        ))
    })?;
    let instance_b = scene.instantiate(artboard_b)?;
    let old_cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(old_cursor_b, 0xff556677)?);

    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let draw_before = factory_b.stream();
    assert!(draw_before.contains("ff556677"));

    scene.edit(|tx| tx.remove(rectangle_a))?;

    assert_eq!(scene.frame().get(old_cursor_b), Err(StaleCursor));
    let fresh_cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(fresh_cursor_b)?, 0xff556677);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(
        factory_b.stream(),
        draw_before,
        "the untouched artboard must retain both its live instance and held cache"
    );
    Ok(())
}

#[test]
fn authored_scene_uses_typed_cursor_writes_and_stales_them_after_structure_changes() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle, color), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 80.0, 60.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        let color = tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard, shape, rectangle, color))
    })?;

    let instance = scene.instantiate(artboard)?;
    let color_cursor = scene.cursor(instance, color, props::COLOR_VALUE)?;
    let opacity_cursor = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let rotation_cursor = scene.cursor(instance, shape, props::ROTATION)?;
    let before = draw_stream(&mut scene, instance)?;

    assert!(scene.frame().set(opacity_cursor, 0.5)?);
    assert!(scene.frame().set(rotation_cursor, 0.25)?);
    assert!(scene.frame().set(color_cursor, 0xff445566)?);
    let after = draw_stream(&mut scene, instance)?;
    assert_ne!(
        after, before,
        "the cursor write must change rendered output"
    );
    assert!(
        !scene.frame().set(opacity_cursor, f32::NAN)?,
        "invalid hot values are rejected as unchanged"
    );
    assert_eq!(
        draw_stream(&mut scene, instance)?,
        after,
        "a rejected hot write must leave the live graph valid"
    );

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;
    assert_eq!(
        scene.frame().set(color_cursor, 0xff778899),
        Err(StaleCursor)
    );
    Ok(())
}

#[test]
fn a_cursor_can_never_write_to_another_scene_with_matching_slots_and_epoch() -> Result<()> {
    let mut left = Scene::new();
    let ((left_artboard, _, left_color), _) =
        left.edit(|tx| create_card(tx, "Left", 0xff112233))?;
    let left_instance = left.instantiate(left_artboard)?;
    let left_cursor = left.cursor(left_instance, left_color, props::COLOR_VALUE)?;

    let mut right = Scene::new();
    let ((right_artboard, _, _), _) = right.edit(|tx| create_card(tx, "Right", 0xff445566))?;
    let right_instance = right.instantiate(right_artboard)?;
    assert_eq!(left.epoch(), right.epoch());

    let before = draw_stream(&mut right, right_instance)?;
    assert_eq!(right.frame().set(left_cursor, 0xffaabbcc), Err(StaleCursor));
    assert_eq!(draw_stream(&mut right, right_instance)?, before);
    Ok(())
}

#[test]
fn public_ids_from_one_scene_never_alias_same_shaped_objects_or_instances_in_another() -> Result<()>
{
    let mut left = Scene::new();
    let ((left_artboard, _, left_color), _) =
        left.edit(|tx| create_card(tx, "Left", 0xff112233))?;
    let left_instance = left.instantiate(left_artboard)?;

    let mut right = Scene::new();
    let ((right_artboard, _, right_color), _) =
        right.edit(|tx| create_card(tx, "Right", 0xff445566))?;
    let right_instance = right.instantiate(right_artboard)?;
    let right_cursor = right.cursor(right_instance, right_color, props::COLOR_VALUE)?;

    assert!(right.instantiate(left_artboard).is_err());
    assert!(matches!(
        right.cursor(right_instance, left_color, props::COLOR_VALUE),
        Err(ResolveError::UnknownObject)
    ));
    let error = right
        .edit(|tx| {
            tx.set(left_color, props::COLOR_VALUE, 0xffaabbcc)?;
            Ok(())
        })
        .expect_err("an object id from another scene must not target a same-shaped object");
    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().reason, EditReason::UnknownObject);

    right.drop_instance(left_instance);
    assert_eq!(right.frame().get(right_cursor)?, 0xff445566);
    Ok(())
}

#[test]
fn frame_get_reads_schema_defaults_and_current_hot_values() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, shape, color), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        let color = tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard, shape, color))
    })?;
    let instance = scene.instantiate(artboard)?;
    let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
    let rotation = scene.cursor(instance, shape, props::ROTATION)?;
    let scale_x = scene.cursor(instance, shape, props::SCALE_X)?;
    let color = scene.cursor(instance, color, props::COLOR_VALUE)?;

    {
        let frame = scene.frame();
        assert_eq!(frame.get(opacity)?, 1.0);
        assert_eq!(frame.get(rotation)?, 0.0);
        assert_eq!(frame.get(scale_x)?, 1.0);
        assert_eq!(frame.get(color)?, 0xff112233);
    }

    {
        let mut frame = scene.frame();
        assert!(frame.set(opacity, 0.25)?);
        assert!(frame.set(color, 0xff445566)?);
        assert_eq!(frame.get(opacity)?, 0.25);
        assert_eq!(frame.get(color)?, 0xff445566);
    }
    Ok(())
}

#[test]
fn frame_get_rejects_stale_and_foreign_scene_cursors() -> Result<()> {
    let mut left = Scene::new();
    let ((left_artboard, left_rectangle, left_color), _) =
        left.edit(|tx| create_card(tx, "Left", 0xff112233))?;
    let left_instance = left.instantiate(left_artboard)?;
    let left_cursor = left.cursor(left_instance, left_color, props::COLOR_VALUE)?;

    let mut right = Scene::new();
    let ((right_artboard, _, _), _) = right.edit(|tx| create_card(tx, "Right", 0xff445566))?;
    right.instantiate(right_artboard)?;
    assert_eq!(left.epoch(), right.epoch());
    assert_eq!(right.frame().get(left_cursor), Err(StaleCursor));

    left.edit(|tx| {
        tx.set(left_rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;
    assert_eq!(left.frame().get(left_cursor), Err(StaleCursor));
    Ok(())
}

#[test]
fn dropped_instance_slots_never_alias_old_cursors_and_do_not_disturb_other_instances() -> Result<()>
{
    let mut scene = Scene::new();
    let ((artboard, rectangle, color), _) = scene.edit(|tx| create_card(tx, "Main", 0xff112233))?;
    let dropped = scene.instantiate(artboard)?;
    let survivor = scene.instantiate(artboard)?;
    let dropped_cursor = scene.cursor(dropped, color, props::COLOR_VALUE)?;
    let survivor_cursor = scene.cursor(survivor, color, props::COLOR_VALUE)?;
    let epoch = scene.epoch();

    scene.drop_instance(dropped);
    assert_eq!(
        scene.epoch(),
        epoch,
        "instance lifecycle is not a definition edit"
    );
    assert_eq!(scene.frame().get(dropped_cursor), Err(StaleCursor));
    assert_eq!(
        scene.frame().set(dropped_cursor, 0xff445566),
        Err(StaleCursor)
    );
    assert!(scene.frame().set(survivor_cursor, 0xff556677)?);
    assert_eq!(scene.frame().get(survivor_cursor)?, 0xff556677);

    let replacement = scene.instantiate(artboard)?;
    let replacement_cursor = scene.cursor(replacement, color, props::COLOR_VALUE)?;
    assert_ne!(replacement, dropped, "instance ids are never reused");
    assert_eq!(scene.frame().get(dropped_cursor), Err(StaleCursor));
    assert_eq!(scene.frame().get(replacement_cursor)?, 0xff112233);
    assert!(scene.frame().set(replacement_cursor, 0xff667788)?);
    assert_eq!(scene.frame().get(replacement_cursor)?, 0xff667788);

    scene.drop_instance(dropped);
    assert_eq!(scene.frame().get(survivor_cursor)?, 0xff556677);

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;
    assert_eq!(scene.frame().get(survivor_cursor), Err(StaleCursor));
    assert_eq!(scene.frame().get(replacement_cursor), Err(StaleCursor));
    let survivor_cursor = scene.cursor(survivor, color, props::COLOR_VALUE)?;
    let replacement_cursor = scene.cursor(replacement, color, props::COLOR_VALUE)?;
    assert_eq!(scene.frame().get(survivor_cursor)?, 0xff112233);
    assert_eq!(scene.frame().get(replacement_cursor)?, 0xff112233);
    Ok(())
}

#[test]
fn aborted_edits_burn_allocated_identity_without_publishing_definitions() -> Result<()> {
    let mut scene = Scene::new();
    let mut leaked_artboard = None;

    let aborted = scene.edit::<()>(|tx| {
        leaked_artboard = Some(tx.create_artboard(ArtboardSpec {
            name: "Aborted".into(),
            width: 100.0,
            height: 100.0,
        })?);
        Err(tx.abort("abort the transaction"))
    });
    assert!(aborted.is_err());

    let leaked_artboard = leaked_artboard.expect("the closure allocated an identity");
    let (committed_artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Committed".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;

    assert_ne!(committed_artboard, leaked_artboard);
    assert!(scene.instantiate(leaked_artboard).is_err());
    assert!(scene.instantiate(committed_artboard).is_ok());
    Ok(())
}

#[test]
fn edit_receipts_report_structure_epoch_and_only_created_object_ids() -> Result<()> {
    let mut scene = Scene::new();
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);

    let ((artboard, shape), receipt) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok((artboard, shape))
    })?;

    assert_eq!(receipt.epoch, scene.epoch());
    assert_eq!(receipt.created, vec![shape]);
    assert_eq!(scene.epoch().get(), 1);

    // Artboard identity is deliberately not exposed as an ObjectId in the receipt.
    assert!(scene.instantiate(artboard).is_ok());
    Ok(())
}

#[test]
fn local_validation_returns_a_structured_abort_diagnostic() -> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    let epoch_before = scene.epoch();

    let error = scene
        .edit(|tx| {
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Rectangle(RectangleSpec::new("Invalid child", 10.0, 10.0)),
            )?;
            Ok(())
        })
        .expect_err("a Rectangle cannot be parented directly to an Artboard");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Artboard(artboard)]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::InvalidParent {
            parent: None,
            child: NodeKind::Rectangle,
        }
    );
    assert_eq!(scene.epoch(), epoch_before, "an abort is atomic");
    Ok(())
}

#[test]
fn materialization_failure_is_a_structured_edit_error_without_internal_details() -> Result<()> {
    let mut scene = Scene::new();
    let mut invalid_artboard = None;
    let error = scene
        .edit(|tx| {
            invalid_artboard = Some(tx.create_artboard(ArtboardSpec {
                name: "Invalid".into(),
                width: f32::NAN,
                height: 100.0,
            })?);
            Ok(())
        })
        .expect_err("non-finite geometry must be rejected at commit");

    assert_eq!(error.kind(), EditErrorKind::CommitRejected);
    assert_eq!(
        error.diagnostic().operation_index,
        0,
        "commit validation must point to the operation that introduced the invalid spec"
    );
    assert_eq!(
        error.diagnostic().involved_ids,
        vec![EditId::Artboard(
            invalid_artboard.expect("the transaction allocated the artboard")
        )]
    );
    assert_eq!(
        error.diagnostic().reason,
        EditReason::NonFiniteProperty { property: "width" }
    );
    assert_eq!(error.to_string(), "scene edit was rejected during commit");
    assert_eq!(scene.epoch(), StructureEpoch::INITIAL);
    Ok(())
}

#[test]
fn generated_authoring_vocabulary_tracks_schema_owners_value_kinds_and_surface_availability() {
    assert_eq!(NodeKind::Rectangle.schema_name(), "Rectangle");

    assert_eq!(props::PATH_WIDTH.schema_name(), "width");
    assert!(props::PATH_WIDTH.is_available_on(NodeKind::Rectangle));
    assert_eq!(props::PATH_WIDTH.value_kind(), PropValueKind::Double);
    assert_eq!(props::PATH_WIDTH.declared_owner(), "ParametricPath");

    assert_eq!(props::COLOR_VALUE.schema_name(), "colorValue");
    assert!(props::COLOR_VALUE.is_available_on(NodeKind::SolidColor));
    assert_eq!(props::COLOR_VALUE.value_kind(), PropValueKind::Color);
    assert_eq!(props::COLOR_VALUE.declared_owner(), "SolidColor");

    assert_eq!(props::WORLD_OPACITY.schema_name(), "opacity");
    assert!(props::WORLD_OPACITY.is_available_on(NodeKind::Shape));
    assert_eq!(props::WORLD_OPACITY.value_kind(), PropValueKind::Double);
    assert_eq!(
        props::WORLD_OPACITY.declared_owner(),
        "WorldTransformComponent"
    );

    for property in [props::TRANSLATE_X, props::TRANSLATE_Y] {
        assert!(property.is_available_on(NodeKind::Shape));
        assert_eq!(property.value_kind(), PropValueKind::Double);
        assert_eq!(property.declared_owner(), "Node");
    }

    for property in [props::ROTATION, props::SCALE_X, props::SCALE_Y] {
        assert!(property.is_available_on(NodeKind::Shape));
        assert_eq!(property.value_kind(), PropValueKind::Double);
        assert_eq!(property.declared_owner(), "TransformComponent");
    }
}

#[test]
fn export_records_are_sparse_canonical_and_compose_one_backboard() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let first = tx.create_artboard(ArtboardSpec {
            name: "First".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let default_shape = tx.create(
            Parent::Artboard(first),
            NodeSpec::Shape(ShapeSpec {
                name: "Default".into(),
                x: 10.0,
                y: 20.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(default_shape),
            NodeSpec::Rectangle(RectangleSpec::new("Default Rectangle", 40.0, 30.0)),
        )?;
        tx.create(
            Parent::Artboard(first),
            NodeSpec::Shape(ShapeSpec {
                name: "Complex".into(),
                x: 30.0,
                y: 40.0,
                opacity: 0.4,
                rotation: 0.25,
                scale_x: 1.5,
                scale_y: 0.5,
            }),
        )?;
        tx.create_artboard(ArtboardSpec {
            name: "Second".into(),
            width: 200.0,
            height: 150.0,
        })?;
        Ok(())
    })?;

    let exported = scene.export_records();
    let records = exported.records();
    assert_eq!(
        records.iter().map(|record| record.kind).collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::Backboard,
            ExportedObjectKind::Artboard,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Rectangle,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Artboard,
        ]
    );
    assert_eq!(
        records
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::Backboard)
            .count(),
        1
    );
    let properties = |index: usize| {
        records
            .get(index)
            .map(|record| record.properties.clone())
            .unwrap_or_default()
    };
    assert_eq!(
        properties(2),
        vec![
            ExportedProperty::ComponentName("Default".into()),
            ExportedProperty::TranslateX(10.0),
            ExportedProperty::TranslateY(20.0),
        ],
        "root parent and identity transform defaults are omitted"
    );
    assert_eq!(
        properties(3),
        vec![
            ExportedProperty::ComponentName("Default Rectangle".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::PathWidth(40.0),
            ExportedProperty::PathHeight(30.0),
        ],
        "non-root parent is present and properties are canonical without exposing schema keys"
    );
    assert_eq!(
        properties(4),
        vec![
            ExportedProperty::ComponentName("Complex".into()),
            ExportedProperty::TranslateX(30.0),
            ExportedProperty::TranslateY(40.0),
            ExportedProperty::Rotation(0.25),
            ExportedProperty::ScaleX(1.5),
            ExportedProperty::ScaleY(0.5),
            ExportedProperty::WorldOpacity(0.4),
        ]
    );
    Ok(())
}

#[test]
fn sparse_export_omits_only_exact_schema_defaults_across_remounts() -> Result<()> {
    let next_below_one = f32::from_bits(1.0f32.to_bits() - 1);
    let tiny_rotation = f32::EPSILON / 2.0;
    let mut scene = Scene::new();
    let ((artboard, shape, rectangle), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Near defaults".into(),
                x: 0.0,
                y: 0.0,
                opacity: next_below_one,
                rotation: tiny_rotation,
                scale_x: next_below_one,
                scale_y: next_below_one,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Rectangle", 80.0, 60.0)),
        )?;
        Ok((artboard, shape, rectangle))
    })?;

    let records = scene.export_records();
    assert_eq!(
        records.records()[2].properties,
        vec![
            ExportedProperty::ComponentName("Near defaults".into()),
            ExportedProperty::TranslateX(0.0),
            ExportedProperty::TranslateY(0.0),
            ExportedProperty::Rotation(tiny_rotation),
            ExportedProperty::ScaleX(next_below_one),
            ExportedProperty::ScaleY(next_below_one),
            ExportedProperty::WorldOpacity(next_below_one),
        ],
        "representable values adjacent to schema defaults must not be elided"
    );

    let instance = scene.instantiate(artboard)?;
    let assert_near_defaults = |scene: &mut Scene| -> Result<()> {
        let opacity = scene.cursor(instance, shape, props::WORLD_OPACITY)?;
        let rotation = scene.cursor(instance, shape, props::ROTATION)?;
        let scale_x = scene.cursor(instance, shape, props::SCALE_X)?;
        let scale_y = scene.cursor(instance, shape, props::SCALE_Y)?;
        let frame = scene.frame();
        assert_eq!(frame.get(opacity)?, next_below_one);
        assert_eq!(frame.get(rotation)?, tiny_rotation);
        assert_eq!(frame.get(scale_x)?, next_below_one);
        assert_eq!(frame.get(scale_y)?, next_below_one);
        Ok(())
    };
    assert_near_defaults(&mut scene)?;

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;
    assert_near_defaults(&mut scene)?;
    Ok(())
}

#[test]
fn typed_scene_materializes_rectangle_radii_and_a_dashed_stroke_without_raw_schema_keys()
-> Result<()> {
    let mut scene = Scene::new();
    let (artboard, _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Border".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Border Shape".into(),
                x: 5.0,
                y: 5.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec {
                name: "Border Rectangle".into(),
                width: 8.0,
                height: 8.0,
                corner_radii: Some(RectangleCornerRadii {
                    top_left: 1.0,
                    top_right: 2.0,
                    bottom_right: 3.0,
                    bottom_left: 4.0,
                    linked: false,
                }),
            }),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Border Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Fill Color".into(),
                color: 0xff112233,
            }),
        )?;
        let stroke = tx.create(
            Parent::Object(shape),
            NodeSpec::Stroke(StrokeSpec {
                name: "Border Stroke".into(),
                thickness: 2.0,
                cap: SceneStrokeCap::Butt,
                join: SceneStrokeJoin::Miter,
                transform_affects_stroke: true,
            }),
        )?;
        tx.create(
            Parent::Object(stroke),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Stroke Color".into(),
                color: 0xff445566,
            }),
        )?;
        let dash_path = tx.create(
            Parent::Object(stroke),
            NodeSpec::DashPath(DashPathSpec {
                name: "Dash Path".into(),
                offset: 0.0,
                offset_is_percentage: false,
            }),
        )?;
        for (index, length) in [0.5, 0.5].into_iter().enumerate() {
            tx.create(
                Parent::Object(dash_path),
                NodeSpec::Dash(DashSpec {
                    name: format!("Dash {index}"),
                    length,
                    length_is_percentage: true,
                }),
            )?;
        }
        Ok(artboard)
    })?;

    let records = scene.export_records();
    let [
        _,
        _,
        _,
        rectangle,
        _,
        _,
        stroke,
        stroke_color,
        dash_path,
        dash_on,
        dash_off,
    ] = records.records()
    else {
        panic!("border scene must export exactly eleven records");
    };
    assert_eq!(
        records
            .records()
            .iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
        vec![
            ExportedObjectKind::Backboard,
            ExportedObjectKind::Artboard,
            ExportedObjectKind::Shape,
            ExportedObjectKind::Rectangle,
            ExportedObjectKind::Fill,
            ExportedObjectKind::SolidColor,
            ExportedObjectKind::Stroke,
            ExportedObjectKind::SolidColor,
            ExportedObjectKind::DashPath,
            ExportedObjectKind::Dash,
            ExportedObjectKind::Dash,
        ]
    );
    assert_eq!(
        rectangle.properties,
        vec![
            ExportedProperty::ComponentName("Border Rectangle".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::PathWidth(8.0),
            ExportedProperty::PathHeight(8.0),
            ExportedProperty::RectangleCornerRadiusTopLeft(1.0),
            ExportedProperty::RectangleCornerRadiusTopRight(2.0),
            ExportedProperty::RectangleCornerRadiusBottomLeft(4.0),
            ExportedProperty::RectangleCornerRadiusBottomRight(3.0),
            ExportedProperty::RectangleLinkCornerRadius(false),
        ]
    );
    assert_eq!(
        stroke.properties,
        vec![
            ExportedProperty::ComponentName("Border Stroke".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::StrokeThickness(2.0),
            ExportedProperty::StrokeCap(SceneStrokeCap::Butt),
            ExportedProperty::StrokeJoin(SceneStrokeJoin::Miter),
            ExportedProperty::StrokeTransformAffectsStroke(true),
        ]
    );
    assert_eq!(
        stroke_color.properties,
        vec![
            ExportedProperty::ComponentName("Stroke Color".into()),
            ExportedProperty::ParentId(5),
            ExportedProperty::ColorValue(0xff445566),
        ]
    );
    assert_eq!(
        dash_path.properties,
        vec![
            ExportedProperty::ComponentName("Dash Path".into()),
            ExportedProperty::ParentId(5),
            ExportedProperty::DashOffset(0.0),
            ExportedProperty::DashOffsetIsPercentage(false),
        ]
    );
    assert_eq!(
        dash_on.properties,
        vec![
            ExportedProperty::ComponentName("Dash 0".into()),
            ExportedProperty::ParentId(7),
            ExportedProperty::DashLength(0.5),
            ExportedProperty::DashLengthIsPercentage(true),
        ]
    );
    assert_eq!(
        dash_off.properties,
        vec![
            ExportedProperty::ComponentName("Dash 1".into()),
            ExportedProperty::ParentId(7),
            ExportedProperty::DashLength(0.5),
            ExportedProperty::DashLengthIsPercentage(true),
        ]
    );

    let instance = scene.instantiate(artboard)?;
    let stream = draw_stream(&mut scene, instance)?;
    assert!(stream.contains("style=stroke"));
    assert!(stream.contains("color=0xff445566"));
    Ok(())
}

#[test]
fn explicit_all_zero_rectangle_radii_remain_present_in_typed_export() -> Result<()> {
    let mut scene = Scene::new();
    scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Zero Radius".into(),
            width: 10.0,
            height: 10.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Shape".into(),
                x: 5.0,
                y: 5.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec {
                name: "Rectangle".into(),
                width: 8.0,
                height: 8.0,
                corner_radii: Some(RectangleCornerRadii {
                    top_left: 0.0,
                    top_right: 0.0,
                    bottom_right: 0.0,
                    bottom_left: 0.0,
                    linked: false,
                }),
            }),
        )?;
        Ok(())
    })?;

    let records = scene.export_records();
    let [_, _, _, rectangle] = records.records() else {
        panic!("zero-radius scene must export exactly four records");
    };
    assert_eq!(
        rectangle.properties,
        vec![
            ExportedProperty::ComponentName("Rectangle".into()),
            ExportedProperty::ParentId(1),
            ExportedProperty::PathWidth(8.0),
            ExportedProperty::PathHeight(8.0),
            ExportedProperty::RectangleCornerRadiusTopLeft(0.0),
            ExportedProperty::RectangleCornerRadiusTopRight(0.0),
            ExportedProperty::RectangleCornerRadiusBottomLeft(0.0),
            ExportedProperty::RectangleCornerRadiusBottomRight(0.0),
            ExportedProperty::RectangleLinkCornerRadius(false),
        ]
    );
    Ok(())
}

#[test]
fn generated_transform_props_keep_local_validation_typed_and_atomic() -> Result<()> {
    let mut scene = Scene::new();
    let ((_, shape), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 0.0,
                y: 0.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        Ok((artboard, shape))
    })?;
    let epoch_before = scene.epoch();

    let error = scene
        .edit(|tx| {
            tx.set(shape, props::SCALE_X, f32::NAN)?;
            Ok(())
        })
        .expect_err("non-finite generated property writes must abort locally");

    assert_eq!(error.kind(), EditErrorKind::Aborted);
    assert_eq!(error.diagnostic().operation_index, 0);
    assert_eq!(error.diagnostic().involved_ids, vec![EditId::Object(shape)]);
    assert_eq!(
        error.diagnostic().reason,
        EditReason::NonFiniteProperty {
            property: "scale_x"
        }
    );
    assert_eq!(scene.epoch(), epoch_before);
    Ok(())
}

#[test]
fn failed_materialization_burns_allocated_identity_without_changing_the_scene() -> Result<()> {
    let mut scene = Scene::new();
    let (initial_artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Initial".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    let initial_instance = scene.instantiate(initial_artboard)?;
    let before = draw_stream(&mut scene, initial_instance)?;

    let mut leaked_artboard = None;
    let failed = scene.edit(|tx| {
        leaked_artboard = Some(tx.create_artboard(ArtboardSpec {
            name: "Invalid".into(),
            width: f32::NAN,
            height: 100.0,
        })?);
        Ok(())
    });
    assert!(failed.is_err(), "non-finite geometry must not materialize");

    let leaked_artboard = leaked_artboard.expect("the closure allocated an identity");
    assert_eq!(draw_stream(&mut scene, initial_instance)?, before);
    assert!(scene.instantiate(leaked_artboard).is_err());

    let (committed_artboard, _) = scene.edit(|tx| {
        tx.create_artboard(ArtboardSpec {
            name: "Committed".into(),
            width: 100.0,
            height: 100.0,
        })
    })?;
    assert_ne!(committed_artboard, leaked_artboard);
    Ok(())
}

#[test]
fn render_cache_held_across_a_structural_remount_matches_a_fresh_cache() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard, rectangle), _) = scene.edit(|tx| {
        let artboard = tx.create_artboard(ArtboardSpec {
            name: "Main".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: "Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle = tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new("Card Rectangle", 80.0, 60.0)),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: "Card Fill".into(),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "Card Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard, rectangle))
    })?;
    let instance = scene.instantiate(artboard)?;

    let mut original_factory = RecordingFactory::new();
    let mut held_cache = scene.new_render_cache(instance, &mut original_factory)?;
    let mut original_renderer = original_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut original_factory,
        &mut original_renderer,
        &mut held_cache,
    )?;

    scene.edit(|tx| {
        tx.set(rectangle, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;

    let mut refreshed_factory = RecordingFactory::new();
    let mut refreshed_renderer = refreshed_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut refreshed_factory,
        &mut refreshed_renderer,
        &mut held_cache,
    )?;

    let mut fresh_factory = RecordingFactory::new();
    let mut fresh_cache = scene.new_render_cache(instance, &mut fresh_factory)?;
    let mut fresh_renderer = fresh_factory.make_renderer();
    scene.frame().draw(
        instance,
        &mut fresh_factory,
        &mut fresh_renderer,
        &mut fresh_cache,
    )?;

    assert_eq!(refreshed_factory.stream(), fresh_factory.stream());
    Ok(())
}

#[test]
fn editing_one_artboard_preserves_another_artboards_hot_state_and_held_cache() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard_a, rectangle_a, color_a, artboard_b, color_b), _) = scene.edit(|tx| {
        let artboard_a = tx.create_artboard(ArtboardSpec {
            name: "A".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape_a = tx.create(
            Parent::Artboard(artboard_a),
            NodeSpec::Shape(ShapeSpec {
                name: "A Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        let rectangle_a = tx.create(
            Parent::Object(shape_a),
            NodeSpec::Rectangle(RectangleSpec::new("A Rectangle", 80.0, 60.0)),
        )?;
        let fill_a = tx.create(
            Parent::Object(shape_a),
            NodeSpec::Fill(FillSpec {
                name: "A Fill".into(),
            }),
        )?;
        let color_a = tx.create(
            Parent::Object(fill_a),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "A Color".into(),
                color: 0xff112233,
            }),
        )?;

        let artboard_b = tx.create_artboard(ArtboardSpec {
            name: "B".into(),
            width: 100.0,
            height: 100.0,
        })?;
        let shape_b = tx.create(
            Parent::Artboard(artboard_b),
            NodeSpec::Shape(ShapeSpec {
                name: "B Card".into(),
                x: 50.0,
                y: 50.0,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape_b),
            NodeSpec::Rectangle(RectangleSpec::new("B Rectangle", 80.0, 60.0)),
        )?;
        let fill_b = tx.create(
            Parent::Object(shape_b),
            NodeSpec::Fill(FillSpec {
                name: "B Fill".into(),
            }),
        )?;
        let color_b = tx.create(
            Parent::Object(fill_b),
            NodeSpec::SolidColor(SolidColorSpec {
                name: "B Color".into(),
                color: 0xff112233,
            }),
        )?;
        Ok((artboard_a, rectangle_a, color_a, artboard_b, color_b))
    })?;

    let instance_a = scene.instantiate(artboard_a)?;
    let instance_b = scene.instantiate(artboard_b)?;
    let cursor_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(cursor_b, 0xff445566)?);

    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let b_before = factory_b.stream();
    assert!(b_before.contains("ff445566"), "the hot color must render");

    let a_before = draw_stream(&mut scene, instance_a)?;
    scene.edit(|tx| {
        tx.set(rectangle_a, props::PATH_WIDTH, 72.0)?;
        Ok(())
    })?;

    assert_eq!(scene.frame().set(cursor_a, 0xff778899), Err(StaleCursor));
    assert_eq!(scene.frame().set(cursor_b, 0xff778899), Err(StaleCursor));
    assert_ne!(draw_stream(&mut scene, instance_a)?, a_before);

    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(
        factory_b.stream(),
        b_before,
        "an unrelated artboard edit must preserve the live instance and its held cache"
    );
    Ok(())
}

#[test]
fn failed_multi_artboard_materialization_publishes_nothing_before_a_valid_commit() -> Result<()> {
    let mut scene = Scene::new();
    let ((artboard_a, rectangle_a, color_a, artboard_b, rectangle_b, color_b), _) =
        scene.edit(|tx| {
            let (artboard_a, rectangle_a, color_a) = create_card(tx, "A", 0xff112233)?;
            let (artboard_b, rectangle_b, color_b) = create_card(tx, "B", 0xff223344)?;
            Ok((
                artboard_a,
                rectangle_a,
                color_a,
                artboard_b,
                rectangle_b,
                color_b,
            ))
        })?;
    let instance_a = scene.instantiate(artboard_a)?;
    let instance_b = scene.instantiate(artboard_b)?;
    let cursor_a = scene.cursor(instance_a, color_a, props::COLOR_VALUE)?;
    let cursor_b = scene.cursor(instance_b, color_b, props::COLOR_VALUE)?;
    assert!(scene.frame().set(cursor_a, 0xff556677)?);
    assert!(scene.frame().set(cursor_b, 0xff667788)?);

    let mut factory_a = RecordingFactory::new();
    let mut cache_a = scene.new_render_cache(instance_a, &mut factory_a)?;
    let mut renderer_a = factory_a.make_renderer();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    let a_before = factory_a.stream();

    let mut factory_b = RecordingFactory::new();
    let mut cache_b = scene.new_render_cache(instance_b, &mut factory_b)?;
    let mut renderer_b = factory_b.make_renderer();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    let b_before = factory_b.stream();
    let epoch_before = scene.epoch();

    let failed = scene.edit(|tx| {
        tx.set(rectangle_a, props::PATH_WIDTH, 72.0)?;
        tx.set(rectangle_b, props::PATH_WIDTH, 68.0)?;
        tx.create_artboard(ArtboardSpec {
            name: "Invalid".into(),
            width: f32::NAN,
            height: 100.0,
        })?;
        Ok(())
    });
    let failed = failed.expect_err("the third candidate must reject the scope");
    assert_eq!(
        failed.diagnostic().operation_index,
        2,
        "commit failure must identify the operation that introduced the invalid candidate"
    );
    assert_eq!(scene.epoch(), epoch_before);
    assert!(scene.frame().set(cursor_a, 0xff556677).is_ok());
    assert!(scene.frame().set(cursor_b, 0xff667788).is_ok());

    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    assert_eq!(factory_a.stream(), a_before);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_eq!(factory_b.stream(), b_before);

    let (_, receipt) = scene.edit(|tx| {
        tx.set(color_a, props::COLOR_VALUE, 0xffaabbcc)?;
        tx.set(color_b, props::COLOR_VALUE, 0xffbbccdd)?;
        Ok(())
    })?;
    assert_eq!(receipt.epoch, scene.epoch());
    assert_ne!(scene.epoch(), epoch_before);
    assert_eq!(scene.frame().set(cursor_a, 0xff8899aa), Err(StaleCursor));
    assert_eq!(scene.frame().set(cursor_b, 0xff99aabb), Err(StaleCursor));

    factory_a.clear();
    scene
        .frame()
        .draw(instance_a, &mut factory_a, &mut renderer_a, &mut cache_a)?;
    assert_ne!(factory_a.stream(), a_before);
    factory_b.clear();
    scene
        .frame()
        .draw(instance_b, &mut factory_b, &mut renderer_b, &mut cache_b)?;
    assert_ne!(factory_b.stream(), b_before);

    // Rebuild an independent oracle with only the valid color edit. Exact stream equality proves
    // the failed width definitions never leaked into the next successful materialization.
    let mut expected = Scene::new();
    let ((expected_a, _, expected_color_a, expected_b, _, expected_color_b), _) =
        expected.edit(|tx| {
            let (artboard_a, rectangle_a, color_a) = create_card(tx, "A", 0xff112233)?;
            let (artboard_b, rectangle_b, color_b) = create_card(tx, "B", 0xff223344)?;
            Ok((
                artboard_a,
                rectangle_a,
                color_a,
                artboard_b,
                rectangle_b,
                color_b,
            ))
        })?;
    expected.edit(|tx| {
        tx.set(expected_color_a, props::COLOR_VALUE, 0xffaabbcc)?;
        tx.set(expected_color_b, props::COLOR_VALUE, 0xffbbccdd)?;
        Ok(())
    })?;
    let expected_instance_a = expected.instantiate(expected_a)?;
    let expected_instance_b = expected.instantiate(expected_b)?;
    assert_eq!(
        draw_stream(&mut scene, instance_a)?,
        draw_stream(&mut expected, expected_instance_a)?
    );
    assert_eq!(
        draw_stream(&mut scene, instance_b)?,
        draw_stream(&mut expected, expected_instance_b)?
    );
    Ok(())
}
