use nuxie_runtime::source::{
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    data_bind::converters::{
        data_converter::DataConverter, data_converter_number_to_list::DataConverterNumberToList,
    },
    data_bind::data_bind::DataBind,
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        data_bind::converters::data_converter_number_to_list_base::DataConverterNumberToListBase,
    },
};
use std::any::Any;
struct OpaqueNumberToList {
    inner: DataConverterNumberToList,
    trait_delegate: bool,
    order: Option<std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>>,
}
impl OpaqueNumberToList {
    fn subtype(key: u16) -> bool {
        key == 65533 || DataConverterNumberToListBase::is_type_of(key)
    }
}
impl CoreCapabilities for OpaqueNumberToList {
    fn as_data_converter(&self) -> Option<&DataConverter> {
        Some(&self.inner.base.base)
    }
    fn as_data_converter_mut(&mut self) -> Option<&mut DataConverter> {
        Some(&mut self.inner.base.base)
    }
    fn clone_completion_handler(&self) -> Option<fn(&CoreHandle, &CoreHandle) -> bool> {
        if let Some(order) = &self.order {
            order.borrow_mut().push("handler");
        }
        if self.trait_delegate {
            self.inner.clone_completion_handler()
        } else {
            Some(DataConverterNumberToList::complete_clone)
        }
    }
}
impl CoreObject for OpaqueNumberToList {
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
        if let Some(order) = &self.order {
            order.borrow_mut().push("definition");
            return None;
        }
        if self.trait_delegate {
            CoreObject::clone_boxed(&self.inner)
        } else {
            Some(Box::new(self.inner.clone_definition()))
        }
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.inner.deserialize(key, reader)
    }
}
impl CoreRegistryObject for OpaqueNumberToList {
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

fn opaque_number_to_list_case(trait_delegate: bool) {
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::source::file::RuntimeFileHandle;
    use nuxie_runtime::{File, RuntimeFactoryHandle};
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = RuntimeFileHandle::new(File::new(
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
    ));
    let arena = CoreArena::default();
    let mut inner = DataConverterNumberToList::new(17);
    inner.set_file(Some(file.downgrade()));
    let owner = arena.insert(OpaqueNumberToList {
        inner,
        trait_delegate,
        order: None,
    });
    assert!(
        owner
            .with_downcast::<DataConverterNumberToList, _>(|_| ())
            .is_none()
    );
    assert!(owner.data_bind_container().is_some());
    let bind = arena.insert(DataBind::default());
    owner
        .with_mut(|o| o.as_data_converter_mut().unwrap().add_data_bind(bind))
        .unwrap();
    owner
        .with_mut(|o| {
            let definition = o.clone_boxed().unwrap();
            assert!(
                definition
                    .as_any()
                    .downcast_ref::<DataConverterNumberToList>()
                    .unwrap()
                    .file()
                    .unwrap()
                    .upgrade()
                    .unwrap()
                    .ptr_eq(&file)
            );
            assert!(o.clone_completion_handler().is_some());
        })
        .unwrap();
    let cloned = owner
        .clone_occurrence()
        .expect("public forwarding requires only base capability");
    assert!(
        cloned
            .with_downcast::<DataConverterNumberToList, _>(|o| o
                .file()
                .unwrap()
                .upgrade()
                .unwrap()
                .ptr_eq(&file))
            .unwrap()
    );
    assert_eq!(
        cloned.with_downcast::<DataConverterNumberToList, _>(|o| o.base.view_model_id()),
        Some(17)
    );
    let children = cloned
        .with(|o| o.as_data_converter().unwrap().data_binds())
        .unwrap();
    assert_eq!(children.len(), 1);
    assert_eq!(
        children[0].with(|o| o.as_data_bind().unwrap().target()),
        Some(Some(cloned))
    );
}

#[test]
fn opaque_number_to_list_keeps_public_definition_and_completion_forwarding() {
    opaque_number_to_list_case(false);
}
#[test]
fn opaque_number_to_list_keeps_public_trait_forwarding() {
    opaque_number_to_list_case(true);
}

#[test]
fn default_occurrence_hook_queries_handler_after_a_missing_definition() {
    let arena = CoreArena::default();
    let order = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let owner = arena.insert(OpaqueNumberToList {
        inner: DataConverterNumberToList::default(),
        trait_delegate: false,
        order: Some(order.clone()),
    });
    assert!(owner.clone_occurrence().is_none());
    assert_eq!(*order.borrow(), ["definition", "handler"]);
}
