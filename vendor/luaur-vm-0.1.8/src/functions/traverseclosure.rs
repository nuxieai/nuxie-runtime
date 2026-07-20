//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:398:traverseclosure`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:398-415, hand-ported)

use crate::macros::markobject::markobject;
use crate::macros::markvalue::markvalue;
use crate::records::closure::{CClosure, Closure, LClosure};
use crate::records::global_state::global_State;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) unsafe fn traverseclosure(g: *mut global_State, cl: *mut Closure) {
    markobject!(g, (*cl).env);
    if (*cl).isC != 0 {
        // ManuallyDrop is repr(transparent); upvals/uprefs are C flexible arrays
        let c = core::ptr::addr_of_mut!((*cl).inner.c) as *mut CClosure;
        let upvals = core::ptr::addr_of_mut!((*c).upvals) as *mut TValue;
        for i in 0..(*cl).nupvalues as usize {
            // mark its upvalues
            markvalue!(g, upvals.add(i));
        }
    } else {
        let l = core::ptr::addr_of_mut!((*cl).inner.l) as *mut LClosure;
        LUAU_ASSERT!((*cl).nupvalues as i32 == (*(*l).p).nups as i32);
        markobject!(g, (*l).p);
        let uprefs = core::ptr::addr_of_mut!((*l).uprefs) as *mut TValue;
        for i in 0..(*cl).nupvalues as usize {
            // mark its upvalues
            markvalue!(g, uprefs.add(i));
        }
    }
}
