use crate::functions::set_compile_constant_integer_64::set_compile_constant_integer_64;
use crate::type_aliases::compile_constant::CompileConstant;
use crate::type_aliases::lua_compile_constant::lua_CompileConstant;

#[allow(non_snake_case)]
pub fn luau_set_compile_constant_integer_64(constant: *mut lua_CompileConstant, l: i64) {
    set_compile_constant_integer_64(constant as CompileConstant, l);
}
