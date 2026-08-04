use crate::functions::get_trip_count::get_trip_count;
use crate::records::cost::Cost;
use crate::records::cost_visitor::CostVisitor;

use luaur_ast::records::ast_stat_for::AstStatFor;

pub fn visit_ast_stat_for(this: &mut CostVisitor, node: *mut core::ffi::c_void) -> bool {
    unsafe {
        let stat_for: &mut AstStatFor = &mut *(node as *mut AstStatFor);

        // C++ `result += model(...)` uses Cost::operator+= (per-byte saturating add,
        // zeroes the constant mask). Raw u64 `+=` here both mis-tracked the constant
        // mask and overflowed once a multiplied loop cost approached u64::MAX.
        let c_from = this.model(stat_for.from);
        this.result.add_assign(&c_from);

        let c_to = this.model(stat_for.to);
        this.result.add_assign(&c_to);

        if !stat_for.step.is_null() {
            let c_step = this.model(stat_for.step);
            this.result.add_assign(&c_step);
        }

        let mut trip_count = -1;
        let mut from_val = 0.0;
        let mut to_val = 0.0;
        let mut step_val = 1.0;

        if this.get_number(stat_for.from, &mut from_val)
            && this.get_number(stat_for.to, &mut to_val)
            && (stat_for.step.is_null() || this.get_number(stat_for.step, &mut step_val))
        {
            trip_count = get_trip_count(from_val, to_val, step_val);
        }

        let factor = if trip_count < 0 { 3 } else { trip_count };
        this.loop_item(
            stat_for.body,
            Cost {
                model: 1,
                constant: 0,
            },
            factor,
        );
    }

    false
}
