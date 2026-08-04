use crate::functions::parallel_mul_sat::parallel_mul_sat;
use crate::records::cost::Cost;

impl Cost {
    pub fn operator_mul(&self, other: i32) -> Cost {
        let mut result = Cost::default();
        result.model = parallel_mul_sat(self.model, other);
        result
    }
}

#[allow(dead_code)]
pub fn cost_operator_deref(self_: &Cost, other: i32) -> Cost {
    self_.operator_mul(other)
}
