use crate::records::small_vector::SmallVector;

impl<T: Clone, const N: usize> SmallVector<T, N> {
    pub fn small_vector_small_vector(&mut self, other: &SmallVector<T, N>) {
        self.small_vector();
        let other_count = other.size();
        self.reserve(other_count);

        for item in other.as_slice() {
            self.push_back(item.clone());
        }
    }
}
