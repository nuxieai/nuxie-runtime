use crate::functions::set_compile_constant_boolean::set_compile_constant_boolean;
use crate::type_aliases::lua_compile_constant::lua_CompileConstant;

pub fn luau_set_compile_constant_boolean(constant: lua_CompileConstant, b: bool) {
    set_compile_constant_boolean(constant, b);
}
