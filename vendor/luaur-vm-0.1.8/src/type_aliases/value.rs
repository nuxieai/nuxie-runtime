use crate::type_aliases::gc_object::GcObject;
use core::ffi::c_void;

#[repr(C)]
#[derive(Copy, Clone)]
pub union Value {
    pub gc: *mut GcObject,
    pub p: *mut c_void,
    pub n: f64,
    pub b: i32,
    pub l: i64,
    pub v: [f32; 2],
}

impl Default for Value {
    /// C aggregate zero-initialization (`Value v = {0}`): the all-zero bit
    /// pattern is valid for every member of the union.
    fn default() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
