use crate::records::constant_key::ConstantKey;
use crate::records::constant_key_hash::ConstantKeyHash;

#[allow(non_snake_case)]
impl ConstantKeyHash {
    pub fn call(&self, key: &ConstantKey) -> usize {
        // Constant::Type_Vector is 4 based on Luau bytecode specification
        if key.r#type as u8 == 4 {
            let mut i = [0u32; 4];
            // Safety: ConstantKey.value and ConstantKey.extra are both u64, totaling 16 bytes.
            // [u32; 4] is also 16 bytes.
            unsafe {
                let src_ptr = &key.value as *const u64 as *const u8;
                core::ptr::copy_nonoverlapping(src_ptr, i.as_mut_ptr() as *mut u8, 16);
            }

            // scramble bits to make sure that integer coordinates have entropy in lower bits
            i[0] ^= i[0] >> 17;
            i[1] ^= i[1] >> 17;
            i[2] ^= i[2] >> 17;
            i[3] ^= i[3] >> 17;

            // Optimized Spatial Hashing for Collision Detection of Deformable Objects
            let h = (i[0].wrapping_mul(73856093))
                ^ (i[1].wrapping_mul(19349663))
                ^ (i[2].wrapping_mul(83492791))
                ^ (i[3].wrapping_mul(39916801));

            h as usize
        } else {
            // finalizer from MurmurHash64B
            const M: u32 = 0x5bd1e995;

            let mut h1 = key.value as u32;
            let mut h2 = (key.value >> 32) as u32 ^ ((key.r#type as u32).wrapping_mul(M));

            h1 ^= h2 >> 18;
            h1 = h1.wrapping_mul(M);
            h2 ^= h1 >> 22;
            h2 = h2.wrapping_mul(M);
            h1 ^= h2 >> 17;
            h1 = h1.wrapping_mul(M);
            h2 ^= h1 >> 19;
            h2 = h2.wrapping_mul(M);

            // ... truncated to 32-bit output
            h2 as usize
        }
    }
}
