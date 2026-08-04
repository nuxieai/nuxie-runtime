use crate::records::lua_state::lua_State;

pub unsafe fn lua_weakref(l: *mut lua_State, idx: core::ffi::c_int) -> core::ffi::c_int {
    luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauGcTraceUdata.get());
    let g = (*l).global;
    crate::functions::registryref::registryref(
        l,
        idx,
        core::ptr::addr_of_mut!((*g).weakregistry),
        core::ptr::addr_of_mut!((*g).weakregistryfree),
    )
}
