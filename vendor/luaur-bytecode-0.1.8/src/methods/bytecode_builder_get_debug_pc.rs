use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn get_debug_pc(&self) -> u32 {
        self.insns.len() as u32
    }
}
