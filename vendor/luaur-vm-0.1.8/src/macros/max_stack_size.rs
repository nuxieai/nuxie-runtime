use crate::type_aliases::t_value::TValue;

pub const MAX_STACK_SIZE: core::ffi::c_int =
    (1024 / core::mem::size_of::<TValue>() as core::ffi::c_int) * 1024 * 1024;
