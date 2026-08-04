use crate::macros::setvvalue::setvvalue;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::lua_vector_type::LuaVectorType;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
#[export_name = "luaur_lua_userdatadirectfield_setvector_void_f32_f32_f32"]
pub unsafe fn lua_userdatadirectfield_setvector_void_f32_f32_f32(
    result: *mut core::ffi::c_void,
    x: LuaVectorType,
    y: LuaVectorType,
    z: LuaVectorType,
) {
    LUAU_ASSERT!(luaur_common::FFlag::LuauDirectFieldGet.get());
    #[cfg(feature = "lua_vector_double")]
    {
        let dfr = result as *mut crate::records::direct_field_result::DirectFieldResult;
        setvvalue!((*dfr).l, (*dfr).slot, x, y, z, 0.0);
    }
    #[cfg(not(feature = "lua_vector_double"))]
    setvvalue!(
        core::ptr::null_mut::<crate::records::lua_state::lua_State>(),
        result as *mut TValue,
        x,
        y,
        z,
        0.0
    );
}
