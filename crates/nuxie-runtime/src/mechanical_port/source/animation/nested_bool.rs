use crate::mechanical_port::source::{
    animation::state_machine_input_instance::SMIBool,
    generated::animation::nested_bool_base::NestedBoolBase,
};
pub trait NestedBoolInput {
    fn bool_input(&self) -> Option<&SMIBool>;
    fn bool_input_mut(&mut self) -> Option<&mut SMIBool>;
}
#[derive(Default)]
pub struct NestedBool {
    pub base: NestedBoolBase,
}
impl NestedBool {
    pub fn apply_value(&mut self, input: &mut dyn NestedBoolInput) {
        if let Some(value) = input.bool_input_mut() {
            value.set_value(self.base.nested_value());
        }
    }
    pub fn set_nested_value(&mut self, value: bool, input: &mut dyn NestedBoolInput) {
        if let Some(boolean) = input.bool_input_mut() {
            if boolean.value() != value {
                boolean.set_value(value);
            }
        }
    }
    pub fn nested_value(&self, input: &dyn NestedBoolInput) -> bool {
        input.bool_input().map(SMIBool::value).unwrap_or(false)
    }
}
