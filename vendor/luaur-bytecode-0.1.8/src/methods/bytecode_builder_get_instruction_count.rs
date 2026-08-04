use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn get_instruction_count(&self) -> usize {
        self.insns.len()
    }
}
