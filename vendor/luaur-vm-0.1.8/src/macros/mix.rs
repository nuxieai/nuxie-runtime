use crate::macros::rol::rol;

#[allow(non_snake_case)]
#[inline(always)]
pub fn mix(u: u32, v: u32, w: u32, a: &mut u32, b: &mut u32, h: &mut u32) {
    *a ^= *h;
    *a = a.wrapping_sub(rol(*h, u));
    *b ^= *a;
    *b = b.wrapping_sub(rol(*a, v));
    *h ^= *b;
    *h = h.wrapping_sub(rol(*b, w));
}
