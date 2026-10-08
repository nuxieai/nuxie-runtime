//! The two additions to `tests/unit_tests/runtime/semantic_artboard_test.cpp`
//! at upstream `081f85a690f26a4e8cceead05bd9c3f86707eae2`.
//! Run with `--features tools`, which exposes upstream TESTING nodeCount.

#![cfg(feature = "tools")]

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::{
        nested_state_machine::NestedStateMachine,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    nested_artboard::NestedArtboard,
    semantic::semantic_manager::RuntimeSemanticManagerHandle,
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
    },
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
};

fn fixture() -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let path = std::env::var_os("RIVE_RUNTIME_DIR").map_or_else(
        || {
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../fixtures/sync/swappable_artboards_focus.riv")
        },
        |root| PathBuf::from(root).join("tests/unit_tests/assets/swappable_artboards_focus.riv"),
    );
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("swappable_artboards_focus imports");
    let artboard = file
        .with_file(|file| file.artboard_named("Main"))
        .expect("Main instance");
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine zero");
    (file, artboard, machine)
}

fn slot_host(artboard: &RuntimeArtboardInstanceHandle) -> CoreHandle {
    // The upstream loop selects the last data-bound host.
    artboard
        .with_artboard(|artboard| artboard.nested_artboards())
        .into_iter()
        .filter(|host| {
            host.with(|host| {
                host.as_nested_artboard()
                    .is_some_and(NestedArtboard::is_artboard_data_bound)
            })
            .unwrap_or(false)
        })
        .last()
        .expect("data-bound nested-artboard slot")
}

fn instance(host: &CoreHandle) -> RuntimeArtboardInstanceHandle {
    host.with(|host| {
        host.as_nested_artboard()
            .and_then(|host| host.artboard_instance_handle(0))
    })
    .flatten()
    .expect("mounted slot instance")
}

fn enable(machine: &RuntimeStateMachineInstanceHandle) -> RuntimeSemanticManagerHandle {
    machine.with_instance_mut(|machine| machine.enable_semantics());
    machine
        .with_instance(|machine| machine.semantic_manager())
        .expect("semantic manager")
}

fn bind(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
) -> CoreHandle {
    let vmi = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view-model instance");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    vmi
}

fn assert_manager(
    artboard: &RuntimeArtboardInstanceHandle,
    manager: &RuntimeSemanticManagerHandle,
) {
    let actual = artboard
        .with_artboard(|artboard| artboard.semantic_manager())
        .expect("nested semantic manager");
    assert!(
        actual.ptr_eq(manager),
        "replacement must use the same semantic manager"
    );
}

fn bind_slot(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
) -> (CoreHandle, CoreHandle) {
    let vmi = bind(file, artboard, machine);
    let property = vmi
        .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named("artboardProp"))
        .flatten()
        .expect("artboardProp");
    machine.advance_and_apply(0.016);
    (slot_host(artboard), property)
}

fn bound_machine(artboard: &RuntimeArtboardInstanceHandle, host: &CoreHandle) -> CoreHandle {
    let animations = host
        .with_downcast::<NestedArtboard, _>(|host| host.nested_animations().to_vec())
        .expect("nested host");
    let mut owned = animations
        .into_iter()
        .filter(|animation| {
            animation
                .with_downcast::<NestedStateMachine, _>(|_| ())
                .is_some()
                && !artboard.with_artboard(|artboard| {
                    artboard
                        .objects()
                        .iter()
                        .flatten()
                        .any(|object| object == animation)
                })
        })
        .collect::<Vec<_>>();
    // Authored nested animations belong to the parent object list. The
    // data-bound host separately owns the machine synthesized by the swap.
    assert_eq!(owned.len(), 1, "one synthetic bound state machine");
    owned.pop().unwrap()
}

fn set_slot_asset(
    file: &RuntimeFileHandle,
    property: &CoreHandle,
    machine: &RuntimeStateMachineInstanceHandle,
    name: Option<&str>,
) {
    let source = name.map(|name| {
        file.with_file(|file| file.bindable_artboard_named(name))
            .expect("bindable source")
    });
    property
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| property.set_asset(source))
        .expect("artboard property");
    machine.advance_and_apply(0.016);
}

