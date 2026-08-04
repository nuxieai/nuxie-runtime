use crate::functions::set_compile_constant_string::set_compile_constant_string;
use crate::type_aliases::lua_compile_constant::lua_CompileConstant;
use core::ffi::c_char;

pub fn luau_set_compile_constant_string(constant: lua_CompileConstant, s: *const c_char, l: usize) {
    set_compile_constant_string(constant, s, l);
}
