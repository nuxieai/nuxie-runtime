use crate::functions::lua_a_toobject::luaA_toobject;
use crate::functions::lua_m_freearray::getcounters;
use crate::macros::api_check::api_check;
use crate::macros::clvalue::clvalue;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::proto::Proto;
use crate::type_aliases::lua_counter_function::lua_CounterFunction;
use crate::type_aliases::lua_counter_value::lua_CounterValue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn lua_getcounters(
    L: *mut lua_State,
    funcindex: core::ffi::c_int,
    context: *mut core::ffi::c_void,
    functionvisit: lua_CounterFunction,
    countervisit: lua_CounterValue,
) {
    let func: *const TValue = luaA_toobject(L, funcindex);
    api_check!(L, ttisfunction!(func) && (*clvalue!(func)).isC == 0);

    if (*(*L).global).ecb.getcounterdata.is_none() {
        return;
    }

    let cl = clvalue!(func);
    let lcl = core::ptr::addr_of!((*cl).inner.l).cast::<crate::records::closure::LClosure>();
    let p = (*lcl).p;

    getcounters(L, p as *mut Proto, context, functionvisit, countervisit);
}
