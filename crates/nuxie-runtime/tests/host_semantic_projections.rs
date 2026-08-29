use std::path::{Path, PathBuf};

use nuxie_render_api::{PersistentFactory, RecordingFactory, Vec2D};
use nuxie_runtime::{
    ArtboardInstance, File, RuntimeArtboardOccurrenceSegment, RuntimeFactoryHandle,
    RuntimeLayoutBounds, RuntimeScrollConstraintSnapshot, StateMachineEventContext,
    StateMachineInputKind,
    source::animation::nested_state_machine::NestedStateMachine,
    source::artboard::Artboard,
    source::artboard_component_list::ArtboardComponentList,
    source::assets::image_asset::ImageAsset,
    source::generated::{
        core_registry::CoreRegistry, layout_component_base::LayoutComponentBase,
        shapes::paint::solid_color_base::SolidColorBase,
    },
    source::nested_artboard::NestedArtboard,
    source::viewmodel::viewmodel_instance::ViewModelInstance,
    source::viewmodel::viewmodel_instance_boolean::ViewModelInstanceBoolean,
};

fn fixture_path(name: &str) -> PathBuf {
    let runtime_dir = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    Path::new(&runtime_dir)
        .join("tests/unit_tests/assets")
        .join(name)
}

fn import_host_artboard(fixture: &str) -> (PersistentFactory<RecordingFactory>, ArtboardInstance) {
    let bytes = std::fs::read(fixture_path(fixture)).expect("pinned fixture bytes");
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("fixture imports");
    let artboard = ArtboardInstance::from_native(file, 0).expect("default artboard instance");
    (factory, artboard)
}

fn occurrence_identity(handle: &nuxie_runtime::CoreHandle) -> u64 {
    let (arena, slot, generation) = handle.identity_key();
    let mut value = 0xcbf2_9ce4_8422_2325u64;
    for part in [arena as u64, slot as u64, generation] {
        value ^= part;
        value = value.wrapping_mul(0x100_0000_01b3);
    }
    value
}

#[test]
fn scroll_snapshots_are_exact_occurrence_observations() {
    let (_factory, mut artboard) = import_host_artboard("layout/layout_scroll_vertical.riv");
    artboard.advance(0.0).expect("initial advance");

    let snapshots: Vec<RuntimeScrollConstraintSnapshot> = artboard.scroll_constraint_occurrences();
    assert_eq!(snapshots.len(), 1);
    let snapshot = snapshots[0];
    assert_eq!(
        artboard.scroll_constraint_for_content(snapshot.content_local_id),
        Some(snapshot)
    );
    assert_eq!(
        artboard.scroll_constraint_for_authored_id(snapshot.constraint_authored_id),
        Some(snapshot)
    );
    assert_eq!(
        artboard.scroll_constraint_for_content_authored_id(snapshot.content_authored_id),
        Some(snapshot)
    );
    assert_eq!(snapshot.offset, snapshot.clamped_offset);
    assert!(!snapshot.physics_running);
    assert!(!snapshot.scroll_active);
}

#[test]
fn layout_and_geometry_reads_share_the_settled_native_occurrence() {
    let (_factory, mut artboard) = import_host_artboard("layout/fixed_participant.riv");
    artboard.advance(0.0).expect("initial advance");

    let local_id = (0..artboard.object_count())
        .find(|&local_id| artboard.layout_bounds(local_id).is_some())
        .expect("fixture has a layout occurrence");
    let bounds: RuntimeLayoutBounds = artboard.layout_bounds(local_id).unwrap();
    assert!(bounds.width.is_finite() && bounds.height.is_finite());
    assert!(artboard.world_transform(local_id).is_some());
    assert!(artboard.world_transform_with_scroll(local_id).is_some());
    assert!(artboard.world_bounds(local_id).is_some());
    assert_eq!(
        artboard.scrolled_layout_bounds(local_id).unwrap().width,
        bounds.width
    );

    assert_eq!(artboard.try_semantic_geometry_revision(), None);

    let visible = artboard.visible_geometry_with_bounds();
    assert!(!visible.is_empty());
    let point = visible[0].bounds.center();
    let hits = artboard.hit_test_segments_with_bounds(Vec2D::new(point.x, point.y));
    assert!(!hits.is_empty());
    let context = StateMachineEventContext::from_geometry_hit(&hits[0]);
    assert_eq!(context.path(), hits[0].path);
    assert_eq!(context.occurrence(), hits[0].occurrence);
    assert!(
        artboard
            .hit_test_segments_with_bounds(Vec2D::new(1.0e20, 1.0e20))
            .is_empty()
    );
}