#[test]
fn replacing_bound_nested_state_machine_retires_old_occurrence() {
    let (file, artboard, machine) = fixture();
    let authored = artboard.with_artboard(|artboard| {
        artboard
            .objects_typed::<NestedStateMachine>()
            .iter()
            .collect::<Vec<_>>()
    });
    let (slot, property) = bind_slot(&file, &artboard, &machine);
    let arena = slot.retain_arena().expect("live host arena");
    for name in ["Swappable2", "Swappable1", "Swappable2"] {
        let outgoing = bound_machine(&artboard, &slot);
        set_slot_asset(&file, &property, &machine, Some(name));
        assert!(arena.contains(&slot), "the host arena remains live");
        assert!(
            !outgoing.is_alive(),
            "the replaced owned machine must retire"
        );
        assert!(outgoing.with(|_| ()).is_none());
        let incoming = bound_machine(&artboard, &slot);
        assert_ne!(incoming, outgoing);
        assert!(arena.contains(&incoming));
        assert!(authored.iter().all(|handle| handle.is_alive()));
    }
}

#[test]
fn clearing_bound_nested_artboard_retires_owned_state_machine() {
    let (file, artboard, machine) = fixture();
    let (slot, property) = bind_slot(&file, &artboard, &machine);
    let outgoing = bound_machine(&artboard, &slot);
    let arena = slot.retain_arena().expect("live host arena");
    set_slot_asset(&file, &property, &machine, None);
    assert!(arena.contains(&slot), "the host arena remains live");
    assert!(
        !outgoing.is_alive(),
        "explicit null must retire the owned machine"
    );
    assert!(outgoing.with(|_| ()).is_none());
    slot.with_downcast::<NestedArtboard, _>(|host| {
        assert!(host.artboard_instance_handle(0).is_none());
        assert!(host.nested_animations().is_empty());
    })
    .expect("live cleared host");
}

#[test]
fn unresolved_nested_artboard_preserves_owned_state_machine() {
    let (file, artboard, machine) = fixture();
    let (slot, property) = bind_slot(&file, &artboard, &machine);
    let existing = bound_machine(&artboard, &slot);
    let mounted = instance(&slot).core_handle();
    property
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
            property.set_property_value(9999);
            assert!(property.asset().is_none());
            assert_ne!(property.base.property_value(), u32::MAX);
        })
        .expect("unresolved artboard property");
    machine.advance_and_apply(0.016);
    assert!(existing.is_alive());
    assert_eq!(bound_machine(&artboard, &slot), existing);
    assert_eq!(instance(&slot).core_handle(), mounted);
}

#[test]
fn destroying_nested_host_retires_owned_machine_with_arena_retained() {
    let (file, artboard, machine) = fixture();
    let (slot, _property) = bind_slot(&file, &artboard, &machine);
    let bound = bound_machine(&artboard, &slot);
    let arena = slot.retain_arena().expect("live host arena");
    assert!(arena.contains(&bound));
    // Retain only the allocation domain, not the parent or mounted runtime
    // instance, so arena destruction cannot hide missing owned-child cleanup.
    drop(machine);
    drop(artboard);
    assert!(!slot.is_alive(), "parent teardown must destroy the host");
    assert!(
        !arena.contains(&bound),
        "host destruction must retire its machine"
    );
    assert!(bound.with(|_| ()).is_none());
}

