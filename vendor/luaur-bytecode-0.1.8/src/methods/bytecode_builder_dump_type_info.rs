use crate::functions::get_base_type_string::get_base_type_string;
use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::string::String;
use core::ffi::CStr;
use luaur_common::enums::luau_bytecode_type::LBC_TYPE_FUNCTION;
use luaur_common::functions::format_append::formatAppend;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn dump_type_info(&self) -> String {
        let mut result = String::new();

        const LBC_TYPE_OPTIONAL_BIT: u8 = 0x80;

        for (i, function) in self.functions.iter().enumerate() {
            let typeinfo = function.typeinfo.as_bytes();
            if typeinfo.is_empty() {
                continue;
            }

            let encoded_type = typeinfo[0];

            LUAU_ASSERT!(encoded_type == LBC_TYPE_FUNCTION.0 as u8);

            formatAppend(&mut result, format_args!("{}: function(", i));

            LUAU_ASSERT!(typeinfo.len() >= 2);

            let numparams = typeinfo[1];

            LUAU_ASSERT!((1 + numparams as usize - 1) < typeinfo.len());

            for j in 0..numparams {
                let et = typeinfo[2 + j as usize];
                let optional = if (et & LBC_TYPE_OPTIONAL_BIT) != 0 {
                    "?"
                } else {
                    ""
                };

                let base_type_str =
                    unsafe { CStr::from_ptr(get_base_type_string(et)).to_string_lossy() };
                formatAppend(&mut result, format_args!("{}{}", base_type_str, optional));

                if j + 1 != numparams {
                    formatAppend(&mut result, format_args!(", "));
                }
            }

            formatAppend(&mut result, format_args!(")\n"));
        }

        result
    }
}
