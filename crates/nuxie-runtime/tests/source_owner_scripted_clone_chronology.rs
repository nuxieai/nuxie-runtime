use nuxie_runtime::source::{
    assets::script_asset::ScriptAsset,
    core::{
        Core, CoreArena, CoreHandle, CoreObject, PropertySetterCompletion,
        binary_reader::BinaryReader, field_types::core_callback_type::CallbackData,
    },
    data_bind::data_bind::DataBind,
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        data_bind::data_bind_base::DataBindBase,
    },
    script_input_number::ScriptInputNumber,
    scripted::scripted_data_converter::ScriptedDataConverter,
};
use std::{any::Any, cell::Cell, rc::Rc};
struct BindingCloneProbe {
    inner: DataBind,
    callback: Rc<dyn Fn()>,
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
            let callback = source
                .with_downcast::<BindingCloneProbe, _>(|source| source.callback.clone())
                .unwrap();
            callback();
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

fn binding(arena: &CoreArena, owner: &CoreHandle, callback: Rc<dyn Fn()>) {
    let binding = arena.insert(BindingCloneProbe {
        inner: DataBind::default(),
        callback,
    });
    owner.data_bind_container().unwrap().add_data_bind(binding);
}
#[test]
fn scripted_clone_reads_properties_after_inherited_binding_callbacks() {
    let arena = CoreArena::default();
    let owner = arena.insert(ScriptedDataConverter::default());
    let property = arena.insert(ScriptInputNumber::default());
    let source = owner.clone();
    let added = property.clone();
    binding(
        &arena,
        &owner,
        Rc::new(move || {
            source
                .with_downcast_mut::<ScriptedDataConverter, _>(|source| {
                    source.add_property(added.clone())
                })
                .unwrap();
        }),
    );
    let clone = owner.clone_occurrence().unwrap();
    let properties = clone
        .with_downcast::<ScriptedDataConverter, _>(|clone| clone.properties.clone())
        .unwrap();
    assert_eq!(
        properties.len(),
        1,
        "source property membership is read after base cloning"
    );
    assert_ne!(properties[0], property);
    use nuxie_runtime::source::assets::script_asset::ScriptInputBehavior;
    assert_eq!(
        properties[0]
            .with_downcast::<ScriptInputNumber, _>(|input| input.script_input().scripted_object()),
        Some(Some(clone))
    );
}
#[test]
fn scripted_clone_selects_and_attaches_asset_after_inherited_binding_callbacks() {
    let arena = CoreArena::default();
    let owner = arena.insert(ScriptedDataConverter::default());
    let before = arena.insert(ScriptAsset::default());
    let after = arena.insert(ScriptAsset::default());
    owner
        .with_downcast_mut::<ScriptedDataConverter, _>(|source| {
            source
                .scripted
                .set_asset(owner.clone(), Some(before.clone()))
        })
        .unwrap();
    let count = Rc::new(Cell::new(0));
    let observed = count.clone();
    let source = owner.clone();
    let old = before.clone();
    let new = after.clone();
    binding(
        &arena,
        &owner,
        Rc::new(move || {
            observed.set(
                old.with(|asset| {
                    asset
                        .as_file_asset()
                        .unwrap()
                        .file_asset_base()
                        .file_asset_referencers()
                        .len()
                })
                .unwrap(),
            );
            source
                .with_downcast_mut::<ScriptedDataConverter, _>(|converter| {
                    converter
                        .scripted
                        .set_asset(source.clone(), Some(new.clone()))
                })
                .unwrap();
        }),
    );
    let clone = owner.clone_occurrence().unwrap();
    let selected = clone
        .with_downcast::<ScriptedDataConverter, _>(|clone| clone.scripted.script_asset())
        .unwrap();
    assert_eq!(
        count.get(),
        1,
        "the clone must not attach an asset before inherited cloning finishes"
    );
    assert_eq!(
        selected,
        Some(after.clone()),
        "source asset is read after inherited cloning"
    );
    assert!(
        before
            .with(|asset| asset
                .as_file_asset()
                .unwrap()
                .file_asset_base()
                .file_asset_referencers()
                .is_empty())
            .unwrap()
    );
    let owners = after
        .with(|asset| {
            asset
                .as_file_asset()
                .unwrap()
                .file_asset_base()
                .file_asset_referencers()
                .to_vec()
        })
        .unwrap();
    assert_eq!(owners, vec![owner, clone]);
}

struct PropertyCloneProbe {
    inner: ScriptInputNumber,
    callback: Rc<dyn Fn(&CoreHandle)>,
    plain_clone: bool,
    plain_type: bool,
}
impl PropertyCloneProbe {
    fn subtype(key: u16) -> bool {
        nuxie_runtime::source::generated::script_input_number_base::ScriptInputNumberBase::is_type_of(key)
    }
}
impl CoreCapabilities for PropertyCloneProbe {
    fn script_input_data_bind(&self) -> Option<CoreHandle> {
        self.inner.script_input_data_bind()
    }
    fn script_input_set_scripted_object(&mut self, owner: Option<CoreHandle>) -> bool {
        self.inner.script_input_set_scripted_object(owner)
    }
    fn script_input_set_data_bind(&mut self, bind: Option<CoreHandle>, owned: bool) -> bool {
        self.inner.script_input_set_data_bind(bind, owned)
    }
    fn clone_completion_handler(&self) -> Option<fn(&CoreHandle, &CoreHandle) -> bool> {
        Some(|source, cloned| {
            let callback = source
                .with_downcast::<Self, _>(|s| s.callback.clone())
                .unwrap();
            callback(cloned);
            true
        })
    }
}
impl CoreObject for PropertyCloneProbe {
    fn core(&self) -> &Core {
        self.inner.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.inner.core_mut()
    }
    fn core_type(&self) -> u16 {
        if self.plain_type {
            CoreObject::core_type(
                &nuxie_runtime::source::custom_property_number::CustomPropertyNumber::default(),
            )
        } else {
            CoreObject::core_type(&self.inner)
        }
    }
    fn is_type_of(&self, key: u16) -> bool {
        Self::subtype(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        Self::subtype
    }
    fn clone_boxed(&self) -> Option<Box<dyn CoreObject>> {
        Some(if self.plain_clone {
            Box::new(nuxie_runtime::source::custom_property_number::CustomPropertyNumber::default())
        } else {
            Box::new(ScriptInputNumber::default())
        })
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.inner.deserialize(key, reader)
    }
}
impl CoreRegistryObject for PropertyCloneProbe {
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

struct CapturingBind {
    inner: DataBind,
    cloned: Rc<std::cell::RefCell<Option<CoreHandle>>>,
}
impl CapturingBind {
    fn subtype(key: u16) -> bool {
        key == 65533 || DataBindBase::is_type_of(key)
    }
}
impl CoreCapabilities for CapturingBind {
    fn as_data_bind(&self) -> Option<&DataBind> {
        Some(&self.inner)
    }
    fn as_data_bind_mut(&mut self) -> Option<&mut DataBind> {
        Some(&mut self.inner)
    }
    fn clone_completion_handler(&self) -> Option<fn(&CoreHandle, &CoreHandle) -> bool> {
        Some(|source, clone| {
            source
                .with_downcast::<Self, _>(|s| *s.cloned.borrow_mut() = Some(clone.clone()))
                .unwrap();
            true
        })
    }
}
impl CoreObject for CapturingBind {
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
impl CoreRegistryObject for CapturingBind {
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

#[test]
fn scripted_clone_reads_binding_lists_after_each_property_clone_callback() {
    use std::cell::RefCell;
    let arena = CoreArena::default();
    let owner = arena.insert(ScriptedDataConverter::default());
    let cloned_binding = Rc::new(RefCell::new(None::<CoreHandle>));
    let capture = cloned_binding.clone();
    // Capture the inherited clone through a dedicated callback probe below.
    let slot = cloned_binding.clone();
    let source_owner = owner.clone();
    let twin_extra = Rc::new(RefCell::new(None::<CoreHandle>));
    let seen = twin_extra.clone();
    let property_slot = Rc::new(RefCell::new(None::<CoreHandle>));
    let source_property = property_slot.clone();
    let property = arena.insert(PropertyCloneProbe {
        inner: ScriptInputNumber::default(),
        plain_type: false,
        plain_clone: false,
        callback: Rc::new(move |_| {
            let inherited = slot.borrow().clone().unwrap();
            let twin = inherited
                .with(|b| b.as_data_bind().unwrap().target().unwrap())
                .unwrap();
            let graph = twin.retain_arena().unwrap();
            let extra_source = graph.insert(DataBind::default());
            extra_source.with_mut(|b| {
                b.as_data_bind_mut()
                    .unwrap()
                    .set_target(source_property.borrow().clone())
            });
            source_owner
                .data_bind_container()
                .unwrap()
                .add_data_bind(extra_source);
            let extra_twin = graph.insert(DataBind::default());
            extra_twin.with_mut(|b| b.as_data_bind_mut().unwrap().set_target(Some(twin.clone())));
            twin.data_bind_container()
                .unwrap()
                .add_data_bind(extra_twin.clone());
            *seen.borrow_mut() = Some(extra_twin);
        }),
    });
    *property_slot.borrow_mut() = Some(property.clone());
    // The callback records a weak clone identity. No source membership changes occur during binding cloning.
    let bind_capture = arena.insert(CapturingBind {
        inner: DataBind::default(),
        cloned: capture,
    });
    bind_capture.with_mut(|b| {
        b.as_data_bind_mut()
            .unwrap()
            .set_target(Some(property.clone()))
    });
    property.with_mut(|p| {
        p.script_input_set_data_bind(Some(bind_capture.clone()), false);
    });
    owner
        .data_bind_container()
        .unwrap()
        .add_data_bind(bind_capture);
    owner
        .with_downcast_mut::<ScriptedDataConverter, _>(|o| o.add_property(property.clone()))
        .unwrap();
    let twin = owner.clone_occurrence().unwrap();
    let twin_property = twin
        .with_downcast::<ScriptedDataConverter, _>(|o| o.properties[0].clone())
        .unwrap();
    let extra = twin_extra.borrow().clone().unwrap();
    assert_eq!(
        extra.with(|b| b.as_data_bind().unwrap().target()),
        Some(Some(twin_property.clone()))
    );
    assert_eq!(
        twin_property.with(|p| p.script_input_data_bind()),
        Some(Some(extra))
    );
}
#[test]
fn scripted_clone_does_not_retarget_bindings_when_input_clones_to_plain_property() {
    let arena = CoreArena::default();
    let owner = arena.insert(ScriptedDataConverter::default());
    let property = arena.insert(PropertyCloneProbe {
        inner: ScriptInputNumber::default(),
        plain_type: false,
        plain_clone: true,
        callback: Rc::new(|_| {}),
    });
    let bind = arena.insert(DataBind::default());
    bind.with_mut(|b| {
        b.as_data_bind_mut()
            .unwrap()
            .set_target(Some(property.clone()))
    });
    property.with_mut(|p| {
        p.script_input_set_data_bind(Some(bind.clone()), false);
    });
    owner.data_bind_container().unwrap().add_data_bind(bind);
    owner
        .with_downcast_mut::<ScriptedDataConverter, _>(|o| o.add_property(property))
        .unwrap();
    let twin = owner.clone_occurrence().unwrap();
    let twin_bind = twin.data_bind_container().unwrap().data_binds()[0].clone();
    assert_eq!(
        twin_bind.with(|b| b.as_data_bind().unwrap().target()),
        Some(Some(twin))
    );
}

#[test]
fn scripted_add_property_uses_source_input_type_gate_before_projection() {
    use nuxie_runtime::source::assets::script_asset::ScriptInputBehavior;
    let arena = CoreArena::default();
    let owner = arena.insert(ScriptedDataConverter::default());
    let property = arena.insert(PropertyCloneProbe {
        inner: ScriptInputNumber::default(),
        plain_type: true,
        plain_clone: true,
        callback: Rc::new(|_| {}),
    });
    owner
        .with_downcast_mut::<ScriptedDataConverter, _>(|owner| owner.add_property(property.clone()))
        .unwrap();
    assert_eq!(
        property
            .with_downcast::<PropertyCloneProbe, _>(|p| p.inner.script_input().scripted_object()),
        Some(None)
    );
    assert_eq!(
        owner.with_downcast::<ScriptedDataConverter, _>(|owner| owner.properties.clone()),
        Some(vec![property])
    );
}

#[test]
fn scripted_clone_copies_inherited_name_after_callbacks_but_own_id_before() {
    use nuxie_runtime::source::generated::{
        core_registry::CoreRegistry, data_bind::converters::data_converter_base::DataConverterBase,
        scripted::scripted_data_converter_base::ScriptedDataConverterBase,
    };
    let arena = CoreArena::default();
    let owner = arena.insert(ScriptedDataConverter::default());
    let name = i32::from(DataConverterBase::NAME_PROPERTY_KEY);
    let asset = i32::from(ScriptedDataConverterBase::SCRIPT_ASSET_ID_PROPERTY_KEY);
    assert!(CoreRegistry::set_string_handle(
        &owner,
        name,
        "before".into()
    ));
    assert!(CoreRegistry::set_uint_handle(&owner, asset, 7));
    let source = owner.clone();
    binding(
        &arena,
        &owner,
        Rc::new(move || {
            assert!(CoreRegistry::set_string_handle(
                &source,
                name,
                "after".into()
            ));
            assert!(CoreRegistry::set_uint_handle(&source, asset, 9));
        }),
    );
    let clone = owner.clone_occurrence().unwrap();
    assert_eq!(
        clone.with_downcast::<ScriptedDataConverter, _>(|clone| (
            clone.base.base.base.name().to_owned(),
            clone.base.script_asset_id()
        )),
        Some(("after".into(), 7))
    );
}
#[test]
fn ordinary_converter_clone_copies_name_after_owned_binding_callbacks() {
    use nuxie_runtime::source::{
        data_bind::converters::data_converter_rounder::DataConverterRounder,
        generated::{
            core_registry::CoreRegistry,
            data_bind::converters::data_converter_base::DataConverterBase,
        },
    };
    let arena = CoreArena::default();
    let owner = arena.insert(DataConverterRounder::default());
    let name = i32::from(DataConverterBase::NAME_PROPERTY_KEY);
    assert!(CoreRegistry::set_string_handle(
        &owner,
        name,
        "before".into()
    ));
    let source = owner.clone();
    binding(
        &arena,
        &owner,
        Rc::new(move || {
            assert!(CoreRegistry::set_string_handle(
                &source,
                name,
                "after".into()
            ));
        }),
    );
    let clone = owner.clone_occurrence().unwrap();
    assert_eq!(
        clone.with(|clone| clone.as_data_converter().unwrap().base.name().to_owned()),
        Some("after".into())
    );
}

fn number_to_list_file_case(clear: bool) {
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::source::{
        data_bind::converters::data_converter_number_to_list::DataConverterNumberToList,
        file::RuntimeFileHandle,
        generated::{
            core_registry::CoreRegistry,
            data_bind::converters::data_converter_number_to_list_base::DataConverterNumberToListBase,
        },
    };
    use nuxie_runtime::{File, RuntimeFactoryHandle};
    let file = || {
        let mut f = PersistentFactory::new(RecordingFactory::new());
        RuntimeFileHandle::new(File::new(
            RuntimeFactoryHandle::from_factory(&mut f).unwrap(),
            None,
        ))
    };
    let before = file();
    let after = file();
    let arena = CoreArena::default();
    let owner = arena.insert(DataConverterNumberToList::new(17));
    owner
        .with_downcast_mut::<DataConverterNumberToList, _>(|c| c.set_file(Some(before.downgrade())))
        .unwrap();
    let source = owner.clone();
    let replacement = (!clear).then(|| after.downgrade());
    binding(
        &arena,
        &owner,
        Rc::new(move || {
            source
                .with_downcast_mut::<DataConverterNumberToList, _>(|c| {
                    c.set_file(replacement.clone())
                })
                .unwrap();
            assert!(CoreRegistry::set_uint_handle(
                &source,
                i32::from(DataConverterNumberToListBase::VIEW_MODEL_ID_PROPERTY_KEY),
                23
            ));
        }),
    );
    let cloned = owner.clone_occurrence().unwrap();
    let (selected, id) = cloned
        .with_downcast::<DataConverterNumberToList, _>(|c| (c.file(), c.base.view_model_id()))
        .unwrap();
    assert_eq!(id, 17, "generated own field copy precedes base cloning");
    if clear {
        assert!(selected.is_none(), "manual file copy follows base cloning");
    } else {
        assert!(
            selected.unwrap().upgrade().unwrap().ptr_eq(&after),
            "manual file copy follows base cloning"
        );
    }
}
#[test]
fn number_to_list_reads_replaced_file_after_base_clone_callbacks() {
    number_to_list_file_case(false);
}
#[test]
fn number_to_list_reads_cleared_file_after_base_clone_callbacks() {
    number_to_list_file_case(true);
}

#[test]
fn registered_number_to_list_file_stays_default_during_base_clone_callbacks() {
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::{File, RuntimeFactoryHandle};
    use nuxie_runtime::source::{file::RuntimeFileHandle, data_bind::converters::data_converter_number_to_list::DataConverterNumberToList};
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let file = RuntimeFileHandle::new(File::new(RuntimeFactoryHandle::from_factory(&mut factory).unwrap(), None));
    let arena = CoreArena::default();
    let mut converter = DataConverterNumberToList::new(17);
    converter.set_file(Some(file.downgrade()));
    let owner = arena.insert(converter);
    let captured = Rc::new(std::cell::RefCell::new(None));
    let first = arena.insert(CapturingBind { inner: DataBind::default(), cloned: captured.clone() });
    owner.with_mut(|o| o.as_data_converter_mut().unwrap().add_data_bind(first)).unwrap();
    let seen = Rc::new(Cell::new(false));
    let saw = seen.clone();
    binding(&arena, &owner, Rc::new(move || {
        let first_clone = captured.borrow().clone().unwrap();
        let partial = first_clone.with(|b| b.as_data_bind().unwrap().target()).unwrap().unwrap();
        assert!(partial.with_downcast::<DataConverterNumberToList, _>(|o| o.file().is_none()).unwrap(), "source constructor leaves file null until the manual override runs");
        saw.set(true);
    }));
    let cloned = owner.clone_occurrence().unwrap();
    assert!(seen.get());
    assert!(cloned.with_downcast::<DataConverterNumberToList, _>(|o| o.file().unwrap().upgrade().unwrap().ptr_eq(&file)).unwrap());
}

fn native_name_stays_default_during_base_clone(scripted: bool) {
    use nuxie_runtime::source::{data_bind::converters::data_converter_rounder::DataConverterRounder, generated::{core_registry::CoreRegistry, data_bind::converters::data_converter_base::DataConverterBase}};
    let arena = CoreArena::default();
    let owner = if scripted { arena.insert(ScriptedDataConverter::default()) } else { arena.insert(DataConverterRounder::new(2)) };
    assert!(CoreRegistry::set_string_handle(&owner, i32::from(DataConverterBase::NAME_PROPERTY_KEY), "source-name".into()));
    let captured = Rc::new(std::cell::RefCell::new(None));
    let first = arena.insert(CapturingBind { inner: DataBind::default(), cloned: captured.clone() });
    owner.with_mut(|o| o.as_data_converter_mut().unwrap().add_data_bind(first)).unwrap();
    let seen = Rc::new(Cell::new(false));
    let saw = seen.clone();
    binding(&arena, &owner, Rc::new(move || {
        let first_clone = captured.borrow().clone().unwrap();
        let partial = first_clone.with(|b| b.as_data_bind().unwrap().target()).unwrap().unwrap();
        assert_eq!(partial.with(|o| o.as_data_converter().unwrap().base.name().to_owned()), Some(String::new()), "inherited Name copy occurs only after all owned binding clones");
        saw.set(true);
    }));
    let cloned = owner.clone_occurrence().unwrap();
    assert!(seen.get());
    assert_eq!(cloned.with(|o| o.as_data_converter().unwrap().base.name().to_owned()), Some("source-name".into()));
}
#[test]
fn native_rounder_name_stays_default_during_base_clone_callbacks() {
    native_name_stays_default_during_base_clone(false);
}
#[test]
fn native_scripted_name_stays_default_during_base_clone_callbacks() {
    native_name_stays_default_during_base_clone(true);
}
