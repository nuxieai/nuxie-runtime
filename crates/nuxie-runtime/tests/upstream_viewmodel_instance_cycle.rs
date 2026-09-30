//! All four cases from 57628249 viewmodel_instance_cycle_test.cpp.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    core::CoreHandle,
    generated::{
        core_registry::CoreRegistry,
        viewmodel::{
            viewmodel_instance_base::ViewModelInstanceBase,
            viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase,
            viewmodel_property_viewmodel_base::ViewModelPropertyViewModelBase,
        },
    },
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
        viewmodel_property_viewmodel::ViewModelPropertyViewModel,
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle, RuntimeFileHandle};

fn fixture() -> RuntimeFileHandle {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    let bytes =
        std::fs::read(root.join("tests/unit_tests/assets/rebind_with_nested_viewmodel.riv"))
            .unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    File::import(&bytes, factory, None, None, None).unwrap()
}

struct Reference {
    view_model_index: usize,
    instance_index: usize,
    property: CoreHandle,
    value: CoreHandle,
}

// Find the first nested reference in the real fixture, then corrupt the
// authored ids in precisely the same way as the source malformed-export cases.
fn find_reference(file: &File) -> Option<Reference> {
    for i in 0..file.view_model_count() {
        let Some(model) = file.view_model_handle(i) else {
            continue;
        };
        let count = model
            .with_downcast::<ViewModel, _>(ViewModel::instance_count)
            .unwrap();
        for j in 0..count {
            let Some(instance) = model
                .with_downcast::<ViewModel, _>(|model| model.instance_at(j))
                .flatten()
            else {
                continue;
            };
            let values = instance
                .with_downcast::<ViewModelInstance, _>(|instance| {
                    instance.property_values().to_vec()
                })
                .unwrap();
            for value in values {
                let Some(property_id) =
                    value.with_downcast::<ViewModelInstanceViewModel, _>(|value| {
                        value.base.view_model_property_id()
                    })
                else {
                    continue;
                };
                let Some(property) = model
                    .with_downcast::<ViewModel, _>(|model| model.property_at(property_id as usize))
                    .flatten()
                else {
                    continue;
                };
                if property
                    .with_downcast::<ViewModelPropertyViewModel, _>(|_| ())
                    .is_none()
                {
                    continue;
                }
                return Some(Reference {
                    view_model_index: i,
                    instance_index: j,
                    property,
                    value,
                });
            }
        }
    }
    None
}

fn integer(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, i32::from(key), value));
}

#[test]
fn a_view_model_id_past_the_end_of_the_file_is_ignored() {
    let file = fixture();
    let reference = file.with_file(find_reference).expect("reference found");
    let source = file
        .with_file(|file| file.view_model_handle(reference.view_model_index))
        .unwrap()
        .with_downcast::<ViewModel, _>(|model| model.instance_at(reference.instance_index))
        .flatten()
        .unwrap();
    integer(
        &source,
        ViewModelInstanceBase::VIEW_MODEL_ID_PROPERTY_KEY,
        file.with_file(File::view_model_count) as u32 + 10,
    );
    assert!(
        file.with_file(|file| file
            .create_view_model_instance_at(reference.view_model_index, reference.instance_index))
            .is_some()
    );
}

#[test]
fn a_nested_reference_past_the_end_of_the_file_is_ignored() {
    let file = fixture();
    let reference = file.with_file(find_reference).expect("reference found");
    integer(
        &reference.property,
        ViewModelPropertyViewModelBase::VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY,
        file.with_file(File::view_model_count) as u32 + 10,
    );
    let instance = file.with_file(|file| {
        file.create_view_model_instance_at(reference.view_model_index, reference.instance_index)
    });
    assert!(instance.is_some());
}

#[test]
fn a_self_referential_view_model_instance_terminates() {
    let file = fixture();
    let reference = file.with_file(find_reference).expect("reference found");
    integer(
        &reference.property,
        ViewModelPropertyViewModelBase::VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY,
        reference.view_model_index as u32,
    );
    integer(
        &reference.value,
        ViewModelInstanceViewModelBase::PROPERTY_VALUE_PROPERTY_KEY,
        reference.instance_index as u32,
    );
    let instance = file
        .with_file(|file| {
            file.create_view_model_instance_at(reference.view_model_index, reference.instance_index)
        })
        .expect("copied instance");
    let copied_value = instance
        .with_downcast::<ViewModelInstance, _>(|instance| {
            instance
                .property_values()
                .iter()
                .find(|value| {
                    value
                        .with_downcast::<ViewModelInstanceViewModel, _>(|_| ())
                        .is_some()
                })
                .cloned()
        })
        .flatten()
        .expect("copied nested value");
    // Dropping the back-edge avoids a strong self-reference in the source model.
    assert!(
        copied_value
            .with_downcast::<ViewModelInstanceViewModel, _>(
                ViewModelInstanceViewModel::reference_view_model_instance
            )
            .unwrap()
            .is_none()
    );
}

#[test]
fn copying_a_null_view_model_instance_returns_null() {
    let file = fixture();
    assert!(
        file.with_file(|file| file.copy_view_model_instance(None))
            .is_none()
    );
}
