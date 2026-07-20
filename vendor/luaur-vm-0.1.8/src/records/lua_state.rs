use crate::records::call_info::CallInfo;
use crate::records::g_cheader::GCheader;
use crate::records::gc_object::GcObject;
use crate::records::global_state::global_State;
use crate::records::lua_table::LuaTable;
use crate::records::t_string::TString;
use crate::records::up_val::UpVal;
use crate::type_aliases::stk_id::StkId;

#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct lua_State {
    pub hdr: GCheader,
    pub status: u8,
    pub activememcat: u8,
    pub isactive: bool,
    pub singlestep: bool,
    pub top: StkId,
    pub base: StkId,
    pub global: *mut global_State,
    pub ci: *mut CallInfo,
    pub stack_last: StkId,
    pub stack: StkId,
    pub end_ci: *mut CallInfo,
    pub base_ci: *mut CallInfo,
    pub stacksize: core::ffi::c_int,
    pub size_ci: core::ffi::c_int,
    pub nCcalls: u16,
    pub baseCcalls: u16,
    pub cachedslot: core::ffi::c_int,
    pub gt: *mut LuaTable,
    pub openupval: *mut UpVal,
    pub gclist: *mut GcObject,
    pub namecall: *mut TString,
    pub userdata: *mut core::ffi::c_void,
}

#[allow(non_camel_case_types)]
pub type LuaState = lua_State;
