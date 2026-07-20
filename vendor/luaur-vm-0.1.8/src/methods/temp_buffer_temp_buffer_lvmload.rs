use crate::records::temp_buffer::TempBuffer;

impl<T> TempBuffer<T> {
    pub fn temp_buffer() -> Self {
        let mut this = Self {
            L: core::ptr::null_mut(),
            data: core::ptr::null_mut(),
            count: 0,
        };
        this
    }
}
