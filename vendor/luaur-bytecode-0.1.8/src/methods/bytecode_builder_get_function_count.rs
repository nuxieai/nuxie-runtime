use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn get_function_count(&self) -> u32 {
        self.functions.len() as u32
    }
}
