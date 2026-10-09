//! Most-derived generated layout callback dispatch against pinned160085.
use std::{cell::Cell, path::PathBuf};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle};
use nuxie_runtime::source::{
    artboard::Artboard, core::{CoreArena, CoreHandle, Core, CoreObject, PropertySetterCompletion, binary_reader::BinaryReader, field_types::core_callback_type::CallbackData}, core_context::{CoreContext, StatusCode},
    generated::{core_registry::{CoreRegistry, CoreRegistryObject, CoreCapabilities, CoreField}, component_base::ComponentBase,
        layout::layout_sizing_style_base::LayoutSizingStyleBase as Sizing,
        layout::layout_node_style_base::LayoutNodeStyleBase as NodeStyle},
    layout::{layout_node_style::LayoutNodeStyle, layout_participant::LayoutParticipant},
    layout_component::LayoutComponent,
};
thread_local! { static DIRTY: Cell<u32> = const { Cell::new(0) }; }
fn layout_dirt(_: *mut ()) { DIRTY.with(|v| v.set(v.get()+1)); }
struct Context<'a> { arena: &'a CoreArena, objects: Vec<CoreHandle> }
impl CoreContext for Context<'_> {
    fn core_arena(&self)->&CoreArena { self.arena }
    fn resolve_handle(&self,id:u32)->Option<CoreHandle> { self.objects.get(id as usize).cloned() }
}
#[test]
fn node_style_inherited_sizing_callbacks_dirty_the_live_parent_layout() {
    let keys = [Sizing::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,Sizing::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
        Sizing::WIDTH_UNITS_VALUE_PROPERTY_KEY,Sizing::HEIGHT_UNITS_VALUE_PROPERTY_KEY,
        Sizing::MIN_WIDTH_UNITS_VALUE_PROPERTY_KEY,Sizing::MAX_WIDTH_UNITS_VALUE_PROPERTY_KEY,
        Sizing::MIN_HEIGHT_UNITS_VALUE_PROPERTY_KEY,Sizing::MAX_HEIGHT_UNITS_VALUE_PROPERTY_KEY,
        Sizing::JUSTIFY_SELF_VALUE_PROPERTY_KEY,Sizing::DISPLAY_VALUE_PROPERTY_KEY,
        Sizing::MIN_WIDTH_PROPERTY_KEY,Sizing::MAX_WIDTH_PROPERTY_KEY,Sizing::MIN_HEIGHT_PROPERTY_KEY,Sizing::MAX_HEIGHT_PROPERTY_KEY];
    let mut missed=Vec::new();
    for key in keys {
        let arena=CoreArena::default();
        let root=arena.insert(Artboard::default());
        let parent=arena.insert(LayoutComponent::default());
        let style=arena.insert(LayoutNodeStyle::default());
        let mut context=Context{arena:&arena,objects:vec![root.clone(),parent.clone(),style.clone()]};
        for (owner,parent_id) in [(&parent,0),(&style,1)] {
            assert_eq!(owner.with_mut(|object| {
                CoreRegistry::set_uint(object,ComponentBase::PARENT_ID_PROPERTY_KEY.into(),parent_id);
                object.as_component_mut().unwrap().on_added_dirty(&mut context)
            }),Some(StatusCode::Ok));
        }
        root.with_downcast_mut::<Artboard,_>(|root| {root.clean_layout(&parent);root.on_layout_dirty(Some(layout_dirt));}).unwrap();
        DIRTY.with(|v|v.set(0));
        if [Sizing::MIN_WIDTH_PROPERTY_KEY,Sizing::MAX_WIDTH_PROPERTY_KEY,Sizing::MIN_HEIGHT_PROPERTY_KEY,Sizing::MAX_HEIGHT_PROPERTY_KEY].contains(&key) {
            assert!(CoreRegistry::set_double_handle(&style,key.into(),12.0));
        } else {
            let prior=CoreRegistry::get_uint_handle(&style,key.into()).unwrap();
            assert!(CoreRegistry::set_uint_handle(&style,key.into(),prior ^ 1));
        }
        if DIRTY.with(Cell::get)==0 {missed.push(key);}
    }
    assert!(missed.is_empty(),"sizing keys failed to dirty parent layout: {missed:?}");
}
fn participant() -> (RuntimeFileHandle,CoreHandle,CoreHandle) {
    let root=std::env::var_os("RIVE_RUNTIME_DIR").unwrap();
    let bytes=std::fs::read(PathBuf::from(root).join("tests/unit_tests/assets/layout/fixed_participant.riv")).unwrap();
    let mut factory=PersistentFactory::new(RecordingFactory::new());
    let retained=RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let mut result=ImportResult::Malformed;
    let file=File::import(&bytes,retained,Some(&mut result),None,None).unwrap();
    assert_eq!(result,ImportResult::Success);
    let root=file.with_file(|file|file.artboard()).unwrap();
    Artboard::advance_handle(&root,0.0,nuxie_runtime::source::advance_flags::AdvanceFlags::ANIMATE);
    let participants=root.with_downcast::<Artboard,_>(|a|a.find_all_handles::<LayoutParticipant>()).unwrap();
    assert_eq!(participants.len(),1);
    let p=participants[0].clone();
    let host=p.with(|p|p.component_parent_handle()).flatten().unwrap();
    (file,p,host)
}
#[test]
fn participant_display_callback_synchronizes_the_host_collapse_state() {
    let (_file,p,host)=participant();
    assert_eq!(host.with(|h|h.component_is_collapsed()),Some(false));
    assert!(CoreRegistry::set_uint_handle(&p,Sizing::DISPLAY_VALUE_PROPERTY_KEY.into(),1));
    assert_eq!(host.with(|h|h.component_is_collapsed()),Some(true));
}
#[test]
fn participant_width_callback_uses_its_own_sizing_handler() {
    let (_file,p,_host)=participant();
    let value=CoreRegistry::get_double_handle(&p,NodeStyle::WIDTH_PROPERTY_KEY.into()).unwrap();
    assert!(CoreRegistry::set_double_handle(&p,NodeStyle::WIDTH_PROPERTY_KEY.into(),value+10.0));
}
#[test]
fn every_participant_sizing_callback_reaches_the_registered_source_operation() {
    let keys = [Sizing::LAYOUT_WIDTH_SCALE_TYPE_PROPERTY_KEY,Sizing::LAYOUT_HEIGHT_SCALE_TYPE_PROPERTY_KEY,
        Sizing::HUG_UNBOUNDED_PROPERTY_KEY,NodeStyle::WIDTH_PROPERTY_KEY,NodeStyle::HEIGHT_PROPERTY_KEY,
        NodeStyle::FRACTIONAL_WIDTH_PROPERTY_KEY,NodeStyle::FRACTIONAL_HEIGHT_PROPERTY_KEY,
        Sizing::WIDTH_UNITS_VALUE_PROPERTY_KEY,Sizing::HEIGHT_UNITS_VALUE_PROPERTY_KEY,
        Sizing::MIN_WIDTH_UNITS_VALUE_PROPERTY_KEY,Sizing::MAX_WIDTH_UNITS_VALUE_PROPERTY_KEY,
        Sizing::MIN_HEIGHT_UNITS_VALUE_PROPERTY_KEY,Sizing::MAX_HEIGHT_UNITS_VALUE_PROPERTY_KEY,
        Sizing::JUSTIFY_SELF_VALUE_PROPERTY_KEY,Sizing::DISPLAY_VALUE_PROPERTY_KEY,
        Sizing::MIN_WIDTH_PROPERTY_KEY,Sizing::MAX_WIDTH_PROPERTY_KEY,Sizing::MIN_HEIGHT_PROPERTY_KEY,Sizing::MAX_HEIGHT_PROPERTY_KEY];
    let mut missed=Vec::new();
    for key in keys {
        let (_file,p,host)=participant();
        let root=host.with(|h|h.component_artboard_handle()).flatten().unwrap();
        root.with_downcast_mut::<Artboard,_>(|root| {
            for layout in root.find_all_handles::<LayoutComponent>() {root.clean_layout(&layout);}
            root.on_layout_dirty(Some(layout_dirt));
        }).unwrap();
        DIRTY.with(|v|v.set(0));
        let applier_arena = CoreArena::default();
        let calls = std::rc::Rc::new(Cell::new(0));
        let applier = applier_arena.insert(OpaqueParticipant { inner: LayoutParticipant::default(), style_calls: Some(calls.clone()) });
        p.with_downcast_mut::<LayoutParticipant, _>(|p| p.add_layout_style_applier(applier)).unwrap();
        match CoreRegistry::property_field_id(key.into()) {
            0 => {let old=CoreRegistry::get_uint_handle(&p,key.into()).unwrap();assert!(CoreRegistry::set_uint_handle(&p,key.into(),old^1));},
            2 => {let old=CoreRegistry::get_double_handle(&p,key.into()).unwrap();assert!(CoreRegistry::set_double_handle(&p,key.into(),old+1.0));},
            4 => {let old=CoreRegistry::get_bool_handle(&p,key.into()).unwrap();assert!(CoreRegistry::set_bool_handle(&p,key.into(),!old));},
            t=>panic!("unexpected sizing field type {t}"),
        }
        assert_eq!(calls.get(), 1, "sizing key {key} must execute syncStyleChanges exactly once");
        if DIRTY.with(Cell::get)!=1 {missed.push((key,DIRTY.with(Cell::get)));}
    }
    assert!(missed.is_empty(),"sizing callbacks did not publish one layout-dirty transition: {missed:?}");
}

