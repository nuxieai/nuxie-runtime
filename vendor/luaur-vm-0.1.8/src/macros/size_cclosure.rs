use crate::records::closure::{CClosure, Closure};
use crate::type_aliases::t_value::TValue;

// VM/src/lfunc.h:7 — #define sizeCclosure(n) (offsetof(Closure, c.upvals) + sizeof(TValue) * (n))
// ClosureInner fields sit at offset 0 of the union; ManuallyDrop is repr(transparent).
pub const fn size_cclosure(n: core::ffi::c_int) -> usize {
    core::mem::offset_of!(Closure, inner)
        + core::mem::offset_of!(CClosure, upvals)
        + core::mem::size_of::<TValue>() * n as usize
}
