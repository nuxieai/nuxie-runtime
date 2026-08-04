use crate::functions::to_function_bytecode_bytecode_graph::to_function_bytecode_bytecode_builder_comp_time_bc_function;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::type_aliases::comp_time_bc_function::CompTimeBcFunction;
use alloc::string::String;

pub fn to_function_bytecode_comp_time_bc_function(_fn_: &mut CompTimeBcFunction) -> String {
    let mut bcb = BytecodeBuilder::new(None);
    to_function_bytecode_bytecode_builder_comp_time_bc_function(&mut bcb, _fn_)
}
