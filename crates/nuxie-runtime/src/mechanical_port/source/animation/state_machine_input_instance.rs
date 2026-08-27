use std::collections::HashSet;
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
pub trait InputInstanceMachine {
    fn mark_needs_advance(&mut self);
    #[cfg(feature = "rive_tools")]
    fn input_changed(&mut self, index: u64);
}
pub struct SMIInput {
    machine: *mut dyn InputInstanceMachine,
    input: *const dyn StateMachineInputDefinition,
    #[cfg(feature = "rive_tools")]
    index: u64,
}
impl SMIInput {
    pub fn new(
        input: &dyn StateMachineInputDefinition,
        machine: &mut dyn InputInstanceMachine,
    ) -> Self {
        Self {
            machine,
            input,
            #[cfg(feature = "rive_tools")]
            index: 0,
        }
    }
    pub fn input(&self) -> &dyn StateMachineInputDefinition {
        unsafe { &*self.input }
    }
    pub fn name(&self) -> &str {
        self.input().name()
    }
    pub fn input_core_type(&self) -> u16 {
        self.input().core_type()
    }
    #[cfg(feature = "rive_tools")]
    pub(crate) fn set_index(&mut self, index: u64) {
        self.index = index;
    }
    fn value_changed(&mut self) {
        unsafe {
            (&mut *self.machine).mark_needs_advance();
            #[cfg(feature = "rive_tools")]
            (&mut *self.machine).input_changed(self.index);
        }
    }
}
pub struct SMIBool {
    pub base: SMIInput,
    value: bool,
}
impl SMIBool {
    pub fn new(
        input: &dyn StateMachineInputDefinition,
        machine: &mut dyn InputInstanceMachine,
    ) -> Self {
        Self {
            value: input.bool_value(),
            base: SMIInput::new(input, machine),
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
    pub fn new(
        input: &dyn StateMachineInputDefinition,
        machine: &mut dyn InputInstanceMachine,
    ) -> Self {
        Self {
            value: input.number_value(),
            base: SMIInput::new(input, machine),
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
    used_layers: HashSet<usize>,
}
impl Triggerable {
    pub fn is_used_in_layer(&self, layer: *mut ()) -> bool {
        self.used_layers.contains(&(layer as usize))
    }
    pub fn use_in_layer(&mut self, layer: *mut ()) {
        self.used_layers.insert(layer as usize);
    }
}
pub struct SMITrigger {
    pub base: SMIInput,
    pub triggerable: Triggerable,
    fired: bool,
}
impl SMITrigger {
    pub fn new(
        input: &dyn StateMachineInputDefinition,
        machine: &mut dyn InputInstanceMachine,
    ) -> Self {
        Self {
            base: SMIInput::new(input, machine),
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
    #[cfg(feature = "testing")]
    pub fn did_fire(&self) -> bool {
        self.fired
    }
}
