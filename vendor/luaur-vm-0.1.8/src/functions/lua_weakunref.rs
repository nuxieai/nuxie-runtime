use crate::records::lua_state::lua_State;

pub unsafe fn lua_weakunref(l: *mut lua_State, reference: core::ffi::c_int) -> core::ffi::c_int {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    let g = (*l).global;
    crate::functions::registryunref::registryunref(
        l,
        reference,
        core::ptr::addr_of_mut!((*g).weakregistry),
        core::ptr::addr_of_mut!((*g).weakregistryfree),
    );
    crate::macros::lua_noref::LUA_NOREF
}
