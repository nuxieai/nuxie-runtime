use crate::functions::set_compile_constant_vectord::set_compile_constant_vectord;
use crate::type_aliases::lua_compile_constant::lua_CompileConstant;

pub fn luau_set_compile_constant_vectord(
    constant: lua_CompileConstant,
    x: f64,
    y: f64,
    z: f64,
    w: f64,
) {
    set_compile_constant_vectord(constant as _, x, y, z, w);
}
