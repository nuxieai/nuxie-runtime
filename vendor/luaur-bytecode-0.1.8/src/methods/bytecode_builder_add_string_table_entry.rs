use crate::enums::dump_flags::DumpFlags;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::string_ref::StringRef;

impl BytecodeBuilder {
    pub fn add_string_table_entry(&mut self, value: StringRef) -> u32 {
        if let Some(idx) = self.string_table.find(&value) {
            return *idx;
        }

        // Bytecode serialization uses 1-based string-table indices (0 is reserved
        // to mean "no string"). C++ computes the index as `stringTable.size()`
        // *after* inserting the new entry via `operator[]`, i.e. pre-insert size+1.
        // Computing it pre-insert made the first string index 0, so
        // `debugStrings[valueString - 1]` underflowed in `dumpConstant`.
        let new_index = self.string_table.size() as u32 + 1;
        self.string_table.try_insert(value, new_index);

        if (self.dump_flags & DumpFlags::Dump_Code as u32) != 0 {
            self.debug_strings.push(value);
        }

        new_index
    }
}
