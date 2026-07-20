use crate::records::temp_buffer::TempBuffer;

impl<T> TempBuffer<T> {
    pub fn temp_buffer_temp_buffer_mut(&mut self, _other: TempBuffer<T>) {
        panic!("TempBuffer(TempBuffer&&) is deleted");
    }
}
