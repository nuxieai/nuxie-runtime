use crate::type_aliases::call_info::CallInfo;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

use crate::functions::correctstack::correctstack;
use crate::functions::lua_d_throw_ldo::lua_d_throw;
use crate::functions::lua_m_realloc::lua_m_realloc_;
use crate::functions::lua_m_toobig::lua_m_toobig;
use crate::macros::cast_to::cast_to;
use crate::macros::extra_stack::EXTRA_STACK;
use crate::macros::max_stack_size::MAX_STACK_SIZE;
use crate::macros::setnilvalue::setnilvalue;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

#[allow(non_snake_case)]
pub unsafe fn luaD_reallocstack(
    L: *mut lua_State,
    newsize: core::ffi::c_int,
    fornewci: core::ffi::c_int,
) {
    // throw 'out of memory' error because space for a custom error message cannot be guaranteed here
    if newsize > MAX_STACK_SIZE {
        // reallocation was performed to setup a new CallInfo frame, which we have to remove
        if fornewci != 0 {
            let cip = (*L).ci.wrapping_offset(-1);

            (*L).ci = cip;
            (*L).base = (*cip).base;
            (*L).top = (*cip).top;
        }

        lua_d_throw(L, crate::enums::lua_status::lua_Status::LUA_ERRMEM as i32);
    }

    let oldstack = (*L).stack;
    let realsize = newsize + EXTRA_STACK;
    LUAU_ASSERT!(
        (*L).stack_last.offset_from((*L).stack) == ((*L).stacksize - EXTRA_STACK) as isize
    );

    let oldsize_bytes = (*L).stacksize as usize * core::mem::size_of::<TValue>();
    let newsize_bytes = if realsize as usize <= usize::MAX / core::mem::size_of::<TValue>() {
        realsize as usize * core::mem::size_of::<TValue>()
    } else {
        lua_m_toobig(L);
        usize::MAX
    };

    (*L).stack = cast_to!(
        *mut TValue,
        lua_m_realloc_(
            L,
            (*L).stack as *mut core::ffi::c_void,
            oldsize_bytes,
            newsize_bytes,
            (*L).activememcat as u8
        )
    );

    let newstack = (*L).stack;

    for i in (*L).stacksize as usize..realsize as usize {
        setnilvalue!(newstack.add(i));
    }

    (*L).stacksize = realsize;
    (*L).stack_last = newstack.add(newsize as usize);

    correctstack(L, oldstack);
}

#[allow(unused_imports)]
pub use luaD_reallocstack as lua_d_reallocstack;
