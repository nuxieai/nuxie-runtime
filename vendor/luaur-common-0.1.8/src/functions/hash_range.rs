#[allow(non_snake_case)]
pub fn hashRange(data: *const i8, size: usize) -> usize {
    let mut hash: u32 = 2166136261;

    for i in 0..size {
        unsafe {
            let byte = *data.add(i) as u8;
            hash ^= byte as u32;
            hash = hash.wrapping_mul(16777619);
        }
    }

    hash as usize
}
