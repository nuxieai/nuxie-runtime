use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn emit_aux(&mut self, aux: u32) {
        self.insns.push(aux);
        self.lines.push(self.debug_line);
    }
}
