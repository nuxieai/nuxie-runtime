use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    data_bind::{converters::data_converter_rounder::DataConverterRounder, data_bind::DataBind},
    file::{RuntimeFileHandle, RuntimeFileWeakHandle},
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        data_bind::data_bind_base::DataBindBase,
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use std::any::Any;
struct BindingCloneProbe {
    inner: DataBind,
    new_file: RuntimeFileWeakHandle,
}
impl BindingCloneProbe {
    fn subtype(key: u16) -> bool {
        key == 65533 || DataBindBase::is_type_of(key)
    }
}
impl CoreCapabilities for BindingCloneProbe {
    fn as_data_bind(&self) -> Option<&DataBind> {
        Some(&self.inner)
    }
    fn as_data_bind_mut(&mut self) -> Option<&mut DataBind> {
        Some(&mut self.inner)
    }
    fn clone_completion_handler(&self) -> Option<fn(&CoreHandle, &CoreHandle) -> bool> {
        Some(|source, _cloned| {
            source
                .with_downcast_mut::<BindingCloneProbe, _>(|source| {
                    source.inner.set_file(source.new_file.clone())
                })
                .unwrap();
            true
        })
    }
}
impl CoreObject for BindingCloneProbe {
    fn core(&self) -> &Core {
        self.inner.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.inner.core_mut()
    }
    fn core_type(&self) -> u16 {
        65533
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        Some(Box::new(DataBind::default()))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.inner.deserialize(key, reader)
    }
}
impl CoreRegistryObject for BindingCloneProbe {
    fn as_registry_any(&self) -> &dyn Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_uint_with_completion(field, value, completion);
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_string_with_completion(field, value, completion);
    }
    fn set_color_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_color_with_completion(field, value, completion);
    }
    fn set_bool_with_completion(
        &mut self,
        field: CoreField,
        value: bool,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_bool_with_completion(field, value, completion);
    }
    fn set_double_with_completion(
        &mut self,
        field: CoreField,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_double_with_completion(field, value, completion);
    }
    fn set_callback_with_completion(
        &mut self,
        field: CoreField,
        value: CallbackData<'_>,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner
            .set_callback_with_completion(field, value, completion);
    }
    fn set_int_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.inner.set_int_with_completion(field, value, completion);
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        self.inner.get_uint(field)
    }
    fn get_string(&mut self, field: CoreField) -> String {
        self.inner.get_string(field)
    }
    fn get_color(&mut self, field: CoreField) -> i32 {
        self.inner.get_color(field)
    }
    fn get_bool(&mut self, field: CoreField) -> bool {
        self.inner.get_bool(field)
    }
    fn get_double(&mut self, field: CoreField) -> f32 {
        self.inner.get_double(field)
    }
    fn get_int(&mut self, field: CoreField) -> i32 {
        self.inner.get_int(field)
    }
}

fn file() -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    RuntimeFileHandle::new(File::new(
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
    ))
}
#[test]
fn inherited_clone_reads_each_binding_file_after_its_virtual_clone() {
    let arena = CoreArena::default();
    let before = file();
    let after = file();
    let mut inner = DataBind::default();
    inner.set_file(before.downgrade());
    let binding = arena.insert(BindingCloneProbe {
        inner,
        new_file: after.downgrade(),
    });
    let converter = arena.insert(DataConverterRounder::default());
    converter
        .data_bind_container()
        .unwrap()
        .add_data_bind(binding.clone());
    let clone = converter.clone_occurrence().unwrap();
    let cloned_binding = clone.data_bind_container().unwrap().data_binds()[0].clone();
    let actual = cloned_binding
        .with(|b| b.as_data_bind().unwrap().file().upgrade().unwrap())
        .unwrap();
    assert!(
        binding
            .with(|b| b.as_data_bind().unwrap().file().upgrade().unwrap())
            .unwrap()
            .ptr_eq(&after)
    );
    assert!(
        actual.ptr_eq(&after),
        "the source binding file is read after virtual cloning"
    );
    assert_eq!(
        cloned_binding.with(|b| b.as_data_bind().unwrap().target()),
        Some(Some(clone))
    );
}
