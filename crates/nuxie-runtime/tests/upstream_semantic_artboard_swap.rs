//! The two additions to `tests/unit_tests/runtime/semantic_artboard_test.cpp`
//! at upstream `081f85a690f26a4e8cceead05bd9c3f86707eae2`.
//! Run with `--features tools`, which exposes upstream TESTING nodeCount.

#![cfg(feature = "tools")]

use std::path::PathBuf;

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
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
