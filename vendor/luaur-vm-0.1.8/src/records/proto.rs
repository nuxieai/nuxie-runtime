use crate::records::feedback_vector_slot::FeedbackVectorSlot;
use crate::records::g_cheader::GCheader;
use crate::records::gc_object::GCObject;
use crate::records::loc_var::LocVar;
use crate::records::t_string::TString;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::t_value::TValue;

#[allow(non_camel_case_types)]
#[derive(Debug)]
#[repr(C)]
pub struct Proto {
    pub hdr: GCheader,

    pub nups: u8,
    pub numparams: u8,
    pub is_vararg: u8,
    pub maxstacksize: u8,
    pub flags: u8,

    pub k: *mut TValue,
    pub code: *mut Instruction,
    pub p: *mut *mut Proto,
    pub codeentry: *const Instruction,

    pub execdata: *mut core::ffi::c_void,
    pub exectarget: usize,

    pub lineinfo: *mut u8,
    pub abslineinfo: *mut core::ffi::c_int,
    pub locvars: *mut LocVar,
    pub upvalues: *mut *mut TString,
    pub source: *mut TString,

    pub debugname: *mut TString,
    pub debuginsn: *mut u8,

    pub typeinfo: *mut u8,

    pub userdata: *mut core::ffi::c_void,

    pub gclist: *mut GCObject,

    pub sizecode: core::ffi::c_int,
    pub sizep: core::ffi::c_int,
    pub sizelocvars: core::ffi::c_int,
    pub sizeupvalues: core::ffi::c_int,
    pub sizek: core::ffi::c_int,
    pub sizelineinfo: core::ffi::c_int,
    pub linegaplog2: core::ffi::c_int,
    pub linedefined: core::ffi::c_int,
    pub bytecodeid: core::ffi::c_int,
    pub sizetypeinfo: core::ffi::c_int,

    pub feedbackvec: *mut FeedbackVectorSlot,
    pub feedbackvecsize: u32,
    pub funid: u32,
}

#[allow(non_camel_case_types)]
pub type proto = Proto;
