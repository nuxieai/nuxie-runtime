#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Hasher;

impl
    luaur_common::records::dense_hash_table::DenseHasher<(
        *mut luaur_ast::records::ast_expr_table::AstExprTable,
        luaur_ast::records::ast_name::AstName,
    )> for Hasher
{
    fn hash(
        &self,
        key: &(
            *mut luaur_ast::records::ast_expr_table::AstExprTable,
            luaur_ast::records::ast_name::AstName,
        ),
    ) -> usize {
        use core::hash::{Hash, Hasher as _};

        struct FnvHasher(u64);

        impl core::hash::Hasher for FnvHasher {
            fn finish(&self) -> u64 {
                self.0
            }

            fn write(&mut self, bytes: &[u8]) {
                for &byte in bytes {
                    self.0 ^= byte as u64;
                    self.0 = self.0.wrapping_mul(0x0000_0100_0000_01b3);
                }
            }
        }

        let mut hasher = FnvHasher(0xcbf2_9ce4_8422_2325);
        key.hash(&mut hasher);
        hasher.finish() as usize
    }
}
