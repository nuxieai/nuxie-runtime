use crate::enums::lua_type::lua_Type;
use crate::functions::index_2_addr::index2addr;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_f_new_lclosure::luaF_newLclosure;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::setclvalue::setclvalue;
use crate::macros::setobj_2_n::setobj2n;
use crate::records::closure::Closure;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;

#[allow(non_snake_case)]
pub unsafe fn lua_clonefunction(L: *mut lua_State, idx: core::ffi::c_int) {
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);
    let p: StkId = index2addr(L, idx);
    let cl = core::ptr::addr_of_mut!((*(*p).value.gc).cl) as *mut Closure;
    api_check!(
        L,
        (*p).tt() == lua_Type::LUA_TFUNCTION as core::ffi::c_int && (*cl).isC == 0
    );
    let lc = core::ptr::addr_of!((*cl).inner.l) as *const crate::records::closure::LClosure;
    let newcl: *mut Closure =
        luaF_newLclosure(L, (*cl).nupvalues as core::ffi::c_int, (*L).gt, (*lc).p);
    let newlc = core::ptr::addr_of_mut!((*newcl).inner.l) as *mut crate::records::closure::LClosure;
    for i in 0..(*cl).nupvalues as i32 {
        setobj2n!(
            L,
            (*newlc).uprefs.as_mut_ptr().add(i as usize),
            (*lc).uprefs.as_ptr().add(i as usize)
        );
    }
    setclvalue!(L, (*L).top, newcl);
    api_incr_top!(L);
}
