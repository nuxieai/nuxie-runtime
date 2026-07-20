use crate::functions::lua_c_barrierf::lua_c_barrierf;
use crate::macros::hvalue::hvalue;
use crate::macros::sethvalue::sethvalue;
use crate::macros::ttistable::ttistable;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_setmetatable(
    L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    // note: setmetatable(_, nil) is rare so we use fallback for it to optimize the fast path
    if nparams >= 2 && nresults <= 1 && ttistable!(arg0) && ttistable!(args) {
        let t = hvalue!(arg0);
        if (*t).readonly != 0 || !(*t).metatable.is_null() {
            return -1; // note: overwriting non-null metatable is very rare but it requires __metatable check
        }

        let mt = hvalue!(args);
        (*t).metatable = mt;

        // luaC_objbarrier(L, t, mt);
        // The barrier logic: if (isblack(p) && iswhite(o)) luaC_barrierf(L, p, o);
        let p_gco = t as *mut crate::records::gc_object::GCObject;
        let o_gco = mt as *mut crate::records::gc_object::GCObject;

        // Layout for bit use in `marked` field:
        // bit 0,1 - WHITEBITS (1 | 2 = 3)
        // bit 2 - BLACKBIT (4)
        let is_black = ((*p_gco).gch.marked & (1 << 2)) != 0;
        let is_white = ((*o_gco).gch.marked & 3) != 0;

        if is_black && is_white {
            // The dependency's stub signature is currently parameterless, but we must emit the correct call site.
            // We cast the function pointer to the correct signature to allow the code to compile against the real logic.
            type LuaCBarrierfFn = unsafe fn(
                *mut crate::records::lua_state::lua_State,
                *mut crate::records::gc_object::GCObject,
                *mut crate::records::gc_object::GCObject,
            );
            let lua_c_barrierf_ptr =
                lua_c_barrierf as *const core::ffi::c_void as *const LuaCBarrierfFn;
            (*lua_c_barrierf_ptr)(L, p_gco, o_gco);
        }

        sethvalue!(L, res, t);
        return 1;
    }

    -1
}
