use crate::records::bytecode_builder::BytecodeBuilder;

impl BytecodeBuilder {
    pub fn validate(&self) {
        self.validate_instructions();
        self.validate_variadic();
    }
}
