use crate::mechanical_port::source::{
    animation::state_machine_input_instance::SMINumber,
    generated::animation::nested_number_base::NestedNumberBase,
};
pub trait NestedNumberInput {
    fn number_input(&self) -> Option<&SMINumber>;
    fn number_input_mut(&mut self) -> Option<&mut SMINumber>;
}
#[derive(Default)]
pub struct NestedNumber {
    pub base: NestedNumberBase,
}
impl NestedNumber {
    pub fn apply_value(&mut self, input: &mut dyn NestedNumberInput) {
        if let Some(value) = input.number_input_mut() {
            value.set_value(self.base.nested_value());
        }
    }
    pub fn set_nested_value(&mut self, value: f32, input: &mut dyn NestedNumberInput) {
        if let Some(number) = input.number_input_mut() {
            if number.value() != value {
                number.set_value(value);
            }
        }
    }
    pub fn nested_value(&self, input: &dyn NestedNumberInput) -> f32 {
        input.number_input().map(SMINumber::value).unwrap_or(0.0)
    }
}
