pub trait BytecodeEncoder {
    fn encode(&mut self, data: &mut [u32]);
}
