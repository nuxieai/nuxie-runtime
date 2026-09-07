//! Regression for rive_0_734's unconditional tagged-userdata metatable roots.
//! Mirrors Conformance.test.cpp's switch from luaL_newmetatable to lua_newtable:
//! the userdata-tag registry must be sufficient to keep the table alive.

#[test]
fn unregistered_tagged_metatable_survives_full_collection() {
    unsafe {
        let state = crate::functions::lua_l_newstate::lua_l_newstate();
        assert!(!state.is_null());
        crate::functions::lua_createtable::lua_createtable(state, 0, 1);
        crate::functions::lua_pushnumber::lua_pushnumber(state, 734.0);
        crate::functions::lua_setfield::lua_setfield(state, -2, c"marker".as_ptr());
        // This pops the table. No stack or named registry reference remains.
        crate::functions::lua_setuserdatametatable::lua_setuserdatametatable(state, 7);
        crate::functions::lua_c_fullgc::lua_c_fullgc(state);
        crate::functions::lua_c_fullgc::lua_c_fullgc(state);
        crate::functions::lua_getuserdatametatable::lua_getuserdatametatable(state, 7);
        crate::functions::lua_getfield::lua_getfield(state, -1, c"marker".as_ptr());
        assert_eq!(
            crate::functions::lua_tonumberx::lua_tonumberx(state, -1, core::ptr::null_mut()),
            734.0
        );
        crate::functions::lua_close::lua_close(state);
    }
}
