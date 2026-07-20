//! Node: `cxx:Function:Luau.VM:VM/src/lgc.cpp:372:traverseproto`
//! Source: `VM/src/lgc.cpp` (lgc.cpp:372-396, hand-ported)

use crate::macros::markobject::markobject;
use crate::macros::markvalue::markvalue;
use crate::macros::stringmark::stringmark;
use crate::records::global_state::global_State;
use crate::records::proto::Proto;

// All marks are conditional because a GC may happen while the
// prototype is still being created
#[allow(non_snake_case)]
pub(crate) unsafe fn traverseproto(g: *mut global_State, f: *mut Proto) {
    if !(*f).source.is_null() {
        stringmark!((*f).source);
    }
    if !(*f).debugname.is_null() {
        stringmark!((*f).debugname);
    }
    for i in 0..(*f).sizek as usize {
        // mark literals
        markvalue!(g, (*f).k.add(i));
    }
    for i in 0..(*f).sizeupvalues as usize {
        // mark upvalue names
        let upvalue = *(*f).upvalues.add(i);
        if !upvalue.is_null() {
            stringmark!(upvalue);
        }
    }
    for i in 0..(*f).sizep as usize {
        // mark nested protos
        let p = *(*f).p.add(i);
        if !p.is_null() {
            markobject!(g, p);
        }
    }
    for i in 0..(*f).sizelocvars as usize {
        // mark local-variable names
        let varname = (*(*f).locvars.add(i)).varname;
        if !varname.is_null() {
            stringmark!(varname);
        }
    }
}
