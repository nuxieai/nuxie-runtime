use nuxie_runtime::source::{
    core::{CoreObject, binary_reader::BinaryReader},
    data_bind::{data_bind_context::DataBindContext, data_bind_path::DataBindPath},
    data_bind_path_referencer::DataBindPathReferencer,
    generated::data_bind::{
        data_bind_context_base::DataBindContextBase, data_bind_path_base::DataBindPathBase,
    },
};

#[test]
fn source_path_decode_and_clone_keep_the_callback_receivers_data() {
    let mut binding = DataBindContext::default();
    let mut reader = BinaryReader::new(&[3, 0, 0x81, 1]);
    assert!(CoreObject::deserialize(
        &mut binding,
        DataBindContextBase::SOURCE_PATH_IDS_PROPERTY_KEY,
        &mut reader,
    ));
    assert!(reader.reached_end());
    assert!(!reader.has_error());
    assert_eq!(binding.source_path_ids(), &[0, 129]);
    binding.base.is_path_resolved = true;

    let cloned = CoreObject::clone_boxed(&binding).expect("cloned DataBindContext");
    let cloned = cloned.as_any().downcast_ref::<DataBindContext>().unwrap();
    assert_eq!(cloned.source_path_ids(), &[0, 129]);
    assert!(cloned.base.is_path_resolved);
}

#[test]
fn data_bind_path_decode_and_clone_preserve_path_and_authored_relative_flag() {
    let mut path = DataBindPath::default();
    let mut reader = BinaryReader::new(&[3, 0, 0x81, 1]);
    assert!(CoreObject::deserialize(
        &mut path,
        DataBindPathBase::PATH_PROPERTY_KEY,
        &mut reader,
    ));
    assert!(reader.reached_end());
    assert!(!reader.has_error());
    assert_eq!(path.path(), &[0, 129]);

    let mut relative = BinaryReader::new(&[1]);
    assert!(CoreObject::deserialize(
        &mut path,
        DataBindPathBase::IS_RELATIVE_PROPERTY_KEY,
        &mut relative,
    ));
    assert!(relative.reached_end());
    assert!(path.is_relative());
    path.set_resolved(true);

    let cloned = CoreObject::clone_boxed(&path).expect("cloned DataBindPath");
    let cloned = cloned.as_any().downcast_ref::<DataBindPath>().unwrap();
    assert_eq!(cloned.path(), &[0, 129]);
    assert!(cloned.base.resolved);
    assert!(cloned.is_relative());

    path.decode_path(&[7]);
    assert_eq!(path.path(), &[0, 129, 7]);
    assert_eq!(cloned.path(), &[0, 129]);
}

#[test]
fn data_bind_path_referencer_copy_keeps_the_generated_relative_field() {
    let mut source = DataBindPathReferencer::default();
    source.decode_data_bind_path(&[0, 0x81, 1]);
    source
        .with_data_bind_path_mut(|path| {
            let mut reader = BinaryReader::new(&[1]);
            assert!(CoreObject::deserialize(
                path,
                DataBindPathBase::IS_RELATIVE_PROPERTY_KEY,
                &mut reader,
            ));
            assert!(path.base.resolved);
        })
        .expect("source path");

    let mut copied = DataBindPathReferencer::default();
    copied.copy_data_bind_path(&source);
    copied
        .with_data_bind_path(|path| {
            assert_eq!(path.path(), &[0, 129]);
            assert!(path.base.resolved);
            assert!(path.is_relative());
        })
        .expect("copied path");
    source.with_data_bind_path_mut(|path| path.decode_path(&[7]));
    copied
        .with_data_bind_path(|path| assert_eq!(path.path(), &[0, 129]))
        .expect("independent copied path");
}

#[test]
fn null_context_constructor_and_null_main_setter_keep_upstream_distinct_states() {
    use nuxie_runtime::source::data_bind::data_context::DataContext;

    // data_context.cpp: the constructor omits null; viewModelInstance(nullptr)
    // retains a null first entry until the explicit main-removal operation.
    let mut context = DataContext::new(None);
    assert!(context.view_model_instances().is_empty());
    context.set_view_model_instance(None);
    assert_eq!(context.view_model_instances().len(), 1);
    assert!(context.view_model_instances()[0].is_none());
    assert!(context.main_view_model_instance().is_none());
    context.remove_main_view_model_instance();
    assert!(context.view_model_instances().is_empty());
}

#[test]
fn clearing_main_context_preserves_global_instances_and_slot_identity() {
    use nuxie_runtime::source::{
        core::CoreArena, data_bind::data_context::DataContext,
        viewmodel::viewmodel_instance::ViewModelInstance,
    };

    let arena = CoreArena::default();
    let main = arena.insert(ViewModelInstance::default());
    let global = arena.insert(ViewModelInstance::default());
    let mut context = DataContext::new(Some(main));
    context.set_view_model_instance_for_slot(7, Some(global.clone()));
    context.set_view_model_instance(None);
    assert!(context.main_view_model_instance().is_none());
    assert_eq!(context.view_model_instances().len(), 1);
    assert_eq!(context.instance_for_slot(7), Some(global));
    context.set_view_model_instance_for_slot(7, None);
    assert!(context.view_model_instances().is_empty());
}
