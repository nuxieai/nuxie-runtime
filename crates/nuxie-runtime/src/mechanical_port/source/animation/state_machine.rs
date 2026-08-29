use crate::mechanical_port::source::{
    core::CoreHandle,
    core_context::CoreContext,
    generated::{animation::state_machine_base::StateMachineBase, artboard_base::ArtboardBase},
    importers::{artboard_importer::ArtboardImporter, import_stack::ImportStack},
    status_code::StatusCode,
};

#[derive(Default)]
pub struct StateMachine {
    pub base: StateMachineBase,
    layers: Vec<CoreHandle>,
    inputs: Vec<Option<CoreHandle>>,
    listeners: Vec<CoreHandle>,
    data_binds: Vec<CoreHandle>,
    scripted_objects: Vec<CoreHandle>,
}
impl StateMachine {
    pub fn set_name(&mut self, value: String) {
        use crate::mechanical_port::source::generated::animation::animation_base::{
            AnimationBase, AnimationBaseCallbacks,
        };
        if self.base.set_name_value(value) {
            AnimationBaseCallbacks::name_changed(self);
            AnimationBaseCallbacks::notify_property_changed(self, AnimationBase::NAME_PROPERTY_KEY);
        }
    }

    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for input in self.inputs.iter().filter_map(Clone::clone) {
            let code = input
                .with_mut(|input| input.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for layer in self.layers.iter().cloned() {
            let code = layer
                .with_downcast_mut::<
                    crate::mechanical_port::source::animation::state_machine_layer::StateMachineLayer,
                    _,
                >(|layer| layer.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for listener in self.listeners.iter().cloned() {
            let code = listener
                .with_mut(|listener| listener.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for input in self.inputs.iter().filter_map(Clone::clone) {
            let code = input
                .with_mut(|input| input.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for layer in self.layers.iter().cloned() {
            let code = layer
                .with_downcast_mut::<
                    crate::mechanical_port::source::animation::state_machine_layer::StateMachineLayer,
                    _,
                >(|layer| layer.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        for listener in self.listeners.iter().cloned() {
            let code = listener
                .with_mut(|listener| listener.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
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
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_state_machine(this);
        self.base.base.import(stack)
    }
    pub(crate) fn add_layer(&mut self, value: CoreHandle) {
        self.layers.push(value);
    }
    pub(crate) fn add_input(&mut self, value: Option<CoreHandle>) {
        self.inputs.push(value);
    }
    pub(crate) fn add_listener(&mut self, value: CoreHandle) {
        self.listeners.push(value);
    }
    pub(crate) fn add_data_bind(&mut self, value: CoreHandle) {
        self.data_binds.push(value);
    }
    pub(crate) fn add_scripted_object(&mut self, value: CoreHandle) {
        self.scripted_objects.push(value);
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
    pub fn scripted_objects(&self) -> Vec<CoreHandle> {
        self.scripted_objects.clone()
    }
    pub fn input(&self, index: usize) -> Option<CoreHandle> {
        self.inputs.get(index).and_then(Clone::clone)
    }
    pub fn input_named(&self, name: &str) -> Option<CoreHandle> {
        self.inputs.iter().filter_map(Clone::clone).find(|input| {
            input
                .with(|input| input.state_machine_input_name().as_deref() == Some(name))
                .unwrap_or(false)
        })
    }
    pub fn layer(&self, index: usize) -> Option<CoreHandle> {
        self.layers.get(index).cloned()
    }
    pub fn layer_named(&self, name: &str) -> Option<CoreHandle> {
        self.layers
            .iter()
            .find(|layer| {
                layer
                    .with(|layer| layer.state_machine_component_name().as_deref() == Some(name))
                    .unwrap_or(false)
            })
            .cloned()
    }
    pub fn listener(&self, index: usize) -> Option<CoreHandle> {
        self.listeners.get(index).cloned()
    }
    pub fn data_bind(&self, index: usize) -> Option<CoreHandle> {
        self.data_binds.get(index).cloned()
    }
}

impl std::ops::Deref for StateMachine {
    type Target = StateMachineBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for StateMachine {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
