use crate::records::bc_ref::BcRef;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a, T> BcRef<'a, T> {
    pub fn operator_deref(&self) -> &T {
        LUAU_ASSERT!((self.op.index as usize) < self.vec.len());
        &self.vec[self.op.index as usize]
    }
}
