use crate::functions::loadsafe::loadsafe;
use crate::records::load_context::LoadContext;
use crate::type_aliases::lua_state::lua_State;

pub trait LoadContextRun {
    unsafe fn run(
        L: *mut lua_State,
        ud: *mut core::ffi::c_void,
    ) -> Result<(), crate::records::lua_exception::lua_exception>;
}

impl LoadContextRun for LoadContext {
    unsafe fn run(
        L: *mut lua_State,
        ud: *mut core::ffi::c_void,
    ) -> Result<(), crate::records::lua_exception::lua_exception> {
        let ctx = ud as *mut LoadContext;

        (*ctx).result = loadsafe(
            L,
            &mut (*ctx).strings,
            &mut (*ctx).protos,
            (*ctx).chunkname,
            (*ctx).data,
            (*ctx).size,
            (*ctx).env,
        )?;
        Ok(())
    }
}
