/* Copyright 2023 Rive */
//! Opt-in hierarchical RTTI. Rust base subobjects retain a predicate slot
//! corresponding to the upstream virtual canRTTICastTo method. Cast pointer
//! adjustment remains an explicit per-related-type contract.
#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]
use super::super::rive::refcnt_hpp::{RefCntTarget, rcp};
use core::marker::PhantomData;
#[inline(always)]
pub const fn CONST_ID(value: &str) -> u32 {
    let bytes = value.as_bytes();
    let mut crc = 0xffff_ffffu32;
    let mut index = 0usize;
    while index < bytes.len() {
        let mut byte = bytes[index] as u32;
        let mut bit = 0;
        while bit < 8 {
            let mix = (crc ^ byte) & 1;
            crc >>= 1;
            if mix != 0 {
                crc ^= 0xedb8_8320;
            }
            byte >>= 1;
            bit += 1;
        }
        index += 1;
    }
    crc ^ 0xffff_ffff
}

pub trait LiteRttiTypeId {
    const LITE_RTTI_TYPE_ID: u32;
    const TypeIDForRTTI: u32 = Self::LITE_RTTI_TYPE_ID;
    fn can_rtti_cast_to(id: u32) -> bool {
        id == Self::LITE_RTTI_TYPE_ID
    }
}
pub trait LiteRttiBase {
    fn canRTTICastTo(&self, id: u32) -> bool;
    fn setRttiPredicate(&mut self, predicate: fn(u32) -> bool);
}
#[repr(C)]
pub struct enable_lite_rtti<Root, const ID: u32> {
    predicate: fn(u32) -> bool,
    _root: PhantomData<fn() -> Root>,
}
impl<Root, const ID: u32> enable_lite_rtti<Root, ID> {
    pub const TypeIDForRTTI: u32 = ID;
    pub const fn new() -> Self {
        Self {
            predicate: |_| false,
            _root: PhantomData,
        }
    }
}
impl<Root, const ID: u32> Default for enable_lite_rtti<Root, ID> {
    fn default() -> Self {
        Self::new()
    }
}
impl<Root, const ID: u32> LiteRttiTypeId for enable_lite_rtti<Root, ID> {
    const LITE_RTTI_TYPE_ID: u32 = ID;
    fn can_rtti_cast_to(_: u32) -> bool {
        false
    }
}
impl<Root, const ID: u32> LiteRttiBase for enable_lite_rtti<Root, ID> {
    fn canRTTICastTo(&self, id: u32) -> bool {
        (self.predicate)(id)
    }
    fn setRttiPredicate(&mut self, predicate: fn(u32) -> bool) {
        self.predicate = predicate;
    }
}
#[repr(C)]
pub struct lite_rtti_override<Base: LiteRttiBase + LiteRttiTypeId, Derived, const ID: u32> {
    base: Base,
    _derived: PhantomData<fn() -> Derived>,
}
impl<Base: LiteRttiBase + LiteRttiTypeId, Derived, const ID: u32>
    lite_rtti_override<Base, Derived, ID>
{
    pub const TypeIDForRTTI: u32 = ID;
    pub fn new(mut base: Base) -> Self {
        base.setRttiPredicate(Self::can_rtti_cast_to);
        Self {
            base,
            _derived: PhantomData,
        }
    }
    pub fn from_args(constructor: impl FnOnce() -> Base) -> Self {
        Self::new(constructor())
    }
    pub fn base(&self) -> &Base {
        &self.base
    }
    pub fn base_mut(&mut self) -> &mut Base {
        &mut self.base
    }
}
impl<Base: LiteRttiBase + LiteRttiTypeId + Default, Derived, const ID: u32> Default
    for lite_rtti_override<Base, Derived, ID>
{
    fn default() -> Self {
        Self::new(Base::default())
    }
}
impl<Base: LiteRttiBase + LiteRttiTypeId, Derived, const ID: u32> LiteRttiTypeId
    for lite_rtti_override<Base, Derived, ID>
{
    const LITE_RTTI_TYPE_ID: u32 = ID;
    fn can_rtti_cast_to(id: u32) -> bool {
        ID == id || Base::can_rtti_cast_to(id)
    }
}
impl<Base: LiteRttiBase + LiteRttiTypeId, Derived, const ID: u32> LiteRttiBase
    for lite_rtti_override<Base, Derived, ID>
{
    fn canRTTICastTo(&self, id: u32) -> bool {
        self.base.canRTTICastTo(id)
    }
    fn setRttiPredicate(&mut self, predicate: fn(u32) -> bool) {
        self.base.setRttiPredicate(predicate);
    }
}
/// A downcast between related types, with an explicit subobject adjustment.
/// Implementations must not be supplied for self-casts or upcasts.
pub trait LiteRttiCastFrom<Base>: LiteRttiTypeId {
    /// # Safety
    /// `base` is a live Base subobject whose dynamic predicate accepts Self.
    unsafe fn from_base(base: *mut Base) -> *mut Self;
}
impl<Base: LiteRttiBase + LiteRttiTypeId, Derived, const ID: u32> LiteRttiCastFrom<Base>
    for lite_rtti_override<Base, Derived, ID>
{
    unsafe fn from_base(base: *mut Base) -> *mut Self {
        base.cast()
    }
}
/// # Safety
/// Non-null input must be a live related base subobject with its installed
/// complete-owner predicate and the cast adjustment contract above.
pub unsafe fn lite_rtti_cast<U: LiteRttiCastFrom<T>, T: LiteRttiBase>(src: *mut T) -> *mut U {
    if !src.is_null() && unsafe { (&*src).canRTTICastTo(U::LITE_RTTI_TYPE_ID) } {
        unsafe { U::from_base(src) }
    } else {
        core::ptr::null_mut()
    }
}
/// Const-preserving counterpart; immutable inputs never produce mutable pointers.
/// # Safety
/// Same live base-subobject contract as lite_rtti_cast.
pub unsafe fn lite_rtti_cast_const<U: LiteRttiCastFrom<T>, T: LiteRttiBase>(
    src: *const T,
) -> *const U {
    unsafe { lite_rtti_cast::<U, T>(src.cast_mut()) }.cast_const()
}
/// # Safety
/// Same live base-subobject contract as lite_rtti_cast.
pub unsafe fn lite_rtti_cast_or_assert<U: LiteRttiCastFrom<T>, T: LiteRttiBase>(
    src: *mut T,
) -> *mut U {
    let dst = unsafe { lite_rtti_cast::<U, T>(src) };
    debug_assert!(!dst.is_null());
    dst
}
/// # Safety
/// Same base-subobject contract, including complete-object reference counting.
pub unsafe fn lite_rtti_rcp_cast<U, T>(mut src: rcp<T>) -> rcp<U>
where
    U: LiteRttiCastFrom<T> + RefCntTarget,
    T: LiteRttiBase + RefCntTarget,
{
    if !src.get().is_null() && unsafe { (&*src.get()).canRTTICastTo(U::LITE_RTTI_TYPE_ID) } {
        let base = src.release();
        unsafe { rcp::from_ptr(U::from_base(base)) }
    } else {
        rcp::new()
    }
}

