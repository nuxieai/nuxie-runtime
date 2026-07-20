use crate::macros::lmod::lmod;
use crate::records::global_state::global_State;
use crate::records::lua_state::lua_State;
use crate::records::t_string::TString;

pub unsafe fn unlinkstr(l: *mut lua_State, ts: *mut TString) -> bool {
    let g: *mut global_State = (*l).global;

    let hash = (*ts).hash as usize;
    let size = (*g).strt.size as usize;
    let mut p: *mut *mut TString = (*g).strt.hash.add(lmod!(hash, size) as usize);

    while !(*p).is_null() {
        let curr = *p;
        if curr == ts {
            *p = (*curr).next;
            return true;
        } else {
            p = &mut (*curr).next as *mut *mut TString;
        }
    }

    false
}
