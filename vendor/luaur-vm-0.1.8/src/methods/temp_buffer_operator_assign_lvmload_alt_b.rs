use crate::records::temp_buffer::TempBuffer;

impl<T> TempBuffer<T> {
    /// C++ `TempBuffer& operator=(TempBuffer&&) = delete;`
    ///
    /// In Rust, we represent a deleted move assignment operator by not providing
    /// a public move assignment method for `TempBuffer`.
    #[allow(dead_code)]
    pub fn operator_assign_mut(&mut self, _other: TempBuffer<T>) {
        panic!("TempBuffer move assignment operator is deleted");
    }
}
