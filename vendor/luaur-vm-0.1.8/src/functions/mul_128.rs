#[inline]
pub fn mul_128(x: u64, y: u64, hi: &mut u64) -> u64 {
    let r = (x as u128) * (y as u128);
    *hi = (r >> 64) as u64;
    r as u64
}
