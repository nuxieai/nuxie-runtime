//! Regression for the pinned File::read custom-enum link and inherited values.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::viewmodel::{
    viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
    viewmodel_instance_enum::ViewModelInstanceEnum,
    viewmodel_instance_number::ViewModelInstanceNumber,
};
use nuxie_runtime::{File, RuntimeFactoryHandle};

#[test]
fn imported_custom_display_enum_retains_show_hide_values() {
    // serialized_rendering_test.cpp's "Collapsed data bound layout styles
    // still update" uses display_2.value(1) / value(0) on this exact fixture.
    let fixture = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/collapse_data_binds.riv");
    let bytes = std::fs::read(fixture).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let model = file.with_file(|file| {
        (0..file.view_model_count())
            .filter_map(|index| file.view_model(index))
            .find(|model| {
                model
                    .with_downcast::<ViewModel, _>(|model| {
                        model.property_named("display_2").is_some()
                    })
                    .unwrap()
            })
            .expect("fixture model declaring display_2")
    });
    let instance = ViewModel::create_instance_handle(&model).unwrap();
    let value = instance
        .with_downcast::<ViewModelInstance, _>(|instance| {
            instance.property_value_named("display_2")
        })
        .flatten()
        .unwrap();
    value
        .with_downcast_mut::<ViewModelInstanceEnum, _>(|value| {
            assert_eq!(value.enum_type(), "Layout Display");
            assert_eq!(value.values(), ["Show", "Hide"]);
            assert!(value.set_value_at(1));
            assert_eq!(value.value(), "Hide");
            assert!(value.set_value_named("Show"));
            assert_eq!(value.base.property_value(), 0);
            assert!(!value.set_value_at(2));
            assert_eq!(value.value(), "Show");
        })
        .unwrap();
}

#[test]
fn imported_collapse_layout_instance_retains_all_pinned_number_values() {
    // The pinned C++ probe reports -1 for pos_3, pos_2, pos_1, and pos_0 in
    // vm instance 0. The scripted renderer clones that exact instance.
    let fixture = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/collapse_data_binds.riv");
    let bytes = std::fs::read(fixture).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let instance = file
        .with_file(|file| file.create_view_model_instance_at(1, 0))
        .unwrap();

    for name in ["pos_3", "pos_2", "pos_1", "pos_0"] {
        let value = instance
            .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
            .flatten()
            .unwrap();
        assert_eq!(
            value
                .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
                .unwrap(),
            -1.0,
            "{name}"
        );
    }

    let artboard = file
        .with_file(|file| file.artboard_named("test-1"))
        .unwrap();
    artboard.bind_view_model_instance(Some(instance.clone()));
    let machine = artboard.state_machine_at(0).unwrap();
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(instance));
    machine.advance_and_apply(0.0);
    let binds = artboard.with_artboard(|artboard| artboard.data_bind_handles());
    assert_eq!(binds.len(), 8);
    let text_binds: Vec<_> = binds
        .iter()
        .filter(|bind| {
            bind.with(|bind| bind.as_data_bind().unwrap().base.property_key() == 268)
                .unwrap()
        })
        .collect();
    assert_eq!(text_binds.len(), 4);
    for (bind, (path, expected)) in
        text_binds
            .into_iter()
            .zip([([1, 3], 0.0), ([1, 2], 0.0), ([1, 1], 0.0), ([1, 0], 250.0)])
    {
        assert_eq!(
            bind.with_downcast::<
                nuxie_runtime::source::data_bind::data_bind_context::DataBindContext,
                _,
            >(|bind| bind.source_path_ids().to_vec())
            .unwrap(),
            path
        );
        let source = bind
            .with(|bind| bind.as_data_bind().unwrap().source())
            .flatten()
            .unwrap();
        assert_eq!(
            source
                .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
                .unwrap(),
            expected,
            "{path:?}"
        );
        let target = bind
            .with(|bind| bind.as_data_bind().unwrap().target())
            .flatten()
            .unwrap();
        assert_eq!(
            target
                .with_downcast::<nuxie_runtime::source::text::text_value_run::TextValueRun, _>(
                    |target| target.base.text().to_owned(),
                )
                .unwrap(),
            if expected == 250.0 { "250" } else { "0" },
            "{path:?}"
        );
    }
}
