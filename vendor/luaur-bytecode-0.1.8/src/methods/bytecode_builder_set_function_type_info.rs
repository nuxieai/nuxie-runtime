use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::string::String;

impl BytecodeBuilder {
    pub fn set_function_type_info(&mut self, value: String) {
        self.functions[self.current_function as usize].typeinfo = value;
    }
}
