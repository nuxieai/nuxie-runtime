use core::ffi::{c_int, c_void};

use crate::enums::lua_type::lua_Type;
use crate::functions::lua_concat::lua_c_threadbarrier_lapi;
use crate::functions::lua_u_newudata::lua_u_newudata;
use crate::macros::api_check::api_check;
use crate::macros::api_incr_top::api_incr_top;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::utag_idtor::UTAG_IDTOR;
use crate::records::gc_object::GCObject;
use crate::records::lua_state::lua_State;

type UserdataDtor = Option<unsafe extern "C" fn(*mut c_void)>;

#[allow(non_snake_case)]
pub unsafe fn lua_newuserdatadtor(L: *mut lua_State, sz: usize, dtor: UserdataDtor) -> *mut c_void {
    api_check!(L, dtor.is_some());
    luaC_checkGC!(L);
    lua_c_threadbarrier_lapi(L);

    let dtor_size = core::mem::size_of::<UserdataDtor>();
    let as_ = if sz < usize::MAX - dtor_size {
        sz + dtor_size
    } else {
        usize::MAX
    };

    let u = lua_u_newudata(L, as_, UTAG_IDTOR);
    core::ptr::copy_nonoverlapping(
        (&dtor as *const UserdataDtor).cast::<u8>(),
        (*u).data.as_mut_ptr().add(sz).cast::<u8>(),
        dtor_size,
    );

    (*(*L).top).value.gc = u as *mut GCObject;
    (*(*L).top).tt = lua_Type::LUA_TUSERDATA as c_int;
    crate::macros::checkliveness::checkliveness!((*L).global, (*L).top);
    api_incr_top!(L);

    (*u).data.as_mut_ptr().cast()
}
