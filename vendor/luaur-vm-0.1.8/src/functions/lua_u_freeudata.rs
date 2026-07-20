use crate::functions::lua_m_freegco::luaM_freegco_;
use crate::macros::lua_m_freegco::luaM_freegco;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::macros::obj_2_gco::obj2gco;
use crate::macros::sizeudata::sizeudata;
use crate::macros::utag_idtor::UTAG_IDTOR;
use crate::records::gc_object::GCObject;
use crate::records::lua_page::lua_Page;
use crate::records::udata::Udata;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_void;

type UserdataDtor = Option<unsafe extern "C" fn(*mut c_void)>;

#[allow(non_snake_case)]
pub unsafe fn lua_u_freeudata(L: *mut lua_State, u: *mut Udata, page: *mut lua_Page) {
    let tag = (*u).tag as i32;

    if tag < LUA_UTAG_LIMIT {
        let dtor = (*(*L).global).udatagc[tag as usize];
        if let Some(dtor_fn) = dtor {
            dtor_fn(L, (*u).data.as_mut_ptr().cast());
        }
    } else if tag == UTAG_IDTOR {
        let len = (*u).len as usize;
        let dtor_size = core::mem::size_of::<UserdataDtor>();
        let dtor_addr = (*u).data.as_ptr().add(len - dtor_size);
        let mut dtor = core::mem::MaybeUninit::<UserdataDtor>::uninit();
        core::ptr::copy_nonoverlapping(
            dtor_addr.cast::<u8>(),
            dtor.as_mut_ptr().cast::<u8>(),
            dtor_size,
        );
        let dtor = dtor.assume_init();

        if let Some(dtor_fn) = dtor {
            dtor_fn((*u).data.as_mut_ptr().cast());
        }
    }

    luaM_freegco_(
        L,
        u as *mut GCObject,
        sizeudata((*u).len as usize),
        (*u).memcat,
        page,
    );
}
