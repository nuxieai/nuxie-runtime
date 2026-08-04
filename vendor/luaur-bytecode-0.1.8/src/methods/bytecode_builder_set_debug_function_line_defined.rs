use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn set_debug_function_line_defined(&mut self, line: i32) {
        self.functions[self.current_function as usize].debuglinedefined = line;
    }
}
