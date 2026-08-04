pub fn compute_cost(model: u64, vars_const: *const bool, var_count: usize) -> i32 {
    let mut cost = (model & 0x7f) as i32;

    // don't apply discounts to what is likely a saturated sum
    if cost == 0x7f {
        return cost;
    }

    // C++ indexes `varsConst[i]` directly; `slice::from_raw_parts(null, 0)` is UB
    // in Rust, so index the raw pointer directly to match C++ (vars_const may be
    // null when var_count == 0).
    let mut i = 0;
    while i < var_count && i < 7 {
        let discount = ((model >> (i * 8 + 8)) & 0x7f) as i32;
        let is_const = unsafe { *vars_const.add(i) };
        cost -= discount * (is_const as i32);
        i += 1;
    }

    cost
}
