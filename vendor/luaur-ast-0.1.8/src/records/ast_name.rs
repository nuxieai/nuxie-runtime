use core::ffi::CStr;

/// Port of `Luau::AstName` (Ast/include/Luau/Ast.h:21-60).
///
/// `value` is an interned C string owned by `AstNameTable`; equality/hash are
/// pointer identity (reference `operator==(const AstName&)`), while ordering is
/// content-based `strcmp` with a pointer fallback when either side is null
/// (reference `operator<`). The two are consistent under the interning
/// invariant: equal content from the same table implies the same pointer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstName {
    pub value: *const core::ffi::c_char,
}

impl PartialOrd for AstName {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AstName {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if !self.value.is_null() && !other.value.is_null() {
            unsafe { CStr::from_ptr(self.value).cmp(CStr::from_ptr(other.value)) }
        } else {
            self.value.cmp(&other.value)
        }
    }
}
