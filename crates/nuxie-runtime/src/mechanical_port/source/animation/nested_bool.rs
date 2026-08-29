use crate::mechanical_port::source::generated::animation::nested_bool_base::NestedBoolBase;
#[derive(Default)]
pub struct NestedBool {
    pub base: NestedBoolBase,
}
impl NestedBool {
    pub fn apply_value(&mut self) {
        self.base.base.set_bool_value(self.base.nested_value());
    }
    pub fn set_nested_value(&mut self, value: bool) {
        if self.base.base.bool_value() != Some(value) {
            self.base.base.set_bool_value(value);
        }
    }
    pub fn nested_value(&self) -> bool {
        self.base.base.bool_value().unwrap_or(false)
    }
}
