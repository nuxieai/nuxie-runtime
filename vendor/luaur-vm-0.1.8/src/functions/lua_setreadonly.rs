use crate::functions::index_2_addr::index2addr;
use crate::macros::api_check::api_check;
use crate::macros::hvalue::hvalue;
use crate::macros::registry::registry;
use crate::macros::ttistable::ttistable;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_setreadonly(
    L: *mut lua_State,
    objindex: core::ffi::c_int,
    enabled: core::ffi::c_int,
) {
    let o: *const TValue = index2addr(L, objindex);
    api_check!(L, ttistable!(o));

    let t: *mut LuaTable = hvalue!(o);

    // The registry macro returns a reference &TValue.
    // hvalue! expects a pointer *mut TValue.
    // We use addr_of! to get a pointer from the reference safely before casting.
    api_check!(
        L,
        t != hvalue!(core::ptr::addr_of!(*registry!(L)) as *mut TValue)
    );

    (*t).readonly = if enabled != 0 { 1 } else { 0 };
}