// A custom owner may forward public native property implementations while
// retaining its own opaque projection. That route must not acquire a new
// assumption that its outer CoreHandle downcasts to the embedded native owner.
struct OpaqueParticipant {
    inner: LayoutParticipant,
    style_calls: Option<std::rc::Rc<Cell<usize>>>,
}
impl nuxie_runtime::source::layout::layout_style_applier::LayoutStyleApplier for OpaqueParticipant {
    fn apply_base_style(&self, _: &mut nuxie_runtime::source::layout::layout_style_applier::YGStyle,
        _: &nuxie_runtime::source::layout::layout_style_applier::LayoutSyncContext) {
        if let Some(calls) = &self.style_calls { calls.set(calls.get() + 1); }
    }
}
impl CoreCapabilities for OpaqueParticipant {
    fn as_layout_style_applier(&self) -> Option<&dyn nuxie_runtime::source::layout::layout_style_applier::LayoutStyleApplier> { Some(self) }
}
impl CoreObject for OpaqueParticipant {
    fn core(&self)->&Core {self.inner.core()}
    fn core_mut(&mut self)->&mut Core {self.inner.core_mut()}
    fn core_type(&self)->u16{65530}
    fn is_type_of(&self,key:u16)->bool{key==65530}
    fn type_predicate(&self)->fn(u16)->bool{|key|key==65530}
    fn clone_boxed(&self)->Option<Box<dyn CoreObject>>{None}
    fn deserialize(&mut self,key:u16,reader:&mut BinaryReader<'_>)->bool{self.inner.deserialize(key,reader)}
}
impl CoreRegistryObject for OpaqueParticipant {
    fn as_registry_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        key == 65530
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
fn opaque_participant_setter_forwarding_does_not_require_a_native_projection(){
    let arena=CoreArena::default();let owner=arena.insert(OpaqueParticipant{inner:LayoutParticipant::default(),style_calls:None});
    assert!(CoreRegistry::set_uint_handle(&owner,Sizing::DISPLAY_VALUE_PROPERTY_KEY.into(),1));
    assert_eq!(CoreRegistry::get_uint_handle(&owner,Sizing::DISPLAY_VALUE_PROPERTY_KEY.into()),Some(1));
}
thread_local! { static FIRST_OBSERVER: std::cell::RefCell<Option<(CoreHandle,CoreHandle)>> = const { std::cell::RefCell::new(None) }; }
fn install_first_observer(_: *mut ()) {
    let pending=FIRST_OBSERVER.with(|slot|slot.borrow_mut().take());
    if let Some((owner,observer))=pending {
        owner.with_mut(|object| {
            observer.with_mut(|observer|object.core_mut().add_property_observer(observer.as_data_bind_mut().unwrap())).unwrap();
        }).unwrap();
    }
}
#[test]
fn participant_sizing_callback_releases_receiver_and_notifies_a_new_observer() {
    let (_file,p,host)=participant();
    let arena=CoreArena::default();
    let observer=arena.insert(nuxie_runtime::source::data_bind::data_bind::DataBind::new(0,NodeStyle::WIDTH_PROPERTY_KEY.into(),0));
    let root=host.with(|h|h.component_artboard_handle()).flatten().unwrap();
    root.with_downcast_mut::<Artboard,_>(|root| {
        for layout in root.find_all_handles::<LayoutComponent>() {root.clean_layout(&layout);}
        root.on_layout_dirty(Some(install_first_observer));
    }).unwrap();
    FIRST_OBSERVER.with(|slot|*slot.borrow_mut()=Some((p.clone(),observer.clone())));
    let old=CoreRegistry::get_double_handle(&p,NodeStyle::WIDTH_PROPERTY_KEY.into()).unwrap();
    assert!(CoreRegistry::set_double_handle(&p,NodeStyle::WIDTH_PROPERTY_KEY.into(),old+1.0));
    assert!(FIRST_OBSERVER.with(|slot|slot.borrow().is_none()));
    assert_eq!(observer.with(|o|o.as_data_bind().unwrap().dirt()),Some(nuxie_runtime::source::component_dirt::ComponentDirt::BINDINGS_TARGET.0.into()));
}