#[test]
fn static_text_queries_use_the_settled_text_occurrence() {
    let (_factory, mut artboard) = import_host_artboard("hello_world.riv");
    artboard.advance(0.0).expect("initial advance");

    let semantic = artboard.semantic_text_with_bounds();
    let text = semantic
        .iter()
        .find(|text| !text.value.is_empty() && text.path.len() == 1)
        .expect("fixture has root semantic Text");
    let local_id = text.path[0].local_id;
    let first_boundary = text.value.char_indices().nth(1).unwrap().0;
    let caret = artboard
        .text_caret(local_id, first_boundary)
        .expect("UTF-8 boundary has shaped caret");
    let midpoint = Vec2D::new(
        (caret.top.x + caret.bottom.x) * 0.5,
        (caret.top.y + caret.bottom.y) * 0.5,
    );
    assert_eq!(artboard.text_hit(local_id, midpoint), Some(first_boundary));
    assert!(
        !artboard
            .text_selection_rects(local_id, 0..first_boundary)
            .is_empty()
    );
}

#[test]
fn image_dimensions_mutate_the_file_owned_image_asset() {
    let (_factory, mut artboard) = import_host_artboard("hosted_image_file.riv");
    let file = artboard.native_file();
    let (asset_global_id, previous) = file.with_file(|file| {
        file.assets()
            .iter()
            .find_map(|asset| {
                asset.with_downcast::<ImageAsset, _>(|image| {
                    Some((
                        u32::try_from(asset.identity_key().1).ok()?,
                        (image.base.width(), image.base.height()),
                    ))
                })?
            })
            .expect("fixture has ImageAsset")
    });
    let dimensions = if previous.0 > 0.0 && previous.1 > 0.0 {
        (previous.0 as u32, previous.1 as u32)
    } else {
        (37, 41)
    };

    artboard
        .register_image_dimensions(asset_global_id, dimensions.0, dimensions.1)
        .expect("first canonical registration succeeds");
    let retained = file.with_file(|file| {
        file.assets()
            .iter()
            .find(|asset| u32::try_from(asset.identity_key().1) == Ok(asset_global_id))
            .and_then(|asset| {
                asset.with_downcast::<ImageAsset, _>(|image| {
                    (image.base.width(), image.base.height())
                })
            })
    });
    assert_eq!(retained, Some((dimensions.0 as f32, dimensions.1 as f32)));
    assert!(
        artboard
            .register_image_dimensions(
                asset_global_id,
                dimensions.0.saturating_add(1),
                dimensions.1,
            )
            .is_err()
    );
}

