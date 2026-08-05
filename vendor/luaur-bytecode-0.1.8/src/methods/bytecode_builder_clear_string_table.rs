use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn clear_string_table(&mut self) {
        self.string_table.clear();
    }
}
