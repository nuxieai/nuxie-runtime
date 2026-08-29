pub trait BinaryStream {
    fn write(&mut self, bytes: &[u8]);
    fn flush(&mut self);
    fn clear(&mut self);
}