#[test]
fn nested_occurrence_input_writes_target_the_retained_machine() {
    let (_factory, mut artboard) = import_host_artboard("runtime_nested_inputs.riv");
    artboard.advance(0.0).expect("initial advance");

    let native = artboard.native_handle();
    let (host_local_id, machine_index, input_index, input_name, initial, machine) = native
        .with_artboard(|artboard| {
            artboard
                .base
                .objects()
                .iter()
                .enumerate()
                .find_map(|(host_local_id, object)| {
                    object
                        .as_ref()?
                        .with_downcast::<NestedArtboard, _>(|nested| {
                            nested.nested_animations().iter().find_map(|animation| {
                                animation
                                    .with_downcast::<NestedStateMachine, _>(|nested_machine| {
                                        let machine = nested_machine.state_machine_instance()?;
                                        machine.with_instance(|machine_instance| {
                                            (0..machine_instance.input_count()).find_map(
                                                |input_index| {
                                                    let input = machine_instance.bool_input(
                                                        u32::try_from(input_index).ok()?,
                                                    )?;
                                                    Some((
                                                        host_local_id,
                                                        nested_machine.base.animation_id() as usize,
                                                        input_index,
                                                        input.base.name().to_owned(),
                                                        input.value(),
                                                        machine.clone(),
                                                    ))
                                                },
                                            )
                                        })
                                    })
                                    .flatten()
                            })
                        })?
                })
        })
        .expect("fixture has a nested bool input occurrence");
    let occurrence = [RuntimeArtboardOccurrenceSegment::NestedArtboard { host_local_id }];

    assert_eq!(
        artboard.occurrence_state_machine_input(&occurrence, machine_index, &input_name),
        Some((input_index, StateMachineInputKind::Bool))
    );
    assert_eq!(
        artboard.set_occurrence_state_machine_bool(
            &occurrence,
            machine_index,
            input_index,
            !initial,
        ),
        Some(true)
    );
    assert_eq!(
        machine.with_instance(|machine| {
            machine
                .bool_input(u32::try_from(input_index).unwrap())
                .unwrap()
                .value()
        }),
        !initial
    );
    assert_eq!(
        artboard.occurrence_state_machine_input(
            &[RuntimeArtboardOccurrenceSegment::NestedArtboard {
                host_local_id: usize::MAX,
            }],
            machine_index,
            &input_name,
        ),
        None
    );
}

