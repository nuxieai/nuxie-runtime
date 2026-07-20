use crate::functions::mul_128::mul_128;

#[inline]
pub fn mul_192_hi(xhi: u64, xlo: u64, y: u64, hi: &mut u64) -> u64 {
    let mut z2: u64 = 0;
    let mut z1 = mul_128(xhi, y, &mut z2);

    let mut z1c: u64 = 0;
    let _z0 = mul_128(xlo, y, &mut z1c);

    z1 = z1.wrapping_add(z1c);
    z2 = z2.wrapping_add(if z1 < z1c { 1 } else { 0 });

    *hi = z2;
    z1
}
