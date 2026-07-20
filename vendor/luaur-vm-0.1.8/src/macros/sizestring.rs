use crate::records::t_string::TString;

#[allow(non_snake_case)]
#[inline]
pub const fn sizestring(len: usize) -> usize {
    core::mem::offset_of!(TString, data) + len + 1
}
