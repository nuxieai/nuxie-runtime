use crate::enums::lua_type::lua_Type;
use crate::functions::freeobj::freeobj;
use crate::functions::lua_m_getpagewalkinfo::lua_m_getpagewalkinfo;
use crate::macros::bitmask::bitmask;
use crate::macros::fixedbit::FIXEDBIT;
use crate::macros::lua_c_white::luaC_white;
use crate::macros::maskmarks::maskmarks;
use crate::macros::otherwhite::otherwhite;
use crate::macros::testbit::testbit;
use crate::macros::whitebits::WHITEBITS;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::{c_char, c_int};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn sweepgcopage(l: *mut lua_State, page: *mut lua_Page) -> c_int {
    let mut start: *mut c_char = core::ptr::null_mut();
    let mut end: *mut c_char = core::ptr::null_mut();
    let mut busy_blocks: c_int = 0;
    let mut block_size: c_int = 0;
    lua_m_getpagewalkinfo(
        page,
        core::ptr::addr_of_mut!(start),
        core::ptr::addr_of_mut!(end),
        core::ptr::addr_of_mut!(busy_blocks),
        core::ptr::addr_of_mut!(block_size),
    );

    LUAU_ASSERT!(busy_blocks > 0);

    let g = (*l).global;
    let deadmask = otherwhite!(g);
    LUAU_ASSERT!(testbit!(deadmask, FIXEDBIT) != 0);

    let newwhite = luaC_white!(g);
    let mut pos = start;

    while pos != end {
        let gco = pos as *mut GCObject;

        if (*gco).gch.tt != lua_Type::LUA_TNIL as u8 {
            if (((*gco).gch.marked as i32 ^ WHITEBITS) & deadmask) != 0 {
                LUAU_ASSERT!(!crate::isdead!(g, gco));
                (*gco).gch.marked = (((*gco).gch.marked & maskmarks!()) | newwhite) as u8;
            } else {
                LUAU_ASSERT!(crate::isdead!(g, gco));
                freeobj(l, gco, page);

                busy_blocks -= 1;
                if busy_blocks == 0 {
                    return (pos.offset_from(start) as c_int) / block_size + 1;
                }
            }
        }

        pos = pos.add(block_size as usize);
    }

    (end.offset_from(start) as c_int) / block_size
}
