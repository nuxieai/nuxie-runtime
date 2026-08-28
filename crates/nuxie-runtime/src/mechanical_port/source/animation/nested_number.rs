use crate::mechanical_port::source::generated::animation::nested_number_base::NestedNumberBase;
#[derive(Default)]
pub struct NestedNumber {
    pub base: NestedNumberBase,
}
impl NestedNumber {
    pub fn apply_value(&mut self) {
        self.base.base.set_number_value(self.base.nested_value());
    }
    pub fn set_nested_value(&mut self, value: f32) {
        if self.base.base.number_value() != Some(value) {
            self.base.base.set_number_value(value);
        }
    }
    pub fn nested_value(&self) -> f32 {
        self.base.base.number_value().unwrap_or(0.0)
    }
}
