use crate::records::closure::{Closure, LClosure};
use crate::type_aliases::t_value::TValue;

// VM/src/lfunc.h:8 — #define sizeLclosure(n) (offsetof(Closure, l.uprefs) + sizeof(TValue) * (n))
// ClosureInner fields sit at offset 0 of the union; ManuallyDrop is repr(transparent).
#[inline]
pub const fn size_lclosure(n: usize) -> usize {
    core::mem::offset_of!(Closure, inner)
        + core::mem::offset_of!(LClosure, uprefs)
        + core::mem::size_of::<TValue>() * n
}
