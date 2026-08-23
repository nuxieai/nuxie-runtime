/*
 * Copyright 2023 Rive
 */

// "lite_rtti_cast<T*>()" is a very basic polyfill for "dynamic_cast<T*>()" that
// can only cast a pointer to its most-derived type. To use it, the base class
// must derive from enable_lite_rtti, and the subclass must inherit from
// lite_rtti_override:
//
//     class Root : public enable_lite_rtti<Root> {};
//     class Derived : public lite_rtti_override<Root, Derived> {};
//     Root* derived = new Derived();
//     lite_rtti_cast<Derived*>(derived);
//

// #pragma once

// #include "utils/compile_time_string_hash.hpp"
// #include "rive/refcnt.hpp"
// #include <stdint.h>
// #include <type_traits>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use super::super::rive::refcnt_hpp::{rcp, RefCntTarget};
use core::marker::PhantomData;

// The source include supplies CONST_ID through compile_time_string_hash.hpp.
// Keep the CRC32 operation local to this source-shaped header so the two RTTI
// macros retain their source behavior when this file is compiled on its own.
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

// namespace rive
// {

// Enable lite rtti on the root of a class hierarchy.
// template <class Root, unsigned int ID> class enable_lite_rtti
// {
// public:
//     unsigned int liteTypeID() const { return m_liteTypeId; }
//
// protected:
//     unsigned int m_liteTypeId = ID;
// };
//
// Rust has no base-class subobject, so the Root template identity is carried
// by PhantomData while m_liteTypeId remains the sole runtime state.
#[repr(C)]
pub struct enable_lite_rtti<Root, const ID: u32> {
    // unsigned int m_liteTypeId = ID;
    pub(crate) m_liteTypeId: u32,
    _root: PhantomData<fn() -> Root>,
}

impl<Root, const ID: u32> enable_lite_rtti<Root, ID> {
    // The source default member initializer is the default construction path.
    pub const fn new() -> Self {
        Self {
            m_liteTypeId: ID,
            _root: PhantomData,
        }
    }

    // unsigned int liteTypeID() const { return m_liteTypeId; }
    pub fn liteTypeID(&self) -> u32 {
        self.m_liteTypeId
    }

    // The C++ member is protected and is assigned by lite_rtti_override.
    pub(crate) fn setLiteTypeID(&mut self, id: u32) {
        self.m_liteTypeId = id;
    }
}

impl<Root, const ID: u32> Default for enable_lite_rtti<Root, ID> {
    fn default() -> Self {
        Self::new()
    }
}

// This narrow source-shaped seam represents access to the protected
// m_liteTypeId member through a C++ base subobject. It is deliberately limited
// to type identity and does not introduce an alternate RTTI mechanism.
pub trait LiteRttiBase {
    fn liteTypeID(&self) -> u32;
    fn setLiteTypeID(&mut self, id: u32);
}

impl<Root, const ID: u32> LiteRttiBase for enable_lite_rtti<Root, ID> {
    fn liteTypeID(&self) -> u32 {
        enable_lite_rtti::<Root, ID>::liteTypeID(self)
    }

    fn setLiteTypeID(&mut self, id: u32) {
        enable_lite_rtti::<Root, ID>::setLiteTypeID(self, id);
    }
}

// Override the lite rtti type ID on subsequent classes of a class hierarchy.
// template <class Base, class Derived, unsigned int ID>
// class lite_rtti_override : public Base
// {
// public:
//     constexpr static uint32_t LITE_RTTI_TYPE_ID = ID;
//     lite_rtti_override() { Base::m_liteTypeId = ID; }
//
//     template <typename... Args>
//     lite_rtti_override(Args&&... args) : Base(std::forward<Args>(args)...)
//     {
//         Base::m_liteTypeId = ID;
//     }
// };
//
// The owned Base field is the Rust representation of the C++ base subobject;
// Derived is retained as a type-only template parameter, exactly like the
// source class identity used by CONST_ID(DERRIVED).
#[repr(C)]
pub struct lite_rtti_override<Base: LiteRttiBase, Derived, const ID: u32> {
    base: Base,
    _derived: PhantomData<fn() -> Derived>,
}

impl<Base: LiteRttiBase, Derived, const ID: u32> lite_rtti_override<Base, Derived, ID> {
    // constexpr static uint32_t LITE_RTTI_TYPE_ID = ID;
    pub const LITE_RTTI_TYPE_ID: u32 = ID;

