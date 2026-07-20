//! Node: `cxx:Function:Luau.VM:VM/src/lstate.cpp:184:lua_newstate`
//! Source: `VM/src/lstate.cpp:184-290` (hand-ported)

use crate::enums::lua_type::lua_Type;
use crate::functions::close_state::close_state;
use crate::functions::f_luaopen::f_luaopen;
use crate::functions::lua_d_rawrunprotected_ldo::luaD_rawrunprotected;
use crate::functions::preinit_state::preinit_state;
use crate::macros::bit_2_mask::bit2mask;
use crate::macros::fixedbit::FIXEDBIT;
use crate::macros::lua_lutag_limit::LUA_LUTAG_LIMIT;
use crate::macros::lua_memory_categories::LUA_MEMORY_CATEGORIES;
use crate::macros::lua_sizeclasses::LUA_SIZECLASSES;
use crate::macros::lua_utag_limit::LUA_UTAG_LIMIT;
use crate::macros::registry::registry;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::utag_internal_limit::UTAG_INTERNAL_LIMIT;
use crate::macros::white_0_bit::WHITE0BIT;
use crate::records::lg::LG;
use crate::type_aliases::lua_alloc::lua_Alloc;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_assert::LUAU_ASSERT as _;

#[allow(non_snake_case)]
pub unsafe fn lua_newstate(f: lua_Alloc, ud: *mut core::ffi::c_void) -> *mut lua_State {
    let Some(falloc) = f else {
        return core::ptr::null_mut();
    };
    let l = falloc(ud, core::ptr::null_mut(), 0, core::mem::size_of::<LG>());
    if l.is_null() {
        return core::ptr::null_mut();
    }
    let L = l as *mut lua_State;
    let g = &mut (*(l as *mut LG)).g as *mut crate::records::global_state::global_State;

    (*L).hdr.tt = lua_Type::LUA_TTHREAD as u8;
    (*g).currentwhite = bit2mask(WHITE0BIT, FIXEDBIT) as u8;
    (*L).hdr.marked = (*g).currentwhite;
    (*L).hdr.memcat = 0;
    preinit_state(L, g);
    (*g).frealloc = f;
    (*g).ud = ud;
    (*g).mainthread = L;
    (*g).uvhead.u.open.prev = &mut (*g).uvhead;
    (*g).uvhead.u.open.next = &mut (*g).uvhead;
    (*g).GCthreshold = 0; // mark it as unfinished state
    (*g).registryfree = 0;
    (*g).errorjmp = core::ptr::null_mut();
    (*g).rngstate = 0;
    (*g).ptrenckey[0] = 1;
    (*g).ptrenckey[1] = 0;
    (*g).ptrenckey[2] = 0;
    (*g).ptrenckey[3] = 0;
    (*g).strt.size = 0;
    (*g).strt.nuse = 0;
    (*g).strt.hash = core::ptr::null_mut();
    setnilvalue!(&mut (*g).pseudotemp as *mut TValue);
    setnilvalue!(registry!(L) as *const TValue as *mut TValue);
    (*g).gcstate = crate::macros::gc_spause::GCSpause as u8;
    (*g).gray = core::ptr::null_mut();
    (*g).grayagain = core::ptr::null_mut();
    (*g).weak = core::ptr::null_mut();
    (*g).totalbytes = core::mem::size_of::<LG>();
    (*g).gcgoal = crate::macros::luai_gcgoal::LUAI_GCGOAL;
    (*g).gcstepmul = crate::macros::luai_gcstepmul::LUAI_GCSTEPMUL;
    (*g).gcstepsize = crate::macros::luai_gcstepsize::LUAI_GCSTEPSIZE << 10;

    for i in 0..LUA_SIZECLASSES as usize {
        (*g).freepages[i] = core::ptr::null_mut();
        (*g).freegcopages[i] = core::ptr::null_mut();
    }

    (*g).allpages = core::ptr::null_mut();
    (*g).allgcopages = core::ptr::null_mut();
    (*g).sweepgcopage = core::ptr::null_mut();

    for i in 0..(*g).mt.len() {
        (*g).mt[i] = core::ptr::null_mut();
    }

    for i in 0..LUA_UTAG_LIMIT as usize {
        (*g).udatagc[i] = None;
        (*g).udatamt[i] = core::ptr::null_mut();
    }

    for i in 0..UTAG_INTERNAL_LIMIT as usize {
        let udatadirect = &mut (*g).udatadirect[i];

        setnilvalue!(&mut udatadirect.indextm as *mut TValue);
        setnilvalue!(&mut udatadirect.newindextm as *mut TValue);
        setnilvalue!(&mut udatadirect.namecalltm as *mut TValue);
        udatadirect.index = None;
        udatadirect.newindex = None;
        udatadirect.namecall = None;
    }

    for i in 0..LUA_LUTAG_LIMIT as usize {
        (*g).lightuserdataname[i] = core::ptr::null_mut();
    }

    if luaur_common::FFlag::LuauDirectFieldGet.get() {
        for i in 0..UTAG_INTERNAL_LIMIT as usize {
            (*g).udatadirectfields[i] = core::ptr::null_mut();
        }
    }

    for i in 0..LUA_MEMORY_CATEGORIES as usize {
        (*g).memcatbytes[i] = 0;
    }

    (*g).memcatbytes[0] = core::mem::size_of::<LG>();

    (*g).cb = core::mem::zeroed(); // lua_Callbacks()
    (*g).ecb = core::mem::zeroed(); // lua_ExecutionCallbacks()

    (*g).ecbdata = core::mem::zeroed();

    (*g).gcstats = Default::default(); // GCStats()
    (*g).lastprotoid = 1;

    if luaD_rawrunprotected(L, Some(f_luaopen), core::ptr::null_mut()) != 0 {
        // memory allocation error: free partial state
        close_state(L);
        return core::ptr::null_mut();
    }

    luaur_common::LUAU_ASSERT!((*g).GCthreshold != 0);
    L
}
