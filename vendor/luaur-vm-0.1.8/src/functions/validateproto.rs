//! Node: `cxx:Function:Luau.VM:VM/src/lgcdebug.cpp:115:validateproto`
//!
//! GC heap-validation: assert that every GC reference reachable from a `Proto`
//! (source, debugname, constants, upvalues, child protos, local-var names)
//! points at a live, correctly-colored object.

use crate::functions::validateobjref::validateobjref;
use crate::functions::validateref::validateref;
use crate::macros::obj_2_gco::obj2gco;
use crate::records::global_state::global_State;
use crate::records::proto::Proto;

#[allow(non_snake_case)]
pub fn validateproto(g: *mut global_State, f: *mut Proto) {
    unsafe {
        if !(*f).source.is_null() {
            validateobjref(g, obj2gco!(f), obj2gco!((*f).source));
        }

        if !(*f).debugname.is_null() {
            validateobjref(g, obj2gco!(f), obj2gco!((*f).debugname));
        }

        for i in 0..(*f).sizek {
            validateref(g, obj2gco!(f), (*f).k.add(i as usize));
        }

        for i in 0..(*f).sizeupvalues {
            let up = *(*f).upvalues.add(i as usize);
            if !up.is_null() {
                validateobjref(g, obj2gco!(f), obj2gco!(up));
            }
        }

        for i in 0..(*f).sizep {
            let p = *(*f).p.add(i as usize);
            if !p.is_null() {
                validateobjref(g, obj2gco!(f), obj2gco!(p));
            }
        }

        for i in 0..(*f).sizelocvars {
            let lv = (*f).locvars.add(i as usize);
            if !(*lv).varname.is_null() {
                validateobjref(g, obj2gco!(f), obj2gco!((*lv).varname));
            }
        }
    }
}