    // lite_rtti_override() { Base::m_liteTypeId = ID; }
    pub fn new(mut base: Base) -> Self {
        base.setLiteTypeID(ID);
        Self {
            base,
            _derived: PhantomData,
        }
    }

    // The source's variadic forwarding constructor is represented by a
    // one-shot Base constructor while retaining the same assignment order.
    // template <typename... Args>
    // lite_rtti_override(Args&&... args) : Base(std::forward<Args>(args)...)
    // {
    //     Base::m_liteTypeId = ID;
    // }
    pub fn from_args<F>(constructor: F) -> Self
    where
        F: FnOnce() -> Base,
    {
        Self::new(constructor())
    }

    pub fn base(&self) -> &Base {
        &self.base
    }

    pub fn base_mut(&mut self) -> &mut Base {
        &mut self.base
    }

    // Source access through the public liteTypeID() member of the base.
    pub fn liteTypeID(&self) -> u32 {
        self.base.liteTypeID()
    }
}

impl<Base: LiteRttiBase + Default, Derived, const ID: u32> Default
    for lite_rtti_override<Base, Derived, ID>
{
    fn default() -> Self {
        Self::new(Base::default())
    }
}

impl<Base: LiteRttiBase, Derived, const ID: u32> LiteRttiBase
    for lite_rtti_override<Base, Derived, ID>
{
    fn liteTypeID(&self) -> u32 {
        lite_rtti_override::<Base, Derived, ID>::liteTypeID(self)
    }

    fn setLiteTypeID(&mut self, id: u32) {
        self.base.setLiteTypeID(id);
    }
}

// The pointee type being requested supplies the source
// `remove_pointer<U>::type::LITE_RTTI_TYPE_ID` static value.
pub trait LiteRttiTypeId {
    const LITE_RTTI_TYPE_ID: u32;
}

pub trait LiteRttiCastFrom<Base>: LiteRttiTypeId {
    /// # Safety
    /// `base` must point at the Base subobject of a live complete `Self`.
    unsafe fn from_base(base: *mut Base) -> *mut Self;
}

impl<Base: LiteRttiBase, Derived, const ID: u32> LiteRttiCastFrom<Base>
    for lite_rtti_override<Base, Derived, ID>
{
    unsafe fn from_base(base: *mut Base) -> *mut Self {
        // repr(C) and the first-field base establish the offset-zero invariant
        // for this concrete inheritance surrogate.
        base.cast()
    }
}

impl<Base: LiteRttiBase, Derived, const ID: u32> LiteRttiTypeId
    for lite_rtti_override<Base, Derived, ID>
{
    const LITE_RTTI_TYPE_ID: u32 = ID;
}

// Like dynamic_cast<>, but can only cast a pointer to its most-derived type.
// template <class U, class T> U lite_rtti_cast(T* t)
// {
//     if (t != nullptr &&
//         t->liteTypeID() == std::remove_pointer<U>::type::LITE_RTTI_TYPE_ID)
//     {
//         return static_cast<U>(t);
//     }
//     return nullptr;
// }
pub unsafe fn lite_rtti_cast<U, T>(t: *mut T) -> *mut U
where
    U: LiteRttiCastFrom<T>,
    T: LiteRttiBase,
{
    if !t.is_null() {
        // SAFETY: as in the source, the caller supplies a live T*; this read
        // occurs only to compare the stored most-derived identity.
        if unsafe { (&*t).liteTypeID() } == U::LITE_RTTI_TYPE_ID {
            return unsafe { U::from_base(t) };
        }
    }
    core::ptr::null_mut()
}

// template <class U, class T> rcp<U> lite_rtti_rcp_cast(rcp<T> t)
// {
//     if (t != nullptr &&
//         t->liteTypeID() == std::remove_pointer<U>::type::LITE_RTTI_TYPE_ID)
//     {
//         return static_rcp_cast<U>(t);
//     }
//     return nullptr;
// }
//
pub unsafe fn lite_rtti_rcp_cast<U, T>(mut t: rcp<T>) -> rcp<U>
where
    U: LiteRttiCastFrom<T> + RefCntTarget,
    T: LiteRttiBase + RefCntTarget,
{
    if !t.get().is_null() && unsafe { (&*t.get()).liteTypeID() } == U::LITE_RTTI_TYPE_ID {
        let base = t.release();
        return unsafe { rcp::from_ptr(U::from_base(base)) };
    }
    // Consuming t and returning an empty rcp preserves the source's failed
    // cast release when the by-value argument leaves scope.
    rcp::new()
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
