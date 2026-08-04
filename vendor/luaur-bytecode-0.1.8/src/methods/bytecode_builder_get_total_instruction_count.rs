use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn get_total_instruction_count(&self) -> usize {
        self.total_instruction_count
    }
}
