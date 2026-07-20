use crate::records::udata::Udata;

#[allow(non_snake_case)]
#[inline]
pub const fn sizeudata(len: usize) -> usize {
    core::mem::offset_of!(Udata, data) + if len > 16 { (len + 15) & !15 } else { len }
}
