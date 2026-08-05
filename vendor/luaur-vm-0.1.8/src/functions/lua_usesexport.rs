use crate::functions::index_2_addr::index2addr;
use crate::macros::clvalue::clvalue;
use crate::macros::ttisfunction::ttisfunction;
use crate::records::closure::LClosure;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::enums::luau_proto_flag::LuauProtoFlag;

pub unsafe fn lua_usesexport(L: *mut lua_State, idx: core::ffi::c_int) -> core::ffi::c_int {
    let value = index2addr(L, idx);
    if !ttisfunction!(value) || (*clvalue!(value)).isC != 0 {
        return 0;
    }

    let closure = clvalue!(value);
    let lua = core::ptr::addr_of!((*closure).inner.l).cast::<LClosure>();
    (((*(*lua).p).flags & LuauProtoFlag::LPF_USES_EXPORT as u8) != 0) as core::ffi::c_int
}
