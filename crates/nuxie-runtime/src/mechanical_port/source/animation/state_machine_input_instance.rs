use crate::mechanical_port::source::animation::state_machine_instance::{
    RuntimeStateMachineInstanceWeakHandle, RuntimeStateMachineLayerInstanceWeakHandle,
};
use std::{cell::Cell, rc::Rc};
pub trait StateMachineInputDefinition {
    fn core_type(&self) -> u16;
    fn name(&self) -> &str;
    fn bool_value(&self) -> bool {
        false
    }
    fn number_value(&self) -> f32 {
        0.0
    }
}
#[derive(Clone)]
pub struct InputInstanceNotifier {
    needs_advance: Rc<Cell<bool>>,
    #[cfg(feature = "tools")]
    machine: Option<RuntimeStateMachineInstanceWeakHandle>,
}
impl InputInstanceNotifier {
    pub fn new(needs_advance: Rc<Cell<bool>>) -> Self {
        Self {
            needs_advance,
            #[cfg(feature = "tools")]
            machine: None,
        }
    }
    #[cfg(feature = "tools")]
    pub fn set_machine(&mut self, machine: RuntimeStateMachineInstanceWeakHandle) {
        self.machine = Some(machine);
    }
    fn value_changed(&self, index: u64) {
        self.needs_advance.set(true);
        #[cfg(feature = "tools")]
        if let Some(machine) = self.machine.as_ref() {
            machine.with_instance_mut(|machine| machine.input_changed(index));
        }
    }
}
pub struct SMIInput {
    notifier: InputInstanceNotifier,
    input_name: String,
    input_core_type: u16,
    #[cfg(feature = "tools")]
    index: u64,
}
impl SMIInput {
    pub fn new(input: &dyn StateMachineInputDefinition, notifier: InputInstanceNotifier) -> Self {
        Self {
            notifier,
            input_name: input.name().to_owned(),
            input_core_type: input.core_type(),
            #[cfg(feature = "tools")]
            index: 0,
        }
    }
    pub fn name(&self) -> &str {
        &self.input_name
    }
    pub fn input_core_type(&self) -> u16 {
        self.input_core_type
    }
    #[cfg(feature = "tools")]
    pub(crate) fn set_index(&mut self, index: u64) {
        self.index = index;
    }
    fn value_changed(&mut self) {
        #[cfg(not(feature = "tools"))]
        let index = 0;
        #[cfg(feature = "tools")]
        let index = self.index;
        self.notifier.value_changed(index);
    }
}
pub struct SMIBool {
    pub base: SMIInput,
    value: bool,
}
impl SMIBool {
    pub fn new(input: &dyn StateMachineInputDefinition, notifier: InputInstanceNotifier) -> Self {
        Self {
            value: input.bool_value(),
            base: SMIInput::new(input, notifier),
        }
    }
    pub fn value(&self) -> bool {
        self.value
    }
    pub fn set_value(&mut self, value: bool) {
        if self.value != value {
            self.value = value;
            self.base.value_changed();
        }
    }
}
pub struct SMINumber {
    pub base: SMIInput,
    value: f32,
}
impl SMINumber {
    pub fn new(input: &dyn StateMachineInputDefinition, notifier: InputInstanceNotifier) -> Self {
        Self {
            value: input.number_value(),
            base: SMIInput::new(input, notifier),
        }
    }
    pub fn value(&self) -> f32 {
        self.value
    }
    pub fn set_value(&mut self, value: f32) {
        if self.value != value {
            self.value = value;
            self.base.value_changed();
        }
    }
}
#[derive(Default)]
pub struct Triggerable {
    used_layers: Vec<RuntimeStateMachineLayerInstanceWeakHandle>,
}
impl Triggerable {
    pub fn is_used_in_layer(&self, layer: &RuntimeStateMachineLayerInstanceWeakHandle) -> bool {
        self.used_layers.iter().any(|used| used.ptr_eq(layer))
    }
    pub fn use_in_layer(&mut self, layer: RuntimeStateMachineLayerInstanceWeakHandle) {
        if !self.is_used_in_layer(&layer) {
            self.used_layers.push(layer);
        }
    }
}
pub struct SMITrigger {
    pub base: SMIInput,
    pub triggerable: Triggerable,
    fired: bool,
}
impl SMITrigger {
    pub fn new(input: &dyn StateMachineInputDefinition, notifier: InputInstanceNotifier) -> Self {
        Self {
            base: SMIInput::new(input, notifier),
            triggerable: Triggerable::default(),
            fired: false,
        }
    }
    pub fn fire(&mut self) {
        if !self.fired {
            self.fired = true;
            self.base.value_changed();
        }
    }
    pub fn advanced(&mut self) {
        self.fired = false;
        self.triggerable.used_layers.clear();
    }
    pub fn fired(&self) -> bool {
        self.fired
    }
    #[cfg(test)]
    pub fn did_fire(&self) -> bool {
        self.fired
    }
}