#[test]
fn named_nested_artboard_projection_weakly_fences_and_mutates_the_exact_occurrence() {
    let (_factory, mut artboard) = import_host_artboard("runtime_nested_inputs.riv");
    let second = ArtboardInstance::from_native(artboard.native_file(), 0)
        .expect("second root occurrence instantiates");
    assert_ne!(artboard.instance_identity(), second.instance_identity());

    artboard.advance(0.0).expect("initial advance");
    let (nested_host, child) = artboard
        .native_handle()
        .with_artboard(|artboard| {
            artboard
                .base
                .nested_artboards()
                .into_iter()
                .find_map(|host| {
                    let child = host
                        .with_downcast::<NestedArtboard, _>(
                            NestedArtboard::artboard_instance_default,
                        )
                        .flatten()?;
                    Some((host, child))
                })
        })
        .expect("fixture has a retained nested occurrence");
    let source_name = child
        .with_artboard(|child| child.base.artboard_source_handle())
        .and_then(|source| {
            source.with_downcast::<Artboard, _>(|source| source.base.name().to_owned())
        })
        .expect("nested occurrence has an exact source Artboard");
    let child_identity = occurrence_identity(&child.core_handle());
    let outgoing_child = child.downgrade();

    let mut occurrences = artboard.nested_artboard_occurrences_named(&source_name);
    let occurrence = occurrences
        .iter_mut()
        .find(|occurrence| occurrence.instance_identity() == child_identity)
        .expect("named projection fences the observed child");
    assert!(!occurrence.set_double_property(0, u16::MAX, 1.0));
    assert!(!occurrence.set_color_property(0, u16::MAX, 0xff00_00ff));

    let authored_width = CoreRegistry::get_double_handle(
        &child.core_handle(),
        i32::from(LayoutComponentBase::WIDTH_PROPERTY_KEY),
    )
    .expect("child root exposes the authored width property");
    assert!(occurrence.set_double_property(
        0,
        LayoutComponentBase::WIDTH_PROPERTY_KEY,
        authored_width + 0.25,
    ));
    assert!(!occurrence.set_double_property(
        0,
        LayoutComponentBase::WIDTH_PROPERTY_KEY,
        authored_width + 0.25,
    ));

    let (color_local_id, color_before) = child
        .with_artboard(|child| {
            child
                .base
                .objects()
                .iter()
                .enumerate()
                .find_map(|(local_id, object)| {
                    let object = object.as_ref()?;
                    object.is_type_of(SolidColorBase::TYPE_KEY).then(|| {
                        (
                            local_id,
                            CoreRegistry::get_color_handle(
                                object,
                                i32::from(SolidColorBase::COLOR_VALUE_PROPERTY_KEY),
                            )
                            .expect("SolidColor exposes its generated color property"),
                        )
                    })
                })
        })
        .expect("nested fixture has a SolidColor occurrence");
    let color_after = (color_before as u32) ^ 1;
    assert!(occurrence.set_color_property(
        color_local_id,
        SolidColorBase::COLOR_VALUE_PROPERTY_KEY,
        color_after,
    ));
    assert!(!occurrence.set_color_property(
        color_local_id,
        SolidColorBase::COLOR_VALUE_PROPERTY_KEY,
        color_after,
    ));

    let (width, height) = occurrence
        .artboard_dimensions()
        .expect("retained child remains mounted");
    assert!(occurrence.set_artboard_dimensions(width + 1.0, height + 2.0));
    assert!(!occurrence.set_artboard_dimensions(width + 1.0, height + 2.0));
    occurrence.update_components();
    assert_eq!(
        occurrence.artboard_dimensions(),
        Some((width + 1.0, height + 2.0))
    );

    let source = child
        .with_artboard(|child| child.base.artboard_source_handle())
        .expect("nested child has a source Artboard");
    let replacement = Artboard::nested_instance_from_handle(&source)
        .expect("source creates a replacement nested occurrence");
    nested_host.with_downcast_mut::<NestedArtboard, _>(|host| {
        host.referenced_artboard_instance(replacement)
    });
    drop(child);
    assert!(
        outgoing_child.upgrade().is_none(),
        "the host projection must not extend an outgoing child's lifetime"
    );
    assert!(!occurrence.is_current());
    assert_eq!(occurrence.artboard_dimensions(), None);
    assert!(!occurrence.set_artboard_dimensions(width, height));
}

