use crate::mechanical_port::source::{
    animation::state_machine_input_instance::SMIInput, core_context::CoreContext,
    generated::animation::nested_input_base::NestedInputBase, status_code::StatusCode,
};

#[derive(Default)]
pub struct NestedInput {
    pub base: NestedInputBase,
}

impl NestedInput {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let result = self.base.on_added_dirty(context);
        if let Some(parent) = self.base.parent() {
            if let Some(state_machine) = context.nested_state_machine_mut(parent) {
                state_machine.add_nested_input(self);
            }
        }
        result
    }

    pub fn apply_value(&mut self) {}

    pub fn input<'a>(&self, context: &'a CoreContext) -> Option<&'a SMIInput> {
        let parent = self.base.parent()?;
        let nested = context.nested_state_machine(parent)?;
        nested.state_machine_instance()?.input(self.base.input_id())
    }

    pub fn name(&self, context: &CoreContext) -> String {
        self.input(context).map(SMIInput::name).unwrap_or_default()
    }
}