// Different versions of clang-format disagree on how to formate these.
// clang-format off

// #define ENABLE_LITE_RTTI(ROOT) enable_lite_rtti<ROOT, CONST_ID(ROOT)>
#[macro_export]
macro_rules! ENABLE_LITE_RTTI {
    ($root:ty) => {
        $crate::mechanical_port::source::include::utils::lite_rtti_hpp::enable_lite_rtti<
            $root,
            { $crate::mechanical_port::source::include::utils::lite_rtti_hpp::CONST_ID(stringify!($root)) },
        >
    };
}

// #define LITE_RTTI_OVERRIDE(BASE, DERRIVED) lite_rtti_override<BASE, DERRIVED, CONST_ID(DERRIVED)>
#[macro_export]
macro_rules! LITE_RTTI_OVERRIDE {
    ($base:ty, $derrived:ty) => {
        $crate::mechanical_port::source::include::utils::lite_rtti_hpp::lite_rtti_override<
            $base,
            $derrived,
            { $crate::mechanical_port::source::include::utils::lite_rtti_hpp::CONST_ID(stringify!($derrived)) },
        >
    };
}

// #define LITE_RTTI_CAST_OR_RETURN(NAME, TYPE, POINTER)                              \
//     auto NAME = rive::lite_rtti_cast<TYPE>(POINTER);                                \
//     if (NAME == nullptr)                                                            \
//         return
#[macro_export]
macro_rules! LITE_RTTI_CAST_OR_RETURN {
    ($name:ident, $type:ty, $pointer:expr) => {
        let $name = $crate::mechanical_port::source::include::utils::lite_rtti_hpp::lite_rtti_cast::<$type, _>($pointer);
        if $name.is_null() {
            return;
        }
    };
}

