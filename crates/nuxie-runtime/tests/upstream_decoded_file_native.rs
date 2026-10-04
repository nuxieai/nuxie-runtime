//! Native mounting half of 6cd5d108 scripting_decode_file_test.cpp.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    nested_artboard::NestedArtboard,
    scripted::decoded_file::{decoded_bindable, import_decoded_file},
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

fn bytes(name: &str) -> Vec<u8> {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    std::fs::read(root.join("tests/unit_tests/assets").join(name)).unwrap()
}

// Rust integration regression: upstream clears a raw File pointer at teardown;
// our weak edge must become unavailable without mutably reborrowing the model.
#[test]
#[cfg(feature = "tools")]
fn last_file_can_drop_inside_a_live_model_read_borrow() {
    use nuxie_runtime::source::viewmodel::viewmodel::ViewModel;

    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = File::import(
        &bytes("viewmodel_self_reference.riv"),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    // CoreHandle is weak: retain its arena, not the File, so the definition
    // stays alive after the last owning File handle is released.
    let (model, _arena) =
        file.with_file(|file| (file.view_model(0).unwrap(), file.core_arena().clone()));
    model
        .with_downcast::<ViewModel, _>(move |model| {
            assert!(model.file().upgrade().is_some());
            drop(file);
            assert!(model.file().upgrade().is_none());
        })
        .unwrap();
    assert!(ViewModel::create_instance_handle(&model).is_none());
}

#[test]
fn stateful_host_keeps_decoded_artboards_own_view_model() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(
        &bytes("stateful_bindable_host.riv"),
        factory.clone(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(File::artboard_default).unwrap();
    let machine = artboard.state_machine_at(0).unwrap();
    let model = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .unwrap();
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(model.clone()));
    machine.advance_and_apply(0.016);
    let slot = artboard
        .with_artboard(|artboard| artboard.find_all_handles::<NestedArtboard>())
        .pop()
        .unwrap();
    assert!(slot
        .with_downcast::<NestedArtboard, _>(|slot| slot.base.is_stateful())
        .unwrap());
    let stateful = slot
        .with_downcast::<NestedArtboard, _>(|slot| {
            slot.base
                .base
                .base
                .children()
                .iter()
                .find(|child| {
                    child
                        .with_downcast::<ViewModelInstance, _>(|_| ())
                        .is_some()
                })
                .cloned()
        })
        .flatten()
        .unwrap();
    let mounted = || {
        slot.with_downcast::<NestedArtboard, _>(NestedArtboard::artboard_instance_default)
            .flatten()
            .unwrap()
    };
    let bound = || {
        mounted()
            .data_context()
            .and_then(|context| context.with_context(|context| context.main_view_model_instance()))
    };
    assert_eq!(
        mounted().with_artboard(|artboard| artboard.name().to_owned()),
        "Own"
    );
    assert_eq!(bound(), Some(stateful.clone()));
    let child =
        import_decoded_file(&bytes("bindable_artboard_child.riv"), factory, None, None).unwrap();
    let decoded = child.with_file(|file| decoded_bindable(file, Some("Artboard")));
    let expected = decoded.view_model.clone().unwrap();
    let property = model
        .with_downcast::<ViewModelInstance, _>(|model| model.property_value_named("someArtboard"))
        .flatten()
        .unwrap();
    property.with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
        property.set_bound_view_model_instance(decoded.view_model);
        property.set_asset(decoded.artboard);
    });
    for _ in 0..3 {
        machine.advance_and_apply(0.016);
    }
    assert_eq!(
        mounted().with_artboard(|artboard| artboard.name().to_owned()),
        "Artboard"
    );
    assert_eq!(bound(), Some(expected));
    property.with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| {
        property.set_property_value(1)
    });
    for _ in 0..3 {
        machine.advance_and_apply(0.016);
    }
    assert_eq!(
        mounted().with_artboard(|artboard| artboard.name().to_owned()),
        "Own"
    );
    assert_eq!(bound(), Some(stateful));
}