#[test]
fn component_list_occurrence_identity_fences_input_writes() {
    let (_factory, mut artboard) = import_host_artboard("component_list_1.riv");
    let view_model = artboard
        .native_file()
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(
                artboard.native_handle().core_handle(),
            )
        })
        .expect("fixture has default view model");
    artboard.bind_native_view_model(Some(view_model));
    let mut root_machine = artboard
        .default_state_machine_instance()
        .expect("fixture has default state machine");
    root_machine.advance_and_apply(0.0);
    artboard.advance(0.0).expect("initial advance");

    let native = artboard.native_handle();
    let (host_local_id, item_index, item, machine_index, input_index, input_name, initial, machine) =
        native
            .with_artboard(|artboard| {
                artboard
                    .base
                    .objects()
                    .iter()
                    .enumerate()
                    .find_map(|(host_local_id, object)| {
                        object
                            .as_ref()?
                            .with_downcast::<ArtboardComponentList, _>(|list| {
                                (0..list.artboard_count()).find_map(|item_index| {
                                    let item_index_i32 = i32::try_from(item_index).ok()?;
                                    let item = list.list_item(item_index_i32)?;
                                    let child = list.artboard_instance(item_index_i32)?;
                                    let machine = list.state_machine_instance(item_index_i32)?;
                                    let machine_definition =
                                        machine.with_instance(|machine| machine.state_machine());
                                    let machine_index = (0..16).find(|&index| {
                                        child.with_artboard(|child| {
                                            child.state_machine_handle_at(index)
                                                == Some(machine_definition.clone())
                                        })
                                    })?;
                                    machine.with_instance(|machine_instance| {
                                        (0..machine_instance.input_count()).find_map(
                                            |input_index| {
                                                let input = machine_instance
                                                    .bool_input(u32::try_from(input_index).ok()?)?;
                                                Some((
                                                    host_local_id,
                                                    item_index,
                                                    item.clone(),
                                                    machine_index,
                                                    input_index,
                                                    input.base.name().to_owned(),
                                                    input.value(),
                                                    machine.clone(),
                                                ))
                                            },
                                        )
                                    })
                                })
                            })?
                    })
            })
            .expect("fixture has a component-list bool input occurrence");
    let occurrence_identity = occurrence_identity(&item);
    let occurrence = [RuntimeArtboardOccurrenceSegment::ComponentListItem {
        host_local_id,
        item_index,
        occurrence_identity,
    }];

    assert_eq!(
        artboard.occurrence_state_machine_input(&occurrence, machine_index, &input_name),
        Some((input_index, StateMachineInputKind::Bool))
    );
    assert_eq!(
        artboard.set_occurrence_state_machine_bool(
            &occurrence,
            machine_index,
            input_index,
            !initial,
        ),
        Some(true)
    );
    assert_eq!(
        machine.with_instance(|machine| {
            machine
                .bool_input(u32::try_from(input_index).unwrap())
                .unwrap()
                .value()
        }),
        !initial
    );
    let stale = [RuntimeArtboardOccurrenceSegment::ComponentListItem {
        host_local_id,
        item_index,
        occurrence_identity: occurrence_identity.wrapping_add(1),
    }];
    assert_eq!(
        artboard.set_occurrence_state_machine_bool(&stale, machine_index, input_index, initial,),
        None
    );
    assert_eq!(artboard.occurrence_view_model_boolean(&stale, &[0]), None);
}

#[test]
fn named_nested_projection_rejects_a_component_list_occurrence_recycled_for_another_item() {
    let (_factory, mut artboard) = import_host_artboard("component_list_virtualized.riv");
    let view_model = artboard
        .native_file()
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(
                artboard.native_handle().core_handle(),
            )
        })
        .expect("fixture has default view model");
    artboard.bind_native_view_model(Some(view_model));
    artboard.advance(0.0).expect("initial advance");

    let (host, first_index, second_index, first_child, source_name) = artboard
        .native_handle()
        .with_artboard(|artboard| {
            artboard.base.objects().iter().flatten().find_map(|host| {
                host.with_downcast::<ArtboardComponentList, _>(|list| {
                    (0..list.artboard_count()).find_map(|first_index| {
                        let first_child = list.artboard_instance(first_index as i32)?;
                        let source = first_child
                            .with_artboard(|child| child.base.artboard_source_handle())?;
                        let second_index =
                            (first_index + 1..list.artboard_count()).find(|&index| {
                                list.artboard_instance(index as i32).is_some_and(|child| {
                                    child
                                        .with_artboard(|child| child.base.artboard_source_handle())
                                        .as_ref()
                                        == Some(&source)
                                })
                            })?;
                        let source_name = source
                            .with_downcast::<Artboard, _>(|source| source.base.name().to_owned())?;
                        Some((
                            host.clone(),
                            first_index,
                            second_index,
                            first_child,
                            source_name,
                        ))
                    })
                })?
            })
        })
        .expect("fixture has two virtualized rows backed by the same source Artboard");
    let first_identity = occurrence_identity(&first_child.core_handle());
    let mut occurrences = artboard.nested_artboard_occurrences_named(&source_name);
    let occurrence = occurrences
        .iter_mut()
        .find(|occurrence| occurrence.instance_identity() == first_identity)
        .expect("projection fences the first list-item occurrence");

    // Pool row B first, then row A. Recreating B pops A's exact occurrence
    // from the source-artboard pool and rebinds it to B's list item.
    host.with_downcast_mut::<ArtboardComponentList, _>(|list| {
        list.remove_virtualizable(second_index as i32);
        list.remove_virtualizable(first_index as i32);
    });
    ArtboardComponentList::add_virtualizable_occurrence(&host, second_index as i32);
    let rebound_identity = host
        .with_downcast::<ArtboardComponentList, _>(|list| {
            list.artboard_instance(second_index as i32)
                .map(|child| occurrence_identity(&child.core_handle()))
        })
        .flatten()
        .expect("second row is rebound from the pool");
    assert_eq!(rebound_identity, first_identity, "fixture exercised reuse");
    assert!(
        !occurrence.is_current(),
        "the old row-A fence must reject the same Artboard root rebound to row B"
    );
    assert!(!occurrence.set_double_property(0, LayoutComponentBase::WIDTH_PROPERTY_KEY, 1.0,));
}

