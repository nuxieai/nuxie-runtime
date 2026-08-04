use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::debug_upval::DebugUpval;
use crate::records::string_ref::StringRef;

impl BytecodeBuilder {
    pub fn push_debug_upval(&mut self, name: StringRef) {
        let index = self.add_string_table_entry(name);

        let upval = DebugUpval { name: index };

        self.debug_upvals.push(upval);
    }
}
