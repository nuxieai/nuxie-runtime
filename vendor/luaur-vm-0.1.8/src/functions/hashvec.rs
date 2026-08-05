use crate::macros::lua_vector_size::LUA_VECTOR_SIZE;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_vector_type::LuaVectorType;

#[allow(non_snake_case)]
pub unsafe fn hashvec(t: *const LuaTable, v: *const LuaVectorType) -> *mut LuaNode {
    #[cfg(not(feature = "lua_vector_double"))]
    let h = {
        let mut i = [0u32; 4];

        core::ptr::copy_nonoverlapping(v as *const u32, i.as_mut_ptr(), LUA_VECTOR_SIZE as usize);

    i[0] = if i[0] == 0x80000000 { 0 } else { i[0] };
    i[1] = if i[1] == 0x80000000 { 0 } else { i[1] };
    i[2] = if i[2] == 0x80000000 { 0 } else { i[2] };

    i[0] ^= i[0] >> 17;
    i[1] ^= i[1] >> 17;
    i[2] ^= i[2] >> 17;

        let mut h = (i[0].wrapping_mul(73856093))
            ^ (i[1].wrapping_mul(19349663))
            ^ (i[2].wrapping_mul(83492791));

        if LUA_VECTOR_SIZE == 4 {
            i[3] = if i[3] == 0x80000000 { 0 } else { i[3] };
            i[3] ^= i[3] >> 17;
            h ^= i[3].wrapping_mul(39916801);
        }
        h
    };

    #[cfg(feature = "lua_vector_double")]
    let h = {
        let mut i = [0u64; 4];
        core::ptr::copy_nonoverlapping(v as *const u64, i.as_mut_ptr(), LUA_VECTOR_SIZE as usize);
        for lane in &mut i[..3] {
            if *lane == 0x8000000000000000 {
                *lane = 0;
            }
            *lane ^= *lane >> 32;
        }
        let mut h = i[0].wrapping_mul(73856093) as u32
            ^ i[1].wrapping_mul(19349663) as u32
            ^ i[2].wrapping_mul(83492791) as u32;
        if LUA_VECTOR_SIZE == 4 {
            if i[3] == 0x8000000000000000 {
                i[3] = 0;
            }
            i[3] ^= i[3] >> 32;
            h ^= i[3].wrapping_mul(39916801) as u32;
        }
        h
    };

    let size = 1i32 << (*t).lsizenode;
    let index = (h as i32) & (size - 1);
    (*t).node.add(index as usize)
}
