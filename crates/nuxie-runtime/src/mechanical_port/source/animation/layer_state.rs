use crate::mechanical_port::source::{
    animation::{state_instance::StateInstance, system_state_instance::SystemStateInstance},
    artboard::RuntimeArtboardInstanceWeakHandle,
    core::CoreHandle,
    core_context::CoreContext,
    generated::animation::{
        layer_state_base::LayerStateBase, state_machine_layer_base::StateMachineLayerBase,
    },
    importers::{
        import_stack::ImportStack, state_machine_layer_importer::StateMachineLayerImporter,
    },
    status_code::StatusCode,
};

#[derive(Default)]
pub struct LayerState {
    pub base: LayerStateBase,
    transitions: Vec<CoreHandle>,
    transitions_settle_safe: bool,
    transitions_ignore_time: bool,
}
impl LayerState {
    pub fn transitions_settle_safe(&self) -> bool {
        self.transitions_settle_safe
    }
    pub fn transitions_ignore_time(&self) -> bool {
        self.transitions_ignore_time
    }
    pub fn set_transitions_settle_safe(&mut self, value: bool) {
        self.transitions_settle_safe = value;
    }
    pub fn set_transitions_ignore_time(&mut self, value: bool) {
        self.transitions_ignore_time = value;
    }
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
    pub fn transition(&self, index: usize) -> Option<CoreHandle> {
        self.transitions.get(index).cloned()
    }
    pub fn transitions(&self) -> &[CoreHandle] {
        &self.transitions
    }
    pub(crate) fn add_transition(&mut self, transition: CoreHandle) {
        self.transitions.push(transition);
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for transition in self.transitions.iter().cloned() {
            let code = transition
                .with_mut(|transition| transition.on_added_dirty(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        for transition in self.transitions.iter().cloned() {
            let code = transition
                .with_mut(|transition| transition.on_added_clean(context))
                .unwrap_or(StatusCode::MissingObject);
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(importer) =
            stack.latest::<StateMachineLayerImporter>(StateMachineLayerBase::TYPE_KEY)
        else {
            return StatusCode::MissingObject;
        };
        let Some(this) = self.base.base.base.base.handle() else {
            return StatusCode::MissingObject;
        };
        importer.add_state(this);
        self.base.base.import(stack)
    }
    pub fn make_instance(
        &self,
        _artboard: RuntimeArtboardInstanceWeakHandle,
    ) -> Box<SystemStateInstance> {
        let state = self
            .base
            .base
            .base
            .base
            .handle()
            .expect("an imported LayerState must have arena identity before instancing");
        Box::new(SystemStateInstance::new(state))
    }
}
impl std::ops::Deref for LayerState {
    type Target = LayerStateBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for LayerState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
impl crate::mechanical_port::source::generated::animation::layer_state_base::LayerStateBaseCallbacks
    for LayerState
{
    fn notify_property_changed(&mut self, key: u16) {
        self.base.notify_property_changed(key);
    }
}
