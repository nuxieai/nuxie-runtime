use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn emit_label(&self) -> usize {
        self.insns.len()
    }
}
