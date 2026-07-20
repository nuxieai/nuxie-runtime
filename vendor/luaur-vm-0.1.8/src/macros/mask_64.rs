#[allow(non_snake_case)]
#[inline]
pub const fn mask64(w: i32) -> u64 {
    if w <= 0 {
        0
    } else if w >= 64 {
        0xFFFFFFFFFFFFFFFFu64
    } else {
        0xFFFFFFFFFFFFFFFFu64 >> (64 - w)
    }
}
