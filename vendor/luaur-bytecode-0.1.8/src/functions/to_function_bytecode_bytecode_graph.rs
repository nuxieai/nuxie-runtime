use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::comp_time_bytecode_graph_serializer::CompTimeBytecodeGraphSerializer;
use crate::records::string_ref::StringRef;
use crate::type_aliases::comp_time_bc_function::CompTimeBcFunction;
use alloc::string::String;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

fn string_ref(value: &str) -> StringRef {
    StringRef {
        data: value.as_ptr() as *const i8,
        length: value.len(),
    }
}

pub fn to_function_bytecode_bytecode_builder_comp_time_bc_function(
    bcb: &mut BytecodeBuilder,
    fn_: &mut CompTimeBcFunction,
) -> String {
    let function_id = bcb.begin_function(fn_.numparams, fn_.is_vararg);

    if !fn_.debugname.is_empty() {
        bcb.set_debug_function_name(string_ref(&fn_.debugname));
    }

    bcb.set_debug_function_line_defined(fn_.linedefined as i32);
    bcb.set_function_type_info(fn_.type_info.clone());

    for t in &fn_.upvalue_types {
        bcb.push_upval_type_info(*t);
    }

    for upval in &fn_.upvalue_names {
        bcb.push_debug_upval(string_ref(upval));
    }

    let mut consts: Vec<u16> = Vec::with_capacity(fn_.constants.len());

    for c in &fn_.constants {
        match c.kind {
            BcVmConstKind::Nil => consts.push(bcb.add_constant_nil() as u16),
            BcVmConstKind::Boolean => {
                consts.push(bcb.add_constant_boolean(unsafe { c.value.valueBoolean }) as u16)
            }
            BcVmConstKind::Number => {
                consts.push(bcb.add_constant_number(unsafe { c.value.valueNumber }) as u16)
            }
            BcVmConstKind::Vector => {
                let value = unsafe { c.value.valueVector };
                consts.push(bcb.add_constant_vector(value[0], value[1], value[2], value[3]) as u16);
            }
            BcVmConstKind::String => consts
                .push(bcb.add_constant_string(string_ref(unsafe { c.value.valueString })) as u16),
            BcVmConstKind::Import => {
                consts.push(bcb.add_import(unsafe { c.value.valueImport }) as u16)
            }
            BcVmConstKind::Table => {
                let value_table = unsafe { c.value.valueTable };
                LUAU_ASSERT!(value_table < fn_.table_shapes.len() as u32);
                consts.push(bcb.add_constant_table(&fn_.table_shapes[value_table as usize]) as u16);
            }
            BcVmConstKind::Closure => {
                consts.push(bcb.add_constant_closure(unsafe { c.value.valueClosure }) as u16)
            }
            BcVmConstKind::Integer => {
                consts.push(bcb.add_constant_integer(unsafe { c.value.valueInteger }) as u16)
            }
        }
    }

    for fid in &fn_.protos {
        bcb.add_child_function(*fid);
    }

    let mut serializer = CompTimeBytecodeGraphSerializer::comp_time_bytecode_graph_serializer_comp_time_bytecode_graph_serializer(bcb, fn_, &mut consts);
    let insns_pc = serializer.emit_bytecode();

    for local in &fn_.local_types {
        let startpc = if local.startpc < insns_pc.len() as u32 {
            insns_pc[local.startpc as usize]
        } else {
            bcb.get_debug_pc()
        };
        let endpc = if local.endpc < insns_pc.len() as u32 {
            insns_pc[local.endpc as usize]
        } else {
            bcb.get_debug_pc()
        };
        bcb.push_local_type_info(local.r#type, local.reg, startpc, endpc);
    }

    for local in &fn_.locals {
        let startpc = if local.startpc < insns_pc.len() as u32 {
            insns_pc[local.startpc as usize]
        } else {
            bcb.get_debug_pc()
        };
        let endpc = if local.endpc < insns_pc.len() as u32 {
            insns_pc[local.endpc as usize]
        } else {
            bcb.get_debug_pc()
        };
        bcb.push_debug_local(string_ref(local.varname), local.reg, startpc, endpc);
    }

    bcb.fold_jumps();
    bcb.expand_jumps();
    bcb.end_function(fn_.maxstacksize, fn_.nups, fn_.flags);

    if serializer.error() {
        return String::new();
    }

    bcb.get_function_data(function_id)
}
