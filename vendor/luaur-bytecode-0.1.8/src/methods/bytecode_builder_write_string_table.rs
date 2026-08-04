use crate::functions::write_var_int::writeVarInt;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::string_ref::StringRef;
use alloc::string::String;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub(crate) fn write_string_table(&self, ss: &mut String) {
        let count = self.string_table.size();
        let mut strings: Vec<StringRef> = Vec::with_capacity(count);
        unsafe {
            strings.set_len(count);
        }

        for (string_ref, &index) in self.string_table.iter() {
            LUAU_ASSERT!(index > 0 && (index as usize) <= strings.len());
            strings[index as usize - 1] = *string_ref;
        }

        writeVarInt(ss, strings.len() as u64);

        for s in strings {
            writeVarInt(ss, s.length as u64);
            let data = unsafe { core::slice::from_raw_parts(s.data as *const u8, s.length) };
            // Safety: ss is an alloc::string::String, which is a wrapper around Vec<u8> that guarantees UTF-8.
            // However, Luau bytecode strings are raw byte buffers. In the Rust port, BytecodeBuilder::bytecode
            // and the ss parameter are Strings, but they are treated as byte buffers (binary data).
            unsafe {
                ss.as_mut_vec().extend_from_slice(data);
            }
        }
    }
}
