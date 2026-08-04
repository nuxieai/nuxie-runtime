use crate::records::bc_inst_hash::BcInstHash;

impl BcInstHash {
    pub fn mix_u32_u32(mut h: u32, mut k: u32) -> u32 {
        // MurmurHash2 step
        k = k.wrapping_mul(Self::M);
        k ^= k >> Self::R;
        k = k.wrapping_mul(Self::M);

        h = h.wrapping_mul(Self::M);
        h ^= k;

        h
    }
}
