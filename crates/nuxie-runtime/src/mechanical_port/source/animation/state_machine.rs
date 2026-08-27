use crate::mechanical_port::source::{
    animation::{
        state_machine_input::StateMachineInput, state_machine_layer::StateMachineLayer,
        state_machine_listener::StateMachineListener,
    },
    data_bind::data_bind::DataBind,
    generated::{animation::state_machine_base::StateMachineBase, artboard_base::ArtboardBase},
    importers::{artboard_importer::ArtboardImporter, import_stack::ImportStack},
    status_code::StatusCode,
};
use std::ptr::NonNull;
pub trait StateMachineOwnedObject {
    fn on_added_dirty(&mut self, context: *mut ()) -> StatusCode;
    fn on_added_clean(&mut self, context: *mut ()) -> StatusCode;
    fn name(&self) -> &str;
}
#[derive(Default)]
pub struct StateMachine {
    pub base: StateMachineBase,
    layers: Vec<Box<StateMachineLayer>>,
    inputs: Vec<Option<Box<StateMachineInput>>>,
    listeners: Vec<Box<StateMachineListener>>,
    data_binds: Vec<Box<DataBind>>,
    scripted_objects: Vec<*mut ()>,
}
impl StateMachine {
    pub fn on_added_dirty(&mut self, context: *mut ()) -> StatusCode {
        for object in self.inputs.iter_mut().filter_map(Option::as_deref_mut) {
            let code = object.on_added_dirty_raw(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for object in &mut self.layers {
            let code = object.on_added_dirty_raw(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for object in &mut self.listeners {
            let code = object.on_added_dirty_raw(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: *mut ()) -> StatusCode {
        for object in self.inputs.iter_mut().filter_map(Option::as_deref_mut) {
            let code = object.on_added_clean_raw(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for object in &mut self.layers {
            let code = object.on_added_clean_raw(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for object in &mut self.listeners {
            let code = object.on_added_clean_raw(context);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = stack.latest::<ArtboardImporter>(ArtboardBase::TYPE_KEY) else {
            return StatusCode::MissingObject;
        };
        importer.add_state_machine(NonNull::from(&mut *self));
        self.base.base.import(stack)
    }
    pub(crate) fn add_layer(&mut self, value: Box<StateMachineLayer>) {
        self.layers.push(value);
    }
    pub(crate) fn add_input(&mut self, value: Option<Box<StateMachineInput>>) {
        self.inputs.push(value);
    }
    pub(crate) fn add_listener(&mut self, value: Box<StateMachineListener>) {
        self.listeners.push(value);
    }
    pub(crate) fn add_data_bind(&mut self, value: Box<DataBind>) {
        self.data_binds.push(value);
    }
    pub(crate) fn add_scripted_object<T>(&mut self, value: NonNull<T>) {
        self.scripted_objects.push(value.as_ptr().cast());
    }
    pub fn layer_count(&self) -> usize {
        self.layers.len()
    }
    pub fn input_count(&self) -> usize {
        self.inputs.len()
    }
    pub fn listener_count(&self) -> usize {
        self.listeners.len()
    }
    pub fn data_bind_count(&self) -> usize {
        self.data_binds.len()
    }
    pub fn scripted_objects(&self) -> Vec<*mut ()> {
        self.scripted_objects.clone()
    }
    pub fn input(&self, index: usize) -> Option<&StateMachineInput> {
        self.inputs.get(index).and_then(Option::as_deref)
    }
    pub fn input_named(&self, name: &str) -> Option<&StateMachineInput> {
        self.inputs
            .iter()
            .filter_map(Option::as_deref)
            .find(|v| v.name() == name)
    }
    pub fn layer(&self, index: usize) -> Option<&StateMachineLayer> {
        self.layers.get(index).map(Box::as_ref)
    }
    pub fn layer_named(&self, name: &str) -> Option<&StateMachineLayer> {
        self.layers
            .iter()
            .map(Box::as_ref)
            .find(|v| v.name() == name)
    }
    pub fn listener(&self, index: usize) -> Option<&StateMachineListener> {
        self.listeners.get(index).map(Box::as_ref)
    }
    pub fn data_bind(&self, index: usize) -> Option<&DataBind> {
        self.data_binds.get(index).map(Box::as_ref)
    }
}
