use nuxie_runtime::source::{
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    data_bind::data_bind::DataBind,
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistry, CoreRegistryObject},
        data_bind::converters::data_converter_base::DataConverterBase,
        scripted::scripted_data_converter_base::ScriptedDataConverterBase,
    },
    scripted::scripted_data_converter::ScriptedDataConverter,
};
use std::any::Any;
struct ProjectedScripted {
    inner: ScriptedDataConverter,
}
impl ProjectedScripted {
    fn subtype(key: u16) -> bool {
        key == 65533 || ScriptedDataConverterBase::is_type_of(key)
    }
}
impl CoreCapabilities for ProjectedScripted {
    fn clone_completion_handler(&self) -> Option<fn(&CoreHandle, &CoreHandle) -> bool> {
        Some(ScriptedDataConverter::complete_clone)
    }
}
impl CoreObject for ProjectedScripted {
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
        Some(Box::new(self.inner.clone_definition()))
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.inner.deserialize(key, reader)
    }
}
impl CoreRegistryObject for ProjectedScripted {
    fn as_registry_any(&self) -> &dyn Any {
        &self.inner
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        &mut self.inner
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

#[test]
fn projected_scripted_clone_keeps_its_direct_owner_route_without_base_capabilities() {
    for binding_count in [0, 1] {
        let arena = CoreArena::default();
        let owner = arena.insert(ProjectedScripted {
            inner: ScriptedDataConverter::default(),
        });
        assert!(owner.with(|o| o.as_data_converter().is_none()).unwrap());
        assert!(owner.data_bind_container().is_none());
        assert!(CoreRegistry::set_string_handle(
            &owner,
            i32::from(DataConverterBase::NAME_PROPERTY_KEY),
            "projected".into()
        ));
        if binding_count == 1 {
            let bind = arena.insert(DataBind::default());
            owner
                .with_downcast_mut::<ScriptedDataConverter, _>(|o| o.base.base.add_data_bind(bind))
                .unwrap();
        }
        let cloned = owner
            .clone_occurrence()
            .expect("the projected Scripted owner does not need an additional base capability");
        assert_eq!(
            cloned.with_downcast::<ScriptedDataConverter, _>(|o| o
                .base
                .base
                .base
                .name()
                .to_owned()),
            Some("projected".into())
        );
        let children = cloned
            .with_downcast::<ScriptedDataConverter, _>(|o| o.base.base.data_binds())
            .unwrap();
        assert_eq!(children.len(), binding_count);
        for child in children {
            assert_eq!(
                child.with(|b| b.as_data_bind().unwrap().target()),
                Some(Some(cloned.clone()))
            );
        }
    }
}
