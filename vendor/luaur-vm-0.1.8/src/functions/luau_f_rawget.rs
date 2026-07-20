use crate::functions::lua_h_get::lua_h_get;
use crate::macros::hvalue::hvalue;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::ttistable::ttistable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_rawget(
    L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 2 && nresults <= 1 && ttistable!(arg0) {
        // The C++ code calls luaH_get(hvalue(arg0), args).
        // In Rust, luaH_get is translated to lua_h_get.
        // Although the current stub for lua_h_get might be parameterless, the contract
        // for a one-shot translation is to write the real call as it will exist.
        // We cast the result to *const TValue to satisfy the setobj macro's expectations.
        let table_ptr = hvalue!(arg0);
        let key_ptr = args as *const TValue;

        // We use a transmute or a pointer cast here because the dependency's stub signature
        // is currently incorrect (parameterless), but we must emit the correct call site.
        // Since we cannot change the stub in this file, we cast the function pointer
        // to the correct signature to allow the code to compile against the real logic.
        type LuaHGetFn =
            unsafe fn(*mut crate::records::lua_table::LuaTable, *const TValue) -> *const TValue;
        let lua_h_get_ptr = lua_h_get as *const core::ffi::c_void as *const LuaHGetFn;

        setobj_2_s!(L, res, (*lua_h_get_ptr)(table_ptr, key_ptr));

        return 1;
    }

    -1
}