#[test]
fn semantics_enabled_before_first_advance_survives_initial_data_bind_swap() {
    let (file, artboard, machine) = fixture();
    let slot = slot_host(&artboard);
    let manager = enable(&machine);
    {
        let built_against = instance(&slot);
        assert_eq!(
            built_against.with_artboard(|artboard| artboard.volume()),
            1.0
        );
        built_against.with_artboard_mut(|artboard| artboard.set_volume(0.25));
        // Do not retain this instance across binding: upstream frees it and
        // the following assertions must inspect the newly mounted instance.
    }
    let _vmi = bind(&file, &artboard, &machine);
    machine.advance_and_apply(0.016);
    {
        let after_bind = instance(&slot);
        assert_eq!(after_bind.with_artboard(|artboard| artboard.volume()), 1.0);
        assert_manager(&after_bind, &manager);
    }
    manager.with_semantic_manager_mut(|manager| manager.drain_diff());
    let nodes_after_bind = manager.with_semantic_manager(|manager| manager.node_count());
    for _ in 0..10 {
        machine.advance_and_apply(0.016);
        manager.with_semantic_manager_mut(|manager| manager.drain_diff());
    }
    assert_eq!(
        manager.with_semantic_manager(|manager| manager.node_count()),
        nodes_after_bind
    );
    assert_manager(&instance(&slot), &manager);
}

#[test]
fn swapping_data_bound_nested_artboard_rehomes_semantic_subtree() {
    let (file, artboard, machine) = fixture();
    let manager = enable(&machine);
    let vmi = bind(&file, &artboard, &machine);
    let slot = slot_host(&artboard);
    let property = vmi
        .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named("artboardProp"))
        .flatten()
        .expect("artboardProp");
    assert!(
        property
            .with_downcast::<ViewModelInstanceArtboard, _>(|_| ())
            .is_some()
    );
    let swap = |name: &str| {
        let source = file
            .with_file(|file| file.bindable_artboard_named(name))
            .expect("bindable source");
        let outgoing_name = instance(&slot).with_artboard(|artboard| artboard.name().to_owned());
        property
            .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
                property.set_asset(Some(source))
            })
            .expect("artboard property");
        machine.advance_and_apply(0.016);
        manager.with_semantic_manager_mut(|manager| manager.drain_diff());
        let incoming = instance(&slot);
        assert_ne!(
            incoming.with_artboard(|artboard| artboard.name().to_owned()),
            outgoing_name,
            "swap must actually instantiate a different source artboard"
        );
        assert_manager(&incoming, &manager);
    };
    swap("Swappable2");
    let nodes_after_first_swap = manager.with_semantic_manager(|manager| manager.node_count());
    swap("Swappable1");
    swap("Swappable2");
    assert_eq!(
        manager.with_semantic_manager(|manager| manager.node_count()),
        nodes_after_first_swap
    );
}

#[test]
fn swapped_in_nested_subtree_builds_semantics_outside_outer_host_borrow() {
    use nuxie_runtime::Artboard;
    use nuxie_runtime::source::semantic::semantic_data::SemanticData;
    use std::rc::Rc;

    let (file, artboard, machine) = fixture();
    // Extend the real fixture's Swappable2 source with one ordinary nested
    // host of its real Swappable1 source. Artboard::add_object is upstream's
    // TESTING seam; the eventual swap still clones and initializes this source
    // through the production artboard lifecycle (no manually built semantics).
    let source_one = file
        .with_file(|file| file.artboard_named_source("Swappable1"))
        .expect("Swappable1 source");
    let source_two = file
        .with_file(|file| file.artboard_named_source("Swappable2"))
        .expect("Swappable2 source");
    let child = file.with_file(|file| file.core_arena().insert(NestedArtboard::new()));
    child
        .with_downcast_mut::<NestedArtboard, _>(|child| child.referenced_artboard(Some(source_one)))
        .expect("ordinary nested source host");
    source_two
        .with_downcast_mut::<Artboard, _>(|source| source.add_object(Some(child)))
        .expect("source artboard object list");

    let manager = enable(&machine);
    let vmi = bind(&file, &artboard, &machine);
    let slot = slot_host(&artboard);
    let property = vmi
        .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named("artboardProp"))
        .flatten()
        .expect("artboardProp");
    let source = file
        .with_file(|file| file.bindable_artboard_named("Swappable2"))
        .expect("bindable Swappable2");
    property
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
            property.set_asset(Some(source))
        })
        .expect("artboard property");
    machine.advance_and_apply(0.016);
    manager.with_semantic_manager_mut(|manager| manager.drain_diff());

    let incoming = instance(&slot);
    assert_eq!(
        incoming.with_artboard(|artboard| artboard.name().to_owned()),
        "Swappable2"
    );
    let children = incoming.with_artboard(|artboard| artboard.nested_artboards());
    assert_eq!(children.len(), 1);
    let child_host = &children[0];
    assert!(
        SemanticData::find_closest_semantic_node_handle(Some(child_host.clone())).is_none(),
        "fixture must cross the incoming root and outer host without a nearer SemanticData"
    );
    let grandchild = instance(child_host);
    assert_manager(&incoming, &manager);
    assert_manager(&grandchild, &manager);
    let incoming_boundary = incoming
        .with_artboard(|artboard| artboard.semantic_boundary_node())
        .expect("incoming boundary");
    let child_boundary = grandchild
        .with_artboard(|artboard| artboard.semantic_boundary_node())
        .expect("grandchild boundary");
    let parent = child_boundary
        .borrow()
        .parent()
        .expect("grandchild semantic parent");
    assert!(
        Rc::ptr_eq(&parent, &incoming_boundary),
        "grandchild boundary belongs under incoming boundary"
    );
    assert!(
        incoming_boundary
            .borrow()
            .children()
            .iter()
            .any(|node| Rc::ptr_eq(node, &child_boundary))
    );
}

