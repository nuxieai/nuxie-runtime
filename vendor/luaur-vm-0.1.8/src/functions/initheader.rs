use crate::records::header::Header;
use crate::records::lua_state::lua_State;

#[inline]
pub fn initheader(L: *mut lua_State, h: *mut Header) {
    unsafe {
        (*h).L = L;
        (*h).islittle = 1; // nativeendian.little is assumed to be 1 (true) for little-endian systems
        (*h).maxalign = 1;
    }
}
