use crate::type_aliases::lua_node::LuaNode;
use crate::type_aliases::lua_table::LuaTable;

#[allow(non_snake_case)]
pub unsafe fn hashpointer(t: *const LuaTable, p: *const core::ffi::c_void) -> *mut LuaNode {
    // Discard high 32-bit portion on 64-bit platforms as it doesn't carry much entropy.
    let mut h: u32 = (p as usize) as u32;

    // MurmurHash3 32-bit finalizer
    h ^= h >> 16;
    h = h.wrapping_mul(0x85eb_ca6b_u32);
    h ^= h >> 13;
    h = h.wrapping_mul(0xc2b2_ae35_u32);
    h ^= h >> 16;

    crate::macros::gnode::gnode!(
        t,
        crate::macros::lmod::lmod!(h as i32, crate::macros::sizenode::sizenode!(t)) as usize
    )
}
