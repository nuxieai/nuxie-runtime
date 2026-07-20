use crate::type_aliases::buffer::Buffer;

#[allow(non_snake_case)]
#[inline]
pub const fn sizebuffer(len: usize) -> usize {
    core::mem::offset_of!(Buffer, data) + if len < 8 { 8 } else { len }
}
