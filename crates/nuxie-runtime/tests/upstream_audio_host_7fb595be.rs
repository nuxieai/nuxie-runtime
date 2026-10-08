//! The two host-engine regressions from upstream audio_test.cpp at 7fb595be.

use std::{path::PathBuf, sync::Arc};

use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::{
    artboard_component_list::ArtboardComponentList,
    audio::audio_engine::AudioEngine,
    nested_artboard::NestedArtboard,
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};

fn read_file(name: &str) -> RuntimeFileHandle {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR");
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(NullFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("native fixture import")
}

#[test]
fn list_items_use_their_hosts_audio_engine() {
    let engine = AudioEngine::make(2, 44_100).expect("audio engine");
    let file = read_file("component_list_1.riv");
    let artboard = file.with_file(|file| file.artboard_named("Main")).unwrap();
    artboard.with_artboard_mut(|artboard| artboard.set_audio_engine(Some(engine.clone())));
    let list = artboard
        .with_artboard(|artboard| artboard.find_handle::<ArtboardComponentList>("List"))
        .expect("List");
    assert_eq!(
        list.with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::artboard_count),
        Some(0)
    );
    let view_model = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view-model instance");
    artboard.bind_view_model_instance(Some(view_model));
    artboard.advance_default(0.0);
    let count = list
        .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::artboard_count)
        .unwrap();
    assert!(count > 0);
    for index in 0..count {
        let item = list
            .with_downcast::<ArtboardComponentList, _>(|list| list.artboard_instance(index as i32))
            .flatten()
            .expect("list item");
        let actual = item
            .with_artboard(|artboard| artboard.audio_engine())
            .unwrap();
        assert!(Arc::ptr_eq(&actual, &engine));
    }
}

#[test]
fn swapped_nested_artboards_use_their_hosts_audio_engine() {
    let engine = AudioEngine::make(2, 44_100).expect("audio engine");
    let file = read_file("data_binding_artboards_test.riv");
    let artboard = file.with_file(|file| file.artboard_default()).unwrap();
    artboard.with_artboard_mut(|artboard| artboard.set_audio_engine(Some(engine.clone())));
    let machine = artboard.state_machine_at(0).expect("state machine");
    let model_id = artboard.with_artboard(|artboard| artboard.base.view_model_id());
    let view_model = file
        .with_file(|file| {
            if model_id == u32::MAX {
                file.create_view_model_instance_for_artboard(artboard.core_handle())
            } else {
                file.create_view_model_instance_at(model_id as usize, 0)
            }
        })
        .expect("view-model instance");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model.clone()));
    machine.advance_and_apply(0.1);
    let nested = artboard.with_artboard(|artboard| artboard.find_all_handles::<NestedArtboard>());
    let before: Vec<_> = nested
        .iter()
        .map(|nested| {
            nested
                .with(|object| {
                    object
                        .as_nested_artboard()
                        .expect("nested host")
                        .artboard_instance_default()
                })
                .expect("live nested host")
                .map(|instance| instance.core_handle())
        })
        .collect();
    let property = view_model
        .with_downcast::<ViewModelInstance, _>(|view_model| view_model.property_value_named("ab"))
        .flatten()
        .expect("ab property");
    let asset = file.with_file(|file| file.bindable_artboard_named("ch1"));
    property
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| property.set_asset(asset))
        .expect("artboard property");
    machine.advance_and_apply(0.1);
    let mut swapped = false;
    for (index, nested) in nested.iter().enumerate() {
        let Some(instance) = nested
            .with(|object| {
                object
                    .as_nested_artboard()
                    .expect("nested host")
                    .artboard_instance_default()
            })
            .expect("live nested host")
        else {
            continue;
        };
        swapped |= Some(instance.core_handle()) != before[index];
        let actual = instance
            .with_artboard(|artboard| artboard.audio_engine())
            .unwrap();
        assert!(Arc::ptr_eq(&actual, &engine));
    }
    assert!(swapped);
}
