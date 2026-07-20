use crate::functions::mul_128::mul_128;

#[inline]
pub fn roundodd(ghi: u64, glo: u64, cp: u64) -> u64 {
    let mut xhi: u64 = 0;
    let _xlo = mul_128(glo, cp, &mut xhi);

    let mut yhi: u64 = 0;
    let ylo = mul_128(ghi, cp, &mut yhi);

    let z = ylo.wrapping_add(xhi);
    let carry = if z < xhi { 1 } else { 0 };
    let bit = if z > 1 { 1 } else { 0 };

    yhi.wrapping_add(carry) | bit
}
