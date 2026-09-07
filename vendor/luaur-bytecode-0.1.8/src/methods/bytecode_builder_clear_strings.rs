use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn clear_strings(&mut self) {
        self.debug_strings.clear();
        self.string_table.clear();
    }
}
