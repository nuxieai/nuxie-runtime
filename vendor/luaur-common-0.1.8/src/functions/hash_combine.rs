#[allow(non_snake_case)]
#[inline]
pub fn hash_combine(seed: &mut usize, hash: usize) {
    // Golden Ratio constant used for better hash scattering
    // See https://softwareengineering.stackexchange.com/a/402543
    *seed ^= hash
        .wrapping_add(0x9e3779b9)
        .wrapping_add(*seed << 6)
        .wrapping_add(*seed >> 2);
}
