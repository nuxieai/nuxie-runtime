use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn get_string_table(&self) -> Vec<&str> {
        let table_len = self.string_table.size();
        let mut strings: Vec<&str> = Vec::with_capacity(table_len);
        unsafe {
            strings.set_len(table_len);
        }

        for (string_ref, &index) in self.string_table.iter() {
            LUAU_ASSERT!(index > 0 && (index as usize) <= strings.len());
            let data = unsafe {
                core::slice::from_raw_parts(string_ref.data as *const u8, string_ref.length)
            };
            strings[index as usize - 1] = core::str::from_utf8(data).unwrap_or("");
        }
        strings
    }
}
