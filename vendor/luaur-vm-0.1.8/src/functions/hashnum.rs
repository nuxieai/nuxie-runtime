use crate::macros::gnode::gnode;
use crate::macros::lmod::lmod;
use crate::macros::sizenode::sizenode;
use crate::type_aliases::lua_node::LuaNode as LuaNodeAlias;
use crate::type_aliases::lua_table::LuaTable as LuaTableAlias;

#[allow(non_snake_case)]
pub unsafe fn hashnum(t: *mut LuaTableAlias, n: f64) -> *mut LuaNodeAlias {
    // static_assert(sizeof(double) == sizeof(unsigned int) * 2, "expected a 8-byte double");
    let mut i: [u32; 2] = [0; 2];
    core::ptr::copy_nonoverlapping(&n as *const f64 as *const u8, i.as_mut_ptr() as *mut u8, 8);

    // mask out sign bit to make sure -0 and 0 hash to the same value
    let h1 = i[0];
    let h2 = i[1] & 0x7fffffff;

    // finalizer from MurmurHash64B
    const M: u32 = 0x5bd1e995;

    let mut h1 = h1 ^ (h2 >> 18);
    h1 = h1.wrapping_mul(M);
    let mut h2 = h2 ^ (h1 >> 22);
    h2 = h2.wrapping_mul(M);
    let mut h1 = h1 ^ (h2 >> 17);
    h1 = h1.wrapping_mul(M);
    let h2 = h2 ^ (h1 >> 19);
    let h2 = h2.wrapping_mul(M);

    // ... truncated to 32-bit output (normally hash is equal to (uint64_t(h1) << 32) | h2, but we only really need the lower 32-bit half)
    gnode!(t, lmod!(h2 as i32, sizenode!(t)) as usize)
}
