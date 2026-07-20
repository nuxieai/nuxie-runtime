use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::small_vector::SmallVector;

impl<T, const N: usize> SmallVector<T, N> {
    #[allow(non_snake_case)]
    pub fn operator_index_mut(&mut self, index: usize) -> &mut T {
        LUAU_ASSERT!(index < self.size() as usize);

        &mut self.as_mut_slice()[index]
    }
}
