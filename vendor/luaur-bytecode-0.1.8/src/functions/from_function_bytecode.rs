extern crate alloc;
use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::functions::read_string::read_string;
use crate::records::bc_function::BcFunction;
use crate::records::bc_vm_const::BcVmConst;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::records::debug_local_bytecode_graph::DebugLocal;
use crate::records::table_shape::TableShape;
use crate::records::typed_local_bytecode_graph::TypedLocal;
use crate::type_aliases::comp_time_bc_function::CompTimeBcFunction;
use crate::type_aliases::instruction::Instruction;
use alloc::string::String;
use alloc::vec::Vec;
use core::option::Option;
use luaur_common::enums::luau_bytecode_tag::LuauBytecodeTag;
use luaur_common::enums::luau_bytecode_type::{LuauBytecodeType, LBC_TYPE_NIL};
use luaur_common::functions::read::read;
use luaur_common::functions::read_var_int::read_var_int;
use luaur_common::functions::read_var_int_64::read_var_int_64;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_upper_case_globals)]
const LBC_CONSTANT_NIL: u8 = LuauBytecodeTag::LBC_CONSTANT_NIL.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_BOOLEAN: u8 = LuauBytecodeTag::LBC_CONSTANT_BOOLEAN.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_NUMBER: u8 = LuauBytecodeTag::LBC_CONSTANT_NUMBER.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_STRING: u8 = LuauBytecodeTag::LBC_CONSTANT_STRING.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_IMPORT: u8 = LuauBytecodeTag::LBC_CONSTANT_IMPORT.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_TABLE: u8 = LuauBytecodeTag::LBC_CONSTANT_TABLE.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_CLOSURE: u8 = LuauBytecodeTag::LBC_CONSTANT_CLOSURE.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_VECTOR: u8 = LuauBytecodeTag::LBC_CONSTANT_VECTOR.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_TABLE_WITH_CONSTANTS: u8 =
    LuauBytecodeTag::LBC_CONSTANT_TABLE_WITH_CONSTANTS.0 as u8;
#[allow(non_upper_case_globals)]
const LBC_CONSTANT_INTEGER: u8 = LuauBytecodeTag::LBC_CONSTANT_INTEGER.0 as u8;

