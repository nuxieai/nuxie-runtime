//! Hosted own-rotation case from runtime/semantic_artboard_test.cpp at 1b9b7381.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    artboard_component_list::ArtboardComponentList,
    generated::{core_registry::CoreRegistry, transform_component_base::TransformComponentBase},
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::path::PathBuf;

#[test]
fn a_hosted_artboards_own_rotation_updates_semantic_bounds() {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .expect("RIVE_RUNTIME_DIR must point to the pinned upstream checkout");
    let path = PathBuf::from(root).join("tests/unit_tests/assets/semantic/data_binding_lists.riv");
    let bytes = std::fs::read(&path).unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|f| f.artboard_default()).unwrap();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine");
    machine.with_instance_mut(|sm| sm.enable_semantics());
    if let Some(vmi) = file
        .with_file(|f| f.create_default_view_model_instance_for_artboard(artboard.core_handle()))
    {
        artboard.bind_view_model_instance(Some(vmi.clone()));
        machine.with_instance_mut(|sm| sm.bind_view_model_instance(vmi));
    }
    for _ in 0..10 {
        machine.advance_and_apply(0.1);
    }
    let manager = machine
        .with_instance(|sm| sm.semantic_manager())
        .expect("semantic manager");
    manager.with_semantic_manager_mut(|mgr| mgr.drain_diff());
    let lists = artboard.with_artboard(|a| {
        a.objects_typed::<ArtboardComponentList>()
            .iter()
            .collect::<Vec<_>>()
    });
    let mut hosted = Vec::new();
    for list in lists {
        list.with_downcast::<ArtboardComponentList, _>(|list| {
            for i in 0..list.artboard_count() {
                if let Some(instance) = list.artboard_instance(i as i32) {
                    hosted.push(instance);
                }
            }
        })
        .unwrap();
    }
    assert!(!hosted.is_empty());
    for instance in hosted {
        assert!(CoreRegistry::set_double_handle(
            &instance.core_handle(),
            TransformComponentBase::ROTATION_PROPERTY_KEY.into(),
            0.7
        ));
    }
    for _ in 0..3 {
        machine.advance_and_apply(0.1);
    }
    assert!(
        !manager
            .with_semantic_manager_mut(|mgr| mgr.drain_diff())
            .updated_geometry
            .is_empty()
    );
}
