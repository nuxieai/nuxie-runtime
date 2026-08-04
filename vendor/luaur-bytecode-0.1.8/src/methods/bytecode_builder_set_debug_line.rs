use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn set_debug_line(&mut self, line: i32) {
        self.debug_line = line;
    }
}
