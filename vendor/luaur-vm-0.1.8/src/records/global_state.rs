#[allow(non_camel_case_types)]
#[repr(C)]
#[derive(Debug)]
pub struct global_State {
    pub strt: crate::records::stringtable::Stringtable,
    pub frealloc: crate::type_aliases::lua_alloc::lua_Alloc,
    pub ud: *mut core::ffi::c_void,
    pub currentwhite: u8,
    pub gcstate: u8,
    pub gray: *mut crate::records::gc_object::GcObject,
    pub grayagain: *mut crate::records::gc_object::GcObject,
    pub weak: *mut crate::records::gc_object::GcObject,
    pub GCthreshold: usize,
    pub totalbytes: usize,
    pub gcgoal: core::ffi::c_int,
    pub gcstepmul: core::ffi::c_int,
    pub gcstepsize: core::ffi::c_int,
    pub freepages: [*mut crate::records::lua_page::lua_Page; 40], // LUA_SIZECLASSES
    pub freegcopages: [*mut crate::records::lua_page::lua_Page; 40], // LUA_SIZECLASSES
    pub allpages: *mut crate::records::lua_page::lua_Page,
    pub allgcopages: *mut crate::records::lua_page::lua_Page,
    pub sweepgcopage: *mut crate::records::lua_page::lua_Page,
    pub mainthread: *mut crate::records::lua_state::LuaState,
    pub uvhead: crate::records::up_val::UpVal,
    pub mt: [*mut crate::records::lua_table::LuaTable; 14], // LUA_T_COUNT = LUA_TDEADKEY
    pub ttname: [*mut crate::records::t_string::TString; 16],
    pub tmname: [*mut crate::records::t_string::TString; 21],
    pub pseudotemp: crate::records::lua_t_value::TValue,
    pub registry: crate::records::lua_t_value::TValue,
    pub registryfree: core::ffi::c_int,
    pub errorjmp: *mut crate::records::lua_jmpbuf::lua_jmpbuf,
    pub rngstate: u64,
    pub ptrenckey: [u64; 4],
    pub cb: crate::records::lua_callbacks::LuaCallbacks,
    pub ecb: crate::records::lua_execution_callbacks::LuaExecutionCallbacks,
    pub ecbdata: crate::records::lua_execution_callback_storage::LuaExecutionCallbackStorage, // LUA_EXECUTION_CALLBACK_STORAGE
    pub udatadirect: [crate::records::lua_udata_direct_access_data::LuaUdataDirectAccessData; 130], // UTAG_INTERNAL_LIMIT
    pub memcatbytes: [usize; 256],
    pub udatagc: [Option<
        unsafe extern "C" fn(*mut crate::records::lua_state::LuaState, *mut core::ffi::c_void),
    >; 128],
    pub udatamt: [*mut crate::records::lua_table::LuaTable; 128],
    pub lightuserdataname: [*mut crate::records::t_string::TString; 128],
    pub udatadirectfields: [*mut crate::records::lua_table::LuaTable; 130], // UTAG_INTERNAL_LIMIT
    pub gcstats: crate::records::gc_stats::GCStats,
    pub lastprotoid: u32,
    #[cfg(feature = "luai_gcmetrics")]
    pub gcmetrics: crate::records::gc_metrics::GCMetrics,
}