#[test]
fn root_frame_origin_preserves_native_bounds_but_moves_host_geometry() {
    use nuxie_runtime::source::{
        generated::{
            artboard_base::ArtboardBase, core_registry::CoreRegistry,
            layout_component_base::LayoutComponentBase,
        },
        semantic::semantic_data::SemanticData,
    };
    let (file, artboard, machine) = fixture();
    // Add authored semantics before the real data-bound clone is instantiated.
    for name in ["Swappable1", "Swappable2"] {
        let source = file
            .with_file(|file| file.artboard_named_source(name))
            .unwrap();
        let data = file.with_file(|file| file.core_arena().insert(SemanticData::default()));
        data.with_downcast_mut::<SemanticData, _>(|data| {
            data.set_label("Hosted control".to_owned());
            data.set_role(1);
        });
        source.with_downcast_mut::<nuxie_runtime::Artboard, _>(|source| {
            source.add_object(Some(data))
        });
    }
    let manager = enable(&machine);
    let _vmi = bind(&file, &artboard, &machine);
    machine.advance_and_apply(0.0);
    let nested = instance(&slot_host(&artboard));
    let ids = nested.with_artboard(|artboard| {
        artboard
            .objects_typed::<SemanticData>()
            .iter()
            .filter_map(|data| data.with_downcast::<SemanticData, _>(SemanticData::semantic_id))
            .collect::<Vec<_>>()
    });
    assert!(!ids.is_empty());
    let root = artboard.core_handle();
    assert!(CoreRegistry::set_bool_handle(
        &root,
        LayoutComponentBase::CLIP_PROPERTY_KEY.into(),
        false
    ));
    assert!(CoreRegistry::set_double_handle(
        &root,
        ArtboardBase::ORIGIN_X_PROPERTY_KEY.into(),
        0.25
    ));
    let width = artboard.with_artboard(|artboard| artboard.layout_width());
    let mut host_points = Vec::new();
    let mut snapshots = Vec::new();
    for frame_origin in [false, true] {
        artboard.with_artboard_mut(|artboard| artboard.set_frame_origin(frame_origin));
        artboard.advance_default(0.0);
        host_points.push(
            nuxie_runtime::source::semantic::semantic_provider::root_transform_point(
                &root,
                nuxie_runtime::source::math::vec2d::Vec2D::new(0.0, 0.0),
            )
            .unwrap(),
        );
        snapshots.push(manager.with_semantic_manager_mut(|manager| manager.snapshot().to_vec()));
    }
    let before = snapshots[0]
        .iter()
        .find(|node| ids.contains(&node.id))
        .expect("visible hosted control");
    let after = snapshots[1]
        .iter()
        .find(|node| node.id == before.id)
        .expect("same hosted control");
    assert_eq!(
        after.bounds(),
        before.bounds(),
        "upstream rootTransform excludes root frame origin"
    );
    assert!((host_points[1].x - host_points[0].x - width * 0.25).abs() < 0.01);
}