pub fn from_function_bytecode(
    bytecode: String,
    strings: &mut Vec<&[u8]>,
) -> Option<CompTimeBcFunction> {
    let mut data = bytecode.as_bytes();
    let mut offset = 0usize;

    let mut fn_ = BcFunction::default();

    fn_.maxstacksize = read::<u8>(&data, &mut offset);
    fn_.numparams = read::<u8>(&data, &mut offset);
    fn_.nups = read::<u8>(&data, &mut offset);
    fn_.is_vararg = read::<u8>(&data, &mut offset) != 0;
    fn_.flags = read::<u8>(&data, &mut offset);

    let types_size = read_var_int(&data, &mut offset);
    if types_size > 0 {
        let type_info_size = read_var_int(&data, &mut offset);
        let typed_upval_size = read_var_int(&data, &mut offset);
        let typed_local_size = read_var_int(&data, &mut offset);

        fn_.type_info = bytecode[offset..offset + type_info_size as usize].to_string();
        offset += type_info_size as usize;

        fn_.upvalue_types
            .resize(typed_upval_size as usize, LBC_TYPE_NIL);
        for i in 0..typed_upval_size as usize {
            let ty = read::<u8>(&data, &mut offset);
            fn_.upvalue_types[i] = LuauBytecodeType(ty as u16);
        }

        fn_.local_types
            .resize(typed_local_size as usize, TypedLocal::default());
        for i in 0..typed_local_size as usize {
            let ty = read::<u8>(&data, &mut offset);
            let reg = read::<u8>(&data, &mut offset);
            let startpc = read_var_int(&data, &mut offset);
            let endpc = startpc + read_var_int(&data, &mut offset);
            fn_.local_types[i] = TypedLocal {
                r#type: LuauBytecodeType(ty as u16),
                reg,
                startpc,
                endpc,
            };
        }
    }

    let codesize = read_var_int(&data, &mut offset) as usize;
    let mut code: Vec<Instruction> = Vec::with_capacity(codesize);
    for _ in 0..codesize {
        code.push(read::<Instruction>(&data, &mut offset));
    }

    let sizek = read_var_int(&data, &mut offset) as usize;
    fn_.constants.resize(sizek, BcVmConst::new());
    for i in 0..sizek {
        let const_type = read::<u8>(&data, &mut offset);
        match const_type {
            LBC_CONSTANT_NIL => {
                fn_.constants[i].kind = BcVmConstKind::Nil;
            }
            LBC_CONSTANT_BOOLEAN => {
                fn_.constants[i].kind = BcVmConstKind::Boolean;
                unsafe {
                    fn_.constants[i].value.valueBoolean = read::<u8>(&data, &mut offset) != 0;
                }
            }
            LBC_CONSTANT_NUMBER => {
                fn_.constants[i].kind = BcVmConstKind::Number;
                unsafe {
                    fn_.constants[i].value.valueNumber = read::<f64>(&data, &mut offset);
                }
            }
            LBC_CONSTANT_VECTOR => {
                fn_.constants[i].kind = BcVmConstKind::Vector;
                unsafe {
                    fn_.constants[i].value.valueVector = [
                        read::<f32>(&data, &mut offset),
                        read::<f32>(&data, &mut offset),
                        read::<f32>(&data, &mut offset),
                        read::<f32>(&data, &mut offset),
                    ];
                }
            }
            LBC_CONSTANT_STRING => {
                fn_.constants[i].kind = BcVmConstKind::String;
                let s = read_string(strings, &data, &mut offset);
                unsafe {
                    let value = core::str::from_utf8_unchecked(s);
                    fn_.constants[i].value.valueString =
                        core::mem::transmute::<&str, &'static str>(value);
                }
            }
            LBC_CONSTANT_IMPORT => {
                fn_.constants[i].kind = BcVmConstKind::Import;
                unsafe {
                    fn_.constants[i].value.valueImport = read::<u32>(&data, &mut offset);
                }
            }
            LBC_CONSTANT_TABLE | LBC_CONSTANT_TABLE_WITH_CONSTANTS => {
                fn_.constants[i].kind = BcVmConstKind::Table;
                fn_.constants[i].value.valueTable = fn_.table_shapes.len() as u32;

                let mut shape = TableShape::default();
                shape.length = read_var_int(&data, &mut offset);
                shape.hasConstants = const_type == LBC_CONSTANT_TABLE_WITH_CONSTANTS;

                for j in 0..shape.length as usize {
                    let key = read_var_int(&data, &mut offset) as i32;
                    shape.keys[j] = key;
                    if shape.hasConstants {
                        let value = read::<i32>(&data, &mut offset);
                        shape.constants[j] = value;
                    }
                }
                fn_.table_shapes.push(shape);
            }
            LBC_CONSTANT_CLOSURE => {
                fn_.constants[i].kind = BcVmConstKind::Closure;
                fn_.constants[i].value.valueClosure = read_var_int(&data, &mut offset);
            }
            LBC_CONSTANT_INTEGER => {
                fn_.constants[i].kind = BcVmConstKind::Integer;
                let is_negative = read::<u8>(&data, &mut offset) != 0;
                let magnitude = read_var_int_64(&data, &mut offset);
                fn_.constants[i].value.valueInteger = if is_negative {
                    !(magnitude - 1) as i64
                } else {
                    magnitude as i64
                };
            }
            _ => {
                LUAU_ASSERT!(false, "Unknown constant type!");
                return None;
            }
        }
    }

    let psize = read_var_int(&data, &mut offset) as usize;
    fn_.protos.resize(psize, 0);
    for i in 0..psize {
        fn_.protos[i] = read_var_int(&data, &mut offset);
    }

    fn_.linedefined = read_var_int(&data, &mut offset);
    fn_.debugname =
        unsafe { core::str::from_utf8_unchecked(read_string(strings, &data, &mut offset)) }
            .to_string();

    let lineinfo = read::<u8>(&data, &mut offset);
    let mut lines: Vec<u32> = Vec::new();

    if lineinfo != 0 {
        let linegaplog2 = read::<u8>(&data, &mut offset) as usize;

        let intervals = ((codesize - 1) >> linegaplog2) + 1;
        let absoffset = (codesize + 3) & !3;

        let mut lineinfo_bytes = vec![0u8; absoffset];
        let mut abslineinfo = vec![0i32; intervals];

        let mut lastoffset = 0u8;
        for i in 0..codesize {
            lastoffset = lastoffset.wrapping_add(read::<u8>(&data, &mut offset));
            lineinfo_bytes[i] = lastoffset;
        }

        let mut lastline = 0i32;
        for i in 0..intervals {
            lastline += read::<i32>(&data, &mut offset);
            abslineinfo[i] = lastline;
        }

        lines.resize(codesize, 0);
        for i in 0..codesize {
            let idx = i >> linegaplog2;
            let abs = abslineinfo[idx];
            let off = lineinfo_bytes[i] as i32;
            lines[i] = (abs + off) as u32;
        }
    }

    let debuginfo = read::<u8>(&data, &mut offset);

    if debuginfo != 0 {
        let sizelocvars = read_var_int(&data, &mut offset) as usize;
        fn_.locals.resize(sizelocvars, DebugLocal::default());

        for i in 0..sizelocvars {
            let varname = read_string(strings, &data, &mut offset);
            let startpc = read_var_int(&data, &mut offset);
            let endpc = read_var_int(&data, &mut offset);
            let reg = read::<u8>(&data, &mut offset);
            fn_.locals[i] = DebugLocal {
                varname: unsafe {
                    core::str::from_utf8_unchecked(core::slice::from_raw_parts(
                        varname.as_ptr() as *const u8,
                        varname.len(),
                    ))
                },
                reg,
                startpc,
                endpc,
            };
        }

        let sizeupvalues = read_var_int(&data, &mut offset) as usize;
        fn_.upvalue_names.resize(sizeupvalues, String::new());

        for i in 0..sizeupvalues {
            let name = read_string(strings, &data, &mut offset);
            fn_.upvalue_names[i] = unsafe { core::str::from_utf8_unchecked(name) }.to_string();
        }
    }

    let mut insns_pc: Vec<u32> = Vec::new();
    let mut graph_parser = BytecodeGraphParser::new(&mut fn_);
    if !graph_parser.rebuild_graph(code.as_ptr(), codesize as u32, &mut lines, &mut insns_pc) {
        return None;
    }

    for l in &mut fn_.local_types {
        l.startpc = if l.startpc < insns_pc.len() as u32 {
            insns_pc[l.startpc as usize]
        } else {
            codesize as u32
        };
        l.endpc = if l.endpc < insns_pc.len() as u32 {
            insns_pc[l.endpc as usize]
        } else {
            codesize as u32
        };
    }

    for l in &mut fn_.locals {
        l.startpc = if l.startpc < insns_pc.len() as u32 {
            insns_pc[l.startpc as usize]
        } else {
            codesize as u32
        };
        l.endpc = if l.endpc < insns_pc.len() as u32 {
            insns_pc[l.endpc as usize]
        } else {
            codesize as u32
        };
    }

    Some(fn_)
}
