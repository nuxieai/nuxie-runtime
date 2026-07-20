use crate::functions::coclose::coclose;
use crate::functions::cocreate::cocreate;
use crate::functions::coresumecont::coresumecont;
use crate::functions::coresumey::coresumey;
use crate::functions::corunning::corunning;
use crate::functions::costatus::costatus;
use crate::functions::cowrap::cowrap;
use crate::functions::coyield::coyield;
use crate::functions::coyieldable::coyieldable;
use crate::functions::lua_l_register::lua_l_register;
use crate::functions::lua_pushcclosurek::lua_pushcclosurek;
use crate::functions::lua_setfield::lua_setfield;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_int;

pub unsafe fn luaopen_coroutine(l: *mut lua_State) -> c_int {
    lua_l_register(l, c"coroutine".as_ptr(), CO_FUNCS.0.as_ptr());

    lua_pushcclosurek(
        l,
        Some(coresumey),
        c"resume".as_ptr(),
        0,
        Some(coresumecont),
    );
    lua_setfield(l, -2, c"resume".as_ptr());

    1
}

struct SyncLuaLReg([LuaLReg; 8]);
unsafe impl Sync for SyncLuaLReg {}

static CO_FUNCS: SyncLuaLReg = SyncLuaLReg([
    LuaLReg {
        name: c"create".as_ptr(),
        func: Some(cocreate),
    },
    LuaLReg {
        name: c"running".as_ptr(),
        func: Some(corunning),
    },
    LuaLReg {
        name: c"status".as_ptr(),
        func: Some(costatus),
    },
    LuaLReg {
        name: c"wrap".as_ptr(),
        func: Some(cowrap),
    },
    LuaLReg {
        name: c"yield".as_ptr(),
        func: Some(coyield),
    },
    LuaLReg {
        name: c"isyieldable".as_ptr(),
        func: Some(coyieldable),
    },
    LuaLReg {
        name: c"close".as_ptr(),
        func: Some(coclose),
    },
    LuaLReg {
        name: core::ptr::null(),
        func: None,
    },
]);

impl core::ops::Deref for SyncLuaLReg {
    type Target = [LuaLReg; 8];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
