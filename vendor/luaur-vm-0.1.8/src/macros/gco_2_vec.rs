#[macro_export]
macro_rules! gco2vec {
    ($o:expr) => {{
        luaur_common::LUAU_ASSERT!(
            (*($o)).gch.tt == $crate::enums::lua_type::lua_Type::LUA_TVECTOR as u8
        );
        core::ptr::addr_of_mut!((*($o)).vec) as *mut $crate::records::luau_vector::LuauVector
    }};
}

pub use gco2vec;
