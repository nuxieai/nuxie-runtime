use crate::functions::parallel_add_sat::parallel_add_sat;
use crate::records::cost::Cost;

impl Cost {
    pub fn operator_add_assign(&mut self, other: &Cost) {
        self.model = parallel_add_sat(self.model, other.model);
        self.constant = 0;
    }
}
