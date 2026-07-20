#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
#[repr(C)]
pub(crate) struct Decimal {
    pub(crate) s: u64,
    pub(crate) k: core::ffi::c_int,
}
