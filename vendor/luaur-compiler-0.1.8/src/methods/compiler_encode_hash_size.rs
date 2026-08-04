use crate::records::compiler::Compiler;

impl Compiler {
    pub fn encode_hash_size(hash_size: u32) -> u8 {
        if hash_size == 0 {
            return 0;
        }

        let mut hash_size_log2 = 0;
        while (1u32 << hash_size_log2) < hash_size {
            hash_size_log2 += 1;
        }

        (hash_size_log2 + 1) as u8
    }
}