#[test]
fn component_list_view_model_reads_target_the_retained_item_context() {
    let (_factory, mut artboard) = import_host_artboard("component_list_virtualized.riv");
    let view_model = artboard
        .native_file()
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(
                artboard.native_handle().core_handle(),
            )
        })
        .expect("fixture has default view model");
    artboard.bind_native_view_model(Some(view_model));
    artboard.advance(0.0).expect("initial advance");

    let native = artboard.native_handle();
    let (host_local_id, item_index, item, source_path, retained_value) = native
        .with_artboard(|artboard| {
            artboard
                .base
                .objects()
                .iter()
                .enumerate()
                .find_map(|(host_local_id, object)| {
                    object
                        .as_ref()?
                        .with_downcast::<ArtboardComponentList, _>(|list| {
                            (0..list.artboard_count()).find_map(|item_index| {
                                let item_index_i32 = i32::try_from(item_index).ok()?;
                                let item = list.list_item(item_index_i32)?;
                                let child = list.artboard_instance(item_index_i32)?;
                                let (view_model_id, property_id, value) =
                                    child.with_artboard(|child| {
                                        let context = child.base.data_context()?;
                                        let instance = context.with_context(|context| {
                                            context.main_view_model_instance()
                                        })?;
                                        instance
                                            .with_downcast::<ViewModelInstance, _>(|instance| {
                                                instance.property_values().iter().find_map(
                                                    |property| {
                                                        property.with_downcast::<
                                                            ViewModelInstanceBoolean,
                                                            _,
                                                        >(|boolean| {
                                                            (
                                                                instance.base.view_model_id(),
                                                                boolean
                                                                    .base
                                                                    .base
                                                                    .view_model_property_id(),
                                                                boolean.value(),
                                                            )
                                                        })
                                                    },
                                                )
                                            })
                                            .flatten()
                                    })?;
                                Some((
                                    host_local_id,
                                    item_index,
                                    item,
                                    [view_model_id, property_id],
                                    value,
                                ))
                            })
                        })?
                })
        })
        .expect("fixture has a retained list item boolean context");
    let occurrence = [RuntimeArtboardOccurrenceSegment::ComponentListItem {
        host_local_id,
        item_index,
        occurrence_identity: occurrence_identity(&item),
    }];

    assert_eq!(
        artboard.occurrence_view_model_boolean(&occurrence, &source_path),
        Some(retained_value)
    );
    let stale = [RuntimeArtboardOccurrenceSegment::ComponentListItem {
        host_local_id,
        item_index,
        occurrence_identity: occurrence_identity(&item).wrapping_add(1),
    }];
    assert_eq!(
        artboard.occurrence_view_model_boolean(&stale, &source_path),
        None
    );
}

#[test]
fn malformed_embedded_fonts_fail_closed_without_panicking() {
    assert!(!nuxie_runtime::embedded_font_is_parseable(b"not a font"));
    assert!(
        std::panic::catch_unwind(|| { nuxie_runtime::embedded_font_is_parseable(&[0xff; 128]) })
            .is_ok()
    );
}
