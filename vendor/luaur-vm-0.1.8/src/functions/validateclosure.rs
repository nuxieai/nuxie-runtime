use crate::functions::validateobjref::validateobjref;
use crate::functions::validateref::validateref;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::closure::{CClosure, Closure, LClosure};
use crate::records::global_state::global_State;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn validateclosure(g: *mut global_State, cl: *mut Closure) {
    validateobjref(g, obj2gco!(cl as *mut Closure), obj2gco!((*cl).env));

    if (*cl).isC != 0 {
        let c = core::ptr::addr_of_mut!((*cl).inner.c) as *mut CClosure;
        for i in 0..(*cl).nupvalues as usize {
            validateref(
                g,
                obj2gco!(cl as *mut Closure),
                (*c).upvals.as_mut_ptr().add(i),
            );
        }
    } else {
        let l = core::ptr::addr_of_mut!((*cl).inner.l) as *mut LClosure;
        LUAU_ASSERT!((*cl).nupvalues as i32 == (*(*l).p).nups as i32);

        validateobjref(g, obj2gco!(cl as *mut Closure), obj2gco!((*l).p));

        let uprefs =
            core::ptr::addr_of_mut!((*l).uprefs) as *mut crate::type_aliases::t_value::TValue;
        for i in 0..(*cl).nupvalues as usize {
            validateref(g, obj2gco!(cl as *mut Closure), uprefs.add(i));
        }
    }
}
