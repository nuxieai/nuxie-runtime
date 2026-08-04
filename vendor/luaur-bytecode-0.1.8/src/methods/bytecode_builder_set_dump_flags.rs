use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn set_dump_flags(&mut self, flags: u32) {
        self.dump_flags = flags;
        self.dump_function_ptr = Some(BytecodeBuilder::dump_current_function);
    }
}
