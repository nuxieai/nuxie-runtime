use crate::records::temp_vector::TempVector;

impl<'a, T> TempVector<'a, T> {
    #[allow(non_snake_case)]
    pub fn new(storage: &mut alloc::vec::Vec<T>) -> Self {
        let offset = storage.len();
        Self {
            storage: storage as *mut _,
            offset,
            size_: 0,
            _marker: core::marker::PhantomData,
        }
    }
}

#[allow(non_snake_case)]
pub fn temp_vector_t_temp_vector<'a, T>(storage: &mut alloc::vec::Vec<T>) -> TempVector<'a, T> {
    TempVector::new(storage)
}
