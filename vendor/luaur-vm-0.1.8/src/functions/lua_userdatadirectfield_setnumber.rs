use crate::macros::setnvalue::setnvalue;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[export_name = "luaur_lua_userdatadirectfield_setnumber"]
pub unsafe fn lua_userdatadirectfield_setnumber(result: *mut core::ffi::c_void, n: f64) {
    LUAU_ASSERT!(luaur_common::FFlag::LuauDirectFieldGet.get());
    #[cfg(feature = "lua_vector_double")]
    let slot = (*(result as *mut crate::records::direct_field_result::DirectFieldResult)).slot;
    #[cfg(not(feature = "lua_vector_double"))]
    let slot = result as *mut TValue;
    setnvalue!(slot, n);
}
