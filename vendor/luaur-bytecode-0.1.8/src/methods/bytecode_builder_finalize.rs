//! Node: `cxx:Method:Luau.Bytecode:Bytecode/src/BytecodeBuilder.cpp:676:finalize`
//!
//! Faithful port of `BytecodeBuilder::finalize`: assemble the final bytecode
//! blob — version byte, type-encoding version, string table, userdata type-name
//! mapping, then every function's pre-serialized `data` blob, then the main
//! function index. `bytecode` is taken out of `self` while writing so the
//! `&self` helpers (`write_string_table`) and `&mut self` field reads don't
//! alias the buffer being filled.

use crate::functions::write_byte::write_byte;
use crate::functions::write_var_int::write_var_int;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::string_ref::StringRef;
use luaur_common::enums::luau_bytecode_tag::{
    LBC_TYPE_VERSION_MAX, LBC_TYPE_VERSION_MIN, LBC_VERSION_MAX, LBC_VERSION_MIN,
};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn finalize(&mut self) {
        LUAU_ASSERT!(self.bytecode.is_empty());

        for i in 0..self.userdata_types.len() {
            if self.userdata_types[i].used {
                let sref = {
                    let name = &self.userdata_types[i].name;
                    StringRef {
                        data: name.as_ptr() as *const i8,
                        length: name.len(),
                    }
                };
                self.userdata_types[i].nameRef = self.add_string_table_entry(sref);
            }
        }

        // preallocate space for bytecode blob
        let mut capacity: usize = 16;

        for (string_ref, _index) in self.string_table.iter() {
            capacity += string_ref.length + 2;
        }

        for func in &self.functions {
            capacity += func.data.len();
        }

        // assemble final bytecode blob — taken out of `self` so the buffer is
        // not aliased by the `&self`/field reads below.
        let mut bytecode = core::mem::take(&mut self.bytecode);
        bytecode.reserve(capacity);

        let version = self.get_version();
        LUAU_ASSERT!(version >= LBC_VERSION_MIN.0 as u8 && version <= LBC_VERSION_MAX.0 as u8);

        unsafe {
            bytecode.as_mut_vec().push(version);
        }

        let typesversion = self.get_type_encoding_version();
        LUAU_ASSERT!(
            typesversion >= LBC_TYPE_VERSION_MIN.0 as u8
                && typesversion <= LBC_TYPE_VERSION_MAX.0 as u8
        );
        write_byte(&mut bytecode, typesversion);

        self.write_string_table(&mut bytecode);

        // Write the mapping between used type name indices and their name
        for i in 0..self.userdata_types.len() {
            if self.userdata_types[i].used {
                write_byte(&mut bytecode, (i + 1) as u8);
                write_var_int(&mut bytecode, self.userdata_types[i].nameRef as u64);
            }
        }

        // 0 marks the end of the mapping
        write_byte(&mut bytecode, 0);

        write_var_int(&mut bytecode, self.functions.len() as u64);

        for func in &self.functions {
            unsafe {
                bytecode
                    .as_mut_vec()
                    .extend_from_slice(func.data.as_bytes());
            }
        }

        LUAU_ASSERT!((self.main_function as usize) < self.functions.len());
        write_var_int(&mut bytecode, self.main_function as u64);

        self.bytecode = bytecode;
    }
}
