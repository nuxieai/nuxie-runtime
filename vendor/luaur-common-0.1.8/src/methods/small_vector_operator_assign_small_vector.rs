use crate::records::small_vector::SmallVector;
use core::ptr;

impl<T: Clone, const N: usize> SmallVector<T, N> {
    pub fn operator_assign(&mut self, other: &SmallVector<T, N>) {
        if ptr::eq(self, other) {
            return;
        }

        let other_size = other.size();
        let current_size = self.size();

        if other_size <= current_size {
            {
                let other_slice = other.as_slice();
                let self_slice = self.as_mut_slice();
                for i in 0..other_size as usize {
                    self_slice[i] = other_slice[i].clone();
                }
            }

            while self.size() > other_size {
                self.pop_back();
            }
        } else {
            {
                let other_slice = other.as_slice();
                let self_slice = self.as_mut_slice();
                for i in 0..current_size as usize {
                    self_slice[i] = other_slice[i].clone();
                }
            }

            self.reserve(other_size);

            // After reserve, we must re-acquire pointers as the heap block might have moved.
            // We use emplace_back to update the count and handle initialization safely
            // for the remaining elements, mirroring std::uninitialized_copy.
            let other_slice = other.as_slice();
            for i in current_size as usize..other_size as usize {
                self.emplace_back(other_slice[i].clone());
            }
        }
    }
}
