pub fn parallel_mul_sat(a: u64, b: i32) -> u64 {
    let bs = if b < 127 { b as u64 } else { 127u64 };

    // multiply every other value by b, yielding 14-bit products
    let l = bs.wrapping_mul(a & 0x007f007f007f007f);
    let h = bs.wrapping_mul((a >> 8) & 0x007f007f007f007f);

    // each product is 14-bit, so adding 32768-128 sets high bit iff the sum is 128 or larger without an overflow
    let ls = l.wrapping_add(0x7f807f807f807f80);
    let hs = h.wrapping_add(0x7f807f807f807f80);

    // we now merge saturation bits as well as low 7-bits of each product into one
    let s = (hs & 0x8000800080008000) | ((ls & 0x8000800080008000) >> 8);
    let r = ((h & 0x007f007f007f007f) << 8) | (l & 0x007f007f007f007f);

    // the low bits are now correct for values that didn't saturate, and we simply need to mask them if high bit is 1
    r | s.wrapping_sub(s >> 7)
}
