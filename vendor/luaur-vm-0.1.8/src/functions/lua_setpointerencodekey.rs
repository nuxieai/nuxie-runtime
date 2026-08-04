use crate::records::lua_state::lua_State;

pub unsafe fn lua_setpointerencodekey(l: *mut lua_State, a: u64, b: u64, c: u64, d: u64) {
    let g = (*l).global;
    (*g).ptrenckey[0] = a & !1;
    (*g).ptrenckey[1] = b | 1;
    (*g).ptrenckey[2] = c;
    (*g).ptrenckey[3] = d;
}
