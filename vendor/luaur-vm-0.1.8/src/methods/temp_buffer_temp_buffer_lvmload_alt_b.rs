use crate::records::temp_buffer::TempBuffer;

impl<T> TempBuffer<T> {
    /// C++ `TempBuffer(const TempBuffer&) = delete;`
    ///
    /// In Rust, we represent a deleted copy constructor by panicking if it is called,
    /// as the C++ compiler would have prevented this at compile time.
    pub fn temp_buffer_temp_buffer(&mut self, _other: &TempBuffer<T>) {
        panic!("TempBuffer copy constructor is deleted");
    }
}
