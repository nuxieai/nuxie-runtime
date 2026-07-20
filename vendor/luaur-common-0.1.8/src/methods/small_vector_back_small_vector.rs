use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    pub fn back_mut(&mut self) -> &mut T {
        LUAU_ASSERT!(self.size() > 0);
        let index = (self.size() as usize) - 1;
        &mut self.as_mut_slice()[index]
    }
}
