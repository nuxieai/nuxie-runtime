#[allow(non_snake_case)]
#[inline(always)]
pub fn luai_num2long(i: &mut i64, d: f64) {
    *i = d as i64;
}
