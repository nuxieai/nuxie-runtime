use crate::enums::r#type::Type;
use crate::functions::ceillog_2::ceillog_2;
use crate::functions::printable_string_constant::printableStringConstant;
use crate::methods::bytecode_builder_get_string_hash::bytecode_builder_get_string_hash;
use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::string::String;
use core::cmp;
use luaur_common::functions::format_append::formatAppend;
use luaur_common::functions::format_g::format_g;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl BytecodeBuilder {
    pub fn dump_constant(&self, result: &mut String, k: i32, detailed: bool) {
        LUAU_ASSERT!((k as u32) < self.constants.len() as u32);
        let data = &self.constants[k as usize];

        match data.r#type {
            Type::Type_Nil => formatAppend(result, format_args!("nil")),
            Type::Type_Boolean => formatAppend(
                result,
                format_args!(
                    "{}",
                    if unsafe { data.value.valueBoolean } {
                        "true"
                    } else {
                        "false"
                    }
                ),
            ),
            Type::Type_Number => formatAppend(
                result,
                format_args!("{}", format_g(unsafe { data.value.valueNumber }, 17)),
            ),
            Type::Type_Integer => formatAppend(
                result,
                format_args!("{}", unsafe { data.value.valueInteger64 } as i64),
            ),
            Type::Type_Vector => {
                let v = unsafe { data.value.valueVector };
                if v[3] == 0.0 {
                    formatAppend(
                        result,
                        format_args!(
                            "{}, {}, {}",
                            format_g(v[0] as f64, 9),
                            format_g(v[1] as f64, 9),
                            format_g(v[2] as f64, 9)
                        ),
                    );
                } else {
                    formatAppend(
                        result,
                        format_args!(
                            "{}, {}, {}, {}",
                            format_g(v[0] as f64, 9),
                            format_g(v[1] as f64, 9),
                            format_g(v[2] as f64, 9),
                            format_g(v[3] as f64, 9)
                        ),
                    );
                }
            }
            Type::Type_String => {
                let str_idx = unsafe { data.value.valueString };
                let str = &self.debug_strings[str_idx as usize - 1];
                let bytes =
                    unsafe { core::slice::from_raw_parts(str.data as *const u8, str.length) };
                if printableStringConstant(bytes) {
                    if str.length < 32 {
                        formatAppend(
                            result,
                            format_args!("'{:.*}'", str.length, unsafe {
                                core::str::from_utf8_unchecked(bytes)
                            }),
                        );
                    } else {
                        formatAppend(
                            result,
                            format_args!("'{:.*}'...", 32, unsafe {
                                core::str::from_utf8_unchecked(bytes)
                            }),
                        );
                    }
                } else {
                    formatAppend(result, format_args!("'"));
                    for i in 0..cmp::min(str.length, 32) {
                        let b = bytes[i];
                        if b < b' ' {
                            formatAppend(result, format_args!("\\x{:02X}", b));
                        } else {
                            formatAppend(result, format_args!("{}", b as char));
                        }
                    }
                    if str.length >= 32 {
                        formatAppend(result, format_args!("'..."));
                    } else {
                        formatAppend(result, format_args!("'"));
                    }
                }
            }
            Type::Type_Import => {
                let mut id0: i32 = -1;
                let mut id1: i32 = -1;
                let mut id2: i32 = -1;
                let count = BytecodeBuilder::decompose_import_id(
                    unsafe { data.value.valueImport },
                    &mut id0,
                    &mut id1,
                    &mut id2,
                );
                if count > 0 {
                    let id = self.constants[id0 as usize];
                    LUAU_ASSERT!(
                        id.r#type == Type::Type_String
                            && unsafe { id.value.valueString } as usize <= self.debug_strings.len()
                    );
                    let str = &self.debug_strings[unsafe { id.value.valueString } as usize - 1];
                    formatAppend(
                        result,
                        format_args!("{}", unsafe {
                            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                                str.data as *const u8,
                                str.length,
                            ))
                        }),
                    );

                    if count > 1 {
                        let id = self.constants[id1 as usize];
                        LUAU_ASSERT!(
                            id.r#type == Type::Type_String
                                && unsafe { id.value.valueString } as usize
                                    <= self.debug_strings.len()
                        );
                        let str = &self.debug_strings[unsafe { id.value.valueString } as usize - 1];
                        formatAppend(
                            result,
                            format_args!(".{}", unsafe {
                                core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                                    str.data as *const u8,
                                    str.length,
                                ))
                            }),
                        );
                    }

                    if count > 2 {
                        let id = self.constants[id2 as usize];
                        LUAU_ASSERT!(
                            id.r#type == Type::Type_String
                                && unsafe { id.value.valueString } as usize
                                    <= self.debug_strings.len()
                        );
                        let str = &self.debug_strings[unsafe { id.value.valueString } as usize - 1];
                        formatAppend(
                            result,
                            format_args!(".{}", unsafe {
                                core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                                    str.data as *const u8,
                                    str.length,
                                ))
                            }),
                        );
                    }
                }
            }
            Type::Type_Table => {
                if detailed {
                    let shape = &self.table_shapes[unsafe { data.value.valueTable } as usize];
                    let sizenode = if shape.length > 0 {
                        1u32 << (ceillog_2(shape.length as i32) as u32)
                    } else {
                        0u32
                    };
                    let mask = if sizenode > 0 { sizenode - 1 } else { 0u32 };

                    let mut slots = vec![0u32; shape.length as usize];
                    let mut slot_owner = vec![!0u32; sizenode as usize];

                    for i in 0..shape.length as usize {
                        let key_const = &self.constants[shape.keys[i] as usize];
                        LUAU_ASSERT!(
                            key_const.r#type == Type::Type_String
                                && unsafe { key_const.value.valueString } != 0
                        );
                        let str = &self.debug_strings
                            [unsafe { key_const.value.valueString } as usize - 1];
                        let hash = bytecode_builder_get_string_hash(*str);
                        slots[i] = hash & mask;

                        if slot_owner[slots[i] as usize] == !0u32 {
                            slot_owner[slots[i] as usize] = i as u32;
                        }
                    }

                    formatAppend(result, format_args!("{{"));

                    for i in 0..shape.length as usize {
                        if i > 0 {
                            formatAppend(result, format_args!(", "));
                        }

                        formatAppend(result, format_args!("["));
                        self.dump_constant(result, shape.keys[i], false);
                        formatAppend(result, format_args!("]"));

                        if shape.hasConstants && shape.constants[i] != -1 {
                            formatAppend(result, format_args!(" = "));
                            self.dump_constant(result, shape.constants[i], false);
                        }

                        formatAppend(result, format_args!(" #{}", slots[i]));

                        if slot_owner[slots[i] as usize] != i as u32 {
                            formatAppend(result, format_args!(" (conflict)"));
                        }
                    }

                    formatAppend(result, format_args!("}} sizenode={}", sizenode));
                } else {
                    formatAppend(result, format_args!("{{...}}"));
                }
            }
            Type::Type_Closure => {
                let func = &self.functions[unsafe { data.value.valueClosure } as usize];
                if !func.dumpname.is_empty() {
                    formatAppend(result, format_args!("'{}'", func.dumpname));
                }
            }
            Type::Type_ClassShape => {
                let cs = &self.class_shapes[unsafe { data.value.valueClassShape } as usize];
                let class_name_const = &self.constants[cs.className as usize];
                LUAU_ASSERT!(
                    class_name_const.r#type == Type::Type_String
                        && unsafe { class_name_const.value.valueString } as usize
                            <= self.debug_strings.len()
                );
                let str =
                    &self.debug_strings[unsafe { class_name_const.value.valueString } as usize - 1];
                LUAU_ASSERT!(printableStringConstant(unsafe {
                    core::slice::from_raw_parts(str.data as *const u8, str.length)
                }));
                formatAppend(
                    result,
                    format_args!(
                        "class {} (props: {}, methods: {})",
                        unsafe {
                            core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                                str.data as *const u8,
                                str.length,
                            ))
                        },
                        cs.propertyNames.len(),
                        cs.methodNames.len()
                    ),
                );
            }
        }
    }
}
