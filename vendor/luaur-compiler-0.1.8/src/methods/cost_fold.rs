use crate::functions::parallel_add_sat::parallel_add_sat;
use crate::records::cost::Cost;

impl Cost {
    pub(crate) fn fold_impl(x: &Cost, y: &Cost) -> Cost {
        let new_model = parallel_add_sat(x.model, y.model);
        let new_constant = x.constant & y.constant;

        let extra = if new_constant == Cost::kLiteral {
            0
        } else {
            1 | (0x0101010101010101u64 & new_constant)
        };

        Cost {
            model: parallel_add_sat(new_model, extra),
            constant: new_constant,
        }
    }
}

#[allow(non_snake_case)]
pub fn cost_fold(x: &Cost, y: &Cost) -> Cost {
    Cost::fold_impl(x, y)
}
