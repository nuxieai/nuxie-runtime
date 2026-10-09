//! Pinned ListenerInputChange::import must validate before registering an action.
use nuxie_runtime::source::{
    core::{CoreArena,Core,CoreObject,PropertySetterCompletion,binary_reader::BinaryReader,field_types::core_callback_type::CallbackData}, artboard::Artboard,
    animation::{state_machine::StateMachine,state_machine_listener::StateMachineListener},
    generated::{core_registry::{CoreRegistry,CoreRegistryObject,CoreCapabilities,CoreField},animation::{state_machine_listener_base::StateMachineListenerBase,state_machine_base::StateMachineBase,listener_input_change_base::ListenerInputChangeBase},artboard_base::ArtboardBase},
    importers::{import_stack::ImportStack,state_machine_listener_importer::StateMachineListenerImporter,state_machine_importer::StateMachineImporter,artboard_importer::ArtboardImporter},
    status_code::StatusCode,
};
fn run_with_nested(action_type:u16,input_type:Option<u16>,machine_present:bool,artboard_present:bool,nested_type:Option<u16>)->(Option<StatusCode>,usize){
    let arena=CoreArena::default();
    let listener=arena.insert(StateMachineListener::default());
    let action=arena.insert_boxed(CoreRegistry::make_core_box(action_type as i32).unwrap());
    assert!(CoreRegistry::set_id_handle(&action,ListenerInputChangeBase::INPUT_ID_PROPERTY_KEY.into(),0));
    let mut stack=ImportStack::default();
    assert_eq!(stack.make_latest(StateMachineListenerBase::TYPE_KEY,Some(Box::new(StateMachineListenerImporter::new(listener.clone())))),StatusCode::Ok);
    if machine_present {
        let machine=arena.insert(StateMachine::default());let mut importer=StateMachineImporter::new(machine);
        importer.add_input(input_type.map(|key| {
            if key>=65531 {arena.insert(ForeignInput{inner:Default::default(),advertises_boolean:key==65532})}
            else {arena.insert_boxed(CoreRegistry::make_core_box(key as i32).unwrap())}
        }));
        assert_eq!(stack.make_latest(StateMachineBase::TYPE_KEY,Some(Box::new(importer))),StatusCode::Ok);
    }
    if artboard_present {
        let artboard=arena.insert(Artboard::default());
        let mut importer=ArtboardImporter::new(artboard);
        if let Some(key)=nested_type {
            importer.add_component(Some(arena.insert_boxed(CoreRegistry::make_core_box(key.into()).unwrap())));
            assert!(CoreRegistry::set_id_handle(&action,ListenerInputChangeBase::NESTED_INPUT_ID_PROPERTY_KEY.into(),0));
        }
        assert_eq!(stack.make_latest(ArtboardBase::TYPE_KEY,Some(Box::new(importer))),StatusCode::Ok);
    }
    let result=action.with_mut(|o|o.import(&mut stack));
    let count=listener.with_downcast::<StateMachineListener,_>(|o|o.action_count()).unwrap();
    (result,count)
}
#[test]
fn native_input_actions_require_machine_and_artboard_before_registering(){
    let actual:[_;6]=std::array::from_fn(|i|run([117,118,115][i/2],None,i%2==1,false));
    assert_eq!(actual,[(Some(StatusCode::MissingObject),0);6]);
}
#[test]
fn native_input_actions_reject_wrong_input_types_before_registering(){
    let actual=[run(117,Some(56),true,true),run(118,Some(58),true,true),run(115,Some(59),true,true)];
    assert_eq!(actual,[(Some(StatusCode::InvalidObject),0);3]);
}
#[test]
fn native_input_actions_accept_matching_and_null_inputs(){
    for (action,input) in [(117,59),(118,56),(115,58)]{
        assert_eq!(run(action,Some(input),true,true),(Some(StatusCode::Ok),1));
        assert_eq!(run(action,None,true,true),(Some(StatusCode::Ok),1));
    }
}

fn run(action_type:u16,input_type:Option<u16>,machine_present:bool,artboard_present:bool)->(Option<StatusCode>,usize){run_with_nested(action_type,input_type,machine_present,artboard_present,None)}
#[test]
fn native_nested_subclass_validation_takes_precedence_over_machine_input(){
    let mut bad=Vec::new();
    for (action,matching,wrong,machine_matching,machine_wrong) in [(117,123,124,59,56),(118,124,122,56,58),(115,122,123,58,59)] {
        let valid=run_with_nested(action,Some(machine_wrong),true,true,Some(matching));
        let invalid=run_with_nested(action,Some(machine_matching),true,true,Some(wrong));
        if valid!=(Some(StatusCode::Ok),1)||invalid!=(Some(StatusCode::InvalidObject),0){bad.push((action,valid,invalid));}
    }
    assert!(bad.is_empty(),"nested validation results: {bad:?}");
}

struct ForeignInput {inner:nuxie_runtime::source::animation::state_machine_bool::StateMachineBool,advertises_boolean:bool}
impl CoreCapabilities for ForeignInput {}
fn boolean_input_predicate(key:u16)->bool{key==65532||nuxie_runtime::source::generated::animation::state_machine_bool_base::StateMachineBoolBase::is_type_of(key)}
fn unknown_input_predicate(key:u16)->bool{key==65531||nuxie_runtime::source::generated::animation::state_machine_input_base::StateMachineInputBase::is_type_of(key)}
impl CoreObject for ForeignInput {
    fn core(&self)->&Core{self.inner.core()}
    fn core_mut(&mut self)->&mut Core{self.inner.core_mut()}
    fn core_type(&self)->u16{if self.advertises_boolean {65532}else{65531}}
    fn is_type_of(&self,key:u16)->bool{self.type_predicate()(key)}
    fn type_predicate(&self)->fn(u16)->bool{if self.advertises_boolean {boolean_input_predicate}else{unknown_input_predicate}}
    fn clone_boxed(&self)->Option<Box<dyn CoreObject>>{None}
    fn deserialize(&mut self,key:u16,reader:&mut BinaryReader<'_>)->bool{self.inner.deserialize(key,reader)}
}
impl CoreRegistryObject for ForeignInput {
    fn as_registry_any(&self) -> &dyn std::any::Any {
        self
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
    fn is_type_of(&self, key: u16) -> bool {
        CoreObject::is_type_of(self, key)
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
fn native_listener_validators_respect_custom_input_predicates_and_unknown_inputs(){
    assert_eq!(run(117,Some(65532),true,true),(Some(StatusCode::Ok),1));
    assert_eq!(run(118,Some(65532),true,true),(Some(StatusCode::InvalidObject),0));
    assert_eq!(run(117,Some(65531),true,true),(Some(StatusCode::InvalidObject),0));
}
