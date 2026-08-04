use crate::enums::r#type::Type;
use crate::functions::write_byte::write_byte;
use crate::functions::write_double::write_double;
use crate::functions::write_float::write_float;
use crate::functions::write_int::write_int;
use crate::functions::write_var_int::write_var_int;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::class_shape::ClassShape;
use crate::records::constant::Constant;
use crate::records::debug_local_bytecode_builder::DebugLocal;
use crate::records::debug_upval::DebugUpval;
use crate::records::function::Function;
use crate::records::table_shape::TableShape;
use crate::records::typed_local_bytecode_builder::TypedLocal;
use crate::records::typed_upval::TypedUpval;
use alloc::string::String;
use alloc::vec::Vec;
use core::cmp;
use luaur_common::enums::luau_bytecode_tag::LuauBytecodeTag;
use luaur_common::enums::luau_feedback_type::LuauFeedbackType;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::FFlag;

impl BytecodeBuilder {
    pub fn write_function(&mut self, ss: &mut String, id: u32, flags: u8) {
        LUAU_ASSERT!(id < self.functions.len() as u32);
        let func = &self.functions[id as usize];

        // header
        write_byte(ss, func.maxstacksize);
        write_byte(ss, func.numparams);
        write_byte(ss, func.numupvalues);
        write_byte(ss, if func.isvararg { 1 } else { 0 });

        write_byte(ss, flags);

        if !func.typeinfo.is_empty()
            || !self.typed_upvals.is_empty()
            || !self.typed_locals.is_empty()
        {
            // collect type info into a temporary string to know the overall size of type data
            self.temp_type_info.clear();
            write_var_int(&mut self.temp_type_info, func.typeinfo.len() as u64);
            write_var_int(&mut self.temp_type_info, self.typed_upvals.len() as u64);
            write_var_int(&mut self.temp_type_info, self.typed_locals.len() as u64);

            self.temp_type_info.push_str(&func.typeinfo);

            for l in &self.typed_upvals {
                write_byte(&mut self.temp_type_info, l.r#type.0 as u8);
            }

            for l in &self.typed_locals {
                write_byte(&mut self.temp_type_info, l.r#type.0 as u8);
                write_byte(&mut self.temp_type_info, l.reg);
                write_var_int(&mut self.temp_type_info, l.startpc as u64);
                LUAU_ASSERT!(l.endpc >= l.startpc);
                write_var_int(&mut self.temp_type_info, (l.endpc - l.startpc) as u64);
            }

            write_var_int(ss, self.temp_type_info.len() as u64);
            ss.push_str(&self.temp_type_info);
        } else {
            write_var_int(ss, 0);
        }

        // instructions
        write_var_int(ss, self.insns.len() as u64);

        for &insn in &self.insns {
            write_int(ss, insn as i32);
        }

        // constants
        write_var_int(ss, self.constants.len() as u64);

        for c in &self.constants {
            match c.r#type {
                Type::Type_Nil => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_NIL.0 as u8);
                }
                Type::Type_Boolean => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_BOOLEAN.0 as u8);
                    write_byte(ss, unsafe { c.value.valueBoolean } as u8);
                }
                Type::Type_Number => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_NUMBER.0 as u8);
                    write_double(ss, unsafe { c.value.valueNumber });
                }
                Type::Type_Integer => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_INTEGER.0 as u8);
                    let value = unsafe { c.value.valueInteger64 };
                    if value < 0 {
                        write_byte(ss, 1);
                        write_var_int(ss, (!(value as u64)).wrapping_add(1));
                    } else {
                        write_byte(ss, 0);
                        write_var_int(ss, value as u64);
                    }
                }
                Type::Type_Vector => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_VECTOR.0 as u8);
                    let vec = unsafe { c.value.valueVector };
                    write_float(ss, vec[0]);
                    write_float(ss, vec[1]);
                    write_float(ss, vec[2]);
                    write_float(ss, vec[3]);
                }
                Type::Type_String => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_STRING.0 as u8);
                    write_var_int(ss, unsafe { c.value.valueString } as u64);
                }
                Type::Type_Import => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_IMPORT.0 as u8);
                    write_int(ss, unsafe { c.value.valueImport } as i32);
                }
                Type::Type_Table => {
                    let shape = &self.table_shapes[unsafe { c.value.valueTable } as usize];
                    if FFlag::LuauCompileDuptableConstantPack2.get() && shape.hasConstants {
                        write_byte(
                            ss,
                            LuauBytecodeTag::LBC_CONSTANT_TABLE_WITH_CONSTANTS.0 as u8,
                        );
                        write_var_int(ss, shape.length as u64);
                        for i in 0..shape.length as usize {
                            write_var_int(ss, shape.keys[i] as u64);
                            write_int(ss, shape.constants[i]);
                        }
                    } else {
                        write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_TABLE.0 as u8);
                        write_var_int(ss, shape.length as u64);
                        for i in 0..shape.length as usize {
                            write_var_int(ss, shape.keys[i] as u64);
                        }
                    }
                }
                Type::Type_Closure => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_CLOSURE.0 as u8);
                    write_var_int(ss, unsafe { c.value.valueClosure } as u64);
                }
                Type::Type_ClassShape => {
                    write_byte(ss, LuauBytecodeTag::LBC_CONSTANT_CLASS_SHAPE.0 as u8);
                    let cs = &self.class_shapes[unsafe { c.value.valueClassShape } as usize];
                    self.write_class_shape(ss, cs);
                }
            }
        }

        // child protos
        write_var_int(ss, self.protos.len() as u64);

        for &child in &self.protos {
            write_var_int(ss, child as u64);
        }

        // debug info
        write_var_int(ss, func.debuglinedefined as u64);
        write_var_int(ss, func.debugname as u64);

        let mut has_lines = true;

        for &line in &self.lines {
            if line == 0 {
                has_lines = false;
                break;
            }
        }

        if has_lines {
            write_byte(ss, 1);

            self.write_line_info(ss);
        } else {
            write_byte(ss, 0);
        }

        let has_debug = !self.debug_locals.is_empty() || !self.debug_upvals.is_empty();

        if has_debug {
            write_byte(ss, 1);

            write_var_int(ss, self.debug_locals.len() as u64);

            for l in &self.debug_locals {
                write_var_int(ss, l.name as u64);
                write_var_int(ss, l.startpc as u64);
                write_var_int(ss, l.endpc as u64);
                write_byte(ss, l.reg);
            }

            write_var_int(ss, self.debug_upvals.len() as u64);

            for l in &self.debug_upvals {
                write_var_int(ss, l.name as u64);
            }
        } else {
            write_byte(ss, 0);
        }

        if FFlag::LuauEmitCallFeedback.get() {
            // Feedback Slots
            write_var_int(ss, self.fb_slots.len() as u64);
            for &pc in &self.fb_slots {
                write_byte(ss, LuauFeedbackType::LFT_CALLTARGET as u8);
                write_var_int(ss, pc as u64);
            }
        }
    }
}
