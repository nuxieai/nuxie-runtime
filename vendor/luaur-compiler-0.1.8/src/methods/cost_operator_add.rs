use crate::functions::parallel_add_sat::parallel_add_sat;
use crate::records::cost::Cost;

impl Cost {
    pub fn operator_add(&self, other: &Cost) -> Cost {
        // C++ operator+ default-constructs `Cost result` (constant = 0) and only sets
        // model; the constant mask is intentionally dropped here, unlike fold().
        let result_model = parallel_add_sat(self.model, other.model);
        Cost {
            model: result_model,
            constant: 0,
        }
    }
}

#[allow(non_snake_case)]
pub fn cost_operator_add(self_: &Cost, other: &Cost) -> Cost {
    self_.operator_add(other)
}
