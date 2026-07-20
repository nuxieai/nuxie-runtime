use crate::functions::lua_v_getimport::lua_v_getimport;
use crate::macros::lua_d_checkstack::luaD_checkstack;
use crate::macros::setnilvalue::setnilvalue;
use crate::records::resolve_import::ResolveImport;
use crate::type_aliases::lua_state::lua_State;

impl ResolveImport {
    pub unsafe fn run(L: *mut lua_State, ud: *mut core::ffi::c_void) {
        let self_ = ud as *mut ResolveImport;

        // note: we call getimport with nil propagation which means that accesses to table chains like A.B.C will resolve in nil
        // this is technically not necessary but it reduces the number of exceptions when loading scripts that rely on getfenv/setfenv for global
        // injection
        // allocate a stack slot so that we can do table lookups
        luaD_checkstack!(L, 1);
        setnilvalue!((*L).top);
        (*L).top = (*L).top.add(1);

        lua_v_getimport(
            L,
            (*L).gt,
            (*self_).k,
            (*L).top.sub(1),
            (*self_).id,
            true, /* propagatenil= */
        );
    }
}
