#[allow(non_snake_case)]
#[inline(always)]
pub fn luai_numisnan(a: f64) -> bool {
    a != a
}
