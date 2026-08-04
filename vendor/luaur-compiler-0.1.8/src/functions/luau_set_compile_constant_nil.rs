use crate::functions::set_compile_constant_nil::set_compile_constant_nil;
use crate::type_aliases::lua_compile_constant::lua_CompileConstant;

pub fn luau_set_compile_constant_nil(constant: lua_CompileConstant) {
    set_compile_constant_nil(constant as *mut core::ffi::c_void);
}
