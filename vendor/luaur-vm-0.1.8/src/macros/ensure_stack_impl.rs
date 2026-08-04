#[macro_export]
#[allow(non_snake_case)]
macro_rules! ensure_stack_impl {
    ($L:expr, $errorL:expr, $size:expr) => {{
        unsafe {
            let size = $size as core::ffi::c_int;
            if luaur_common::FFlag::LuauAutoStack.get()
                && (*$L).top.offset(size as isize) > (*(*$L).ci).top
                && $crate::functions::lua_checkstack::lua_checkstack($L, size) == 0
            {
                $crate::functions::lua_o_pushfstring::luaO_pushfstring(
                    $errorL,
                    c"stack overflow".as_ptr(),
                    format_args!("stack overflow"),
                );
                $crate::functions::lua_error::lua_error($errorL);
            }
        }
    }};
}

pub use ensure_stack_impl;
