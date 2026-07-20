use crate::enums::lua_type::lua_Type;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use core::ffi::{c_char, c_int, c_void};
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn lua_m_visitpage(
    page: *mut lua_Page,
    context: *mut c_void,
    visitor: *mut core::ffi::c_void, // function pointer: bool (*)(void* context, lua_Page* page, GCObject* gco)
) {
    let mut start: *mut c_char = core::ptr::null_mut();
    let mut end: *mut c_char = core::ptr::null_mut();
    let mut busy_blocks: c_int = 0;
    let mut block_size: c_int = 0;

    crate::functions::lua_m_getpagewalkinfo::lua_m_getpagewalkinfo(
        page,
        &mut start,
        &mut end,
        &mut busy_blocks,
        &mut block_size,
    );

    let mut pos = start;
    while pos != end {
        let gco = pos as *mut GCObject;

        // skip memory blocks that are already freed
        if (*gco).gch.tt == lua_Type::LUA_TNIL as u8 {
            pos = pos.add(block_size as usize);
            continue;
        }

        // when true is returned it means that the element was deleted
        let visitor_fn: extern "C" fn(*mut c_void, *mut lua_Page, *mut GCObject) -> bool =
            core::mem::transmute(visitor);
        if visitor_fn(context, page, gco) {
            LUAU_ASSERT!(busy_blocks > 0);

            // if the last block was removed, page would be removed as well
            busy_blocks -= 1;
            if busy_blocks == 0 {
                break;
            }
        }

        pos = pos.add(block_size as usize);
    }
}
