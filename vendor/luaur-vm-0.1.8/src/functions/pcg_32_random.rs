use crate::macros::pcg_32_inc::PCG32_INC;

#[inline]
pub fn pcg_32_random(state: &mut u64) -> u32 {
    let oldstate = *state;
    *state = oldstate.wrapping_mul(6364136223846793005) + (PCG32_INC | 1);
    let xorshifted = (((oldstate >> 18) ^ oldstate) >> 27) as u32;
    let rot = (oldstate >> 59) as u32;
    let shift = (0u32.wrapping_sub(rot)) & 31;
    (xorshifted >> rot) | (xorshifted << shift)
}
