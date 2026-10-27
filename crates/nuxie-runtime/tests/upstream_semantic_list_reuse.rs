//! Upstream 6771fdad semantic_artboard_test.cpp: replacement rows remain semantic.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    semantic::{
        semantic_role::SemanticRole,
        semantic_snapshot::{SemanticsDiff, SemanticsDiffNode},
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_string::ViewModelInstanceString,
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

fn apply_diff(entries: &mut BTreeMap<u32, SemanticsDiffNode>, diff: SemanticsDiff) {
    for id in diff.removed {
        entries.remove(&id);
    }
    for node in diff.added {
        entries.insert(node.id, node);
    }
    for node in diff.updated_semantic {
        if let Some(entry) = entries.get_mut(&node.id) {
            entry.role = node.role;
            entry.label = node.label;
            entry.state_flags = node.state_flags;
            entry.trait_flags = node.trait_flags;
        }
    }
    for geometry in diff.updated_geometry {
        if let Some(entry) = entries.get_mut(&geometry.id) {
            entry.set_bounds(geometry.bounds());
        }
    }
}

#[test]
fn replacing_every_list_item_keeps_new_rows_in_the_semantic_tree() {
    use nuxie_runtime::source::math::random::RandomProvider;
    struct TestingRandomMode;
    impl Drop for TestingRandomMode {
        fn drop(&mut self) {
            RandomProvider::clear_testing_mode();
        }
    }
    // The upstream TESTING fixture's random formula uses the empty FIFO.
    RandomProvider::clear_randoms();
    let _random_mode = TestingRandomMode;
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    let bytes = std::fs::read(
        PathBuf::from(root).join("tests/unit_tests/assets/semantic/data_binding_lists.riv"),
    )
    .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    machine.with_instance_mut(|machine| machine.enable_semantics());
    let vmi = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    artboard.bind_view_model_instance(Some(vmi.clone()));
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(vmi.clone()));
    for _ in 0..10 {
        machine.advance_and_apply(0.1);
    }
    let manager = machine
        .with_instance(|machine| machine.semantic_manager())
        .unwrap();
    let mut entries = BTreeMap::new();
    apply_diff(
        &mut entries,
        manager.with_semantic_manager_mut(|manager| manager.drain_diff()),
    );
    let fandom: BTreeSet<&str> = [
        "War of the Stars",
        "Scufflestar Galactica",
        "Galaxy Hike",
        "Dino Planet",
    ]
    .into_iter()
    .collect();
    let labels = |entries: &BTreeMap<u32, SemanticsDiffNode>| -> BTreeSet<String> {
        entries
            .values()
            .filter(|entry| {
                entry.role == SemanticRole::Text as u32 && fandom.contains(entry.label.as_str())
            })
            .map(|entry| entry.label.clone())
            .collect()
    };
    assert_eq!(
        labels(&entries),
        fandom.iter().map(|label| label.to_string()).collect()
    );
    let items = vmi
        .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named("menu"))
        .flatten()
        .unwrap();
    let old_items = items
        .with_downcast::<ViewModelInstanceList, _>(|items| items.list_items().to_vec())
        .unwrap();
    assert_eq!(old_items.len(), 4);
    let item_vmi = old_items[0]
        .with_downcast::<ViewModelInstanceListItem, _>(|item| item.view_model_instance())
        .flatten()
        .unwrap();
    let item_view_model = item_vmi
        .with_downcast::<ViewModelInstance, _>(|vmi| vmi.get_view_model())
        .flatten()
        .unwrap();
    let replacements: BTreeSet<&str> = [
        "First replacement",
        "Second replacement",
        "Third replacement",
        "Fourth replacement",
    ]
    .into_iter()
    .collect();
    items
        .with_downcast_mut::<ViewModelInstanceList, _>(|items| items.remove_all_items())
        .unwrap();
    for label in &replacements {
        let replacement = file
            .with_file(|file| file.create_default_view_model_instance(item_view_model.clone()))
            .unwrap();
        let property = replacement
            .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named("label"))
            .flatten()
            .unwrap();
        property
            .with_downcast_mut::<ViewModelInstanceString, _>(|property| property.set_value(*label))
            .unwrap();
        let item = items
            .insert_sibling(ViewModelInstanceListItem::default())
            .unwrap();
        item.with_downcast_mut::<ViewModelInstanceListItem, _>(|item| {
            item.set_view_model_instance(Some(replacement))
        })
        .unwrap();
        items
            .with_downcast_mut::<ViewModelInstanceList, _>(|items| items.add_item(item))
            .unwrap();
    }
    for _ in 0..10 {
        machine.advance_and_apply(0.1);
    }
    apply_diff(
        &mut entries,
        manager.with_semantic_manager_mut(|manager| manager.drain_diff()),
    );
    assert!(labels(&entries).is_empty());
    for label in replacements {
        let entry = entries
            .values()
            .find(|entry| entry.label == label)
            .unwrap_or_else(|| panic!("missing replacement {label}"));
        assert_eq!(entry.role, SemanticRole::Text as u32);
        assert!(manager.with_semantic_manager(|manager| manager.node_by_id(entry.id).is_some()));
    }
}