// #define LITE_RTTI_CAST_OR_BREAK(NAME, TYPE, POINTER)                               \
//     auto NAME = rive::lite_rtti_cast<TYPE>(POINTER);                                \
//     if (NAME == nullptr)                                                            \
//         break
#[macro_export]
macro_rules! LITE_RTTI_CAST_OR_BREAK {
    ($name:ident, $type:ty, $pointer:expr) => {
        let $name = $crate::mechanical_port::source::include::utils::lite_rtti_hpp::lite_rtti_cast::<$type, _>($pointer);
        if $name.is_null() {
            break;
        }
    };
}

// #define LITE_RTTI_CAST_OR_CONTINUE(NAME, TYPE, POINTER)                            \
//     auto NAME = rive::lite_rtti_cast<TYPE>(POINTER);                                \
//     if (NAME == nullptr)                                                            \
//         continue
#[macro_export]
macro_rules! LITE_RTTI_CAST_OR_CONTINUE {
    ($name:ident, $type:ty, $pointer:expr) => {
        let $name = $crate::mechanical_port::source::include::utils::lite_rtti_hpp::lite_rtti_cast::<$type, _>($pointer);
        if $name.is_null() {
            continue;
        }
    };
}
// clang-format on

// } // namespace rive

#[cfg(test)]
#[path = "lite_rtti_test.rs"]
mod upstream_tests;

#[cfg(test)]
mod tests {
    use super::*;

    struct Root;
    struct Derived;
    type Base = enable_lite_rtti<Root, 1>;
    type MostDerived = lite_rtti_override<Base, Derived, 2>;

    #[test]
    fn cast_or_assert_preserves_the_matching_complete_object_pointer() {
        let mut value = MostDerived::default();
        let expected = &mut value as *mut MostDerived;
        let base = value.base_mut() as *mut Base;
        // SAFETY: base is the live, offset-zero Base subobject of MostDerived.
        assert_eq!(
            unsafe { lite_rtti_cast_or_assert::<MostDerived, _>(base) },
            expected
        );
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn cast_or_assert_rejects_null_in_debug() {
        // SAFETY: null is a permitted cast input; the assertion rejects it.
        unsafe { lite_rtti_cast_or_assert::<MostDerived, Base>(core::ptr::null_mut()) };
    }

    #[cfg(debug_assertions)]
    #[test]
    #[should_panic]
    fn cast_or_assert_rejects_mismatched_identity_in_debug() {
        let mut base = Base::default();
        // SAFETY: base is live and its nonmatching ID prevents a downcast.
        unsafe { lite_rtti_cast_or_assert::<MostDerived, _>(&mut base) };
    }

    #[cfg(not(debug_assertions))]
    #[test]
    fn cast_or_assert_returns_null_for_failed_casts_in_release() {
        let mut base = Base::default();
        // SAFETY: null and a live base with a nonmatching ID are cast inputs.
        unsafe {
            assert!(lite_rtti_cast_or_assert::<MostDerived, Base>(core::ptr::null_mut()).is_null());
            assert!(lite_rtti_cast_or_assert::<MostDerived, _>(&mut base).is_null());
        }
    }
}
