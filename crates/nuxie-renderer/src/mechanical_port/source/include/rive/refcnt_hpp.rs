/*
 * Copyright 2021 Rive
 */

// Mechanical translation of the complete pinned source header
// include/rive/refcnt.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// #ifndef _RIVE_REFCNT_HPP_
// #define _RIVE_REFCNT_HPP_

// #include "rive/rive_types.hpp"
// #include <atomic>
// #include <cstddef>
// #include <type_traits>
// #include <utility>

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::hash::{Hash, Hasher};
use core::marker::PhantomData;
use core::ptr;
use core::sync::atomic::{AtomicI32, Ordering};

/*
 *  RefCnt : Threadsafe shared pointer baseclass.
 *
 *  The reference count is set to one in the constructor, and goes up on every
 * call to ref(), and down on every call to unref(). When a call to unref()
 * brings the counter to 0, the object is casted to class "const T*" and
 * deleted. Usage:
 *
 *    class MyClass : public RefCnt<MyClass>
 *
 *  rcp : template wrapper for subclasses of RefCnt, to manage assignment and
 * parameter passing to safely keep track of shared ownership.
 *
 *  Both of these inspired by Skia's SkRefCnt and sk_sp
 */

// namespace rive
// {

// Rust has no direct spelling for a C++ derived class inheriting the base
// methods. This trait is the narrow source-shaped seam used by `safe_ref`,
// `safe_unref`, and `rcp`; concrete RefCnt-derived payloads implement these
// methods while retaining the source's atomic counter and zero-release hook.
/// # Safety
/// Implementors must place their `RefCnt<Self>` base at offset zero. Any value
/// passed through the unsafe zero-transition `unref` operation must be the
/// live heap-published allocation carrying an intrusive reference; source
/// stack temporaries may implement this trait but must never take that path.
/// This is the Rust equivalent of the source's valid
/// `static_cast<const T*>(this)` base-to-derived conversion and delete.
pub unsafe trait RefCntTarget {
    fn r#ref(&self);
    unsafe fn unref(&self);

    // The default is the source `delete static_cast<const T*>(this)` branch.
    // A concrete payload may override this hook for specialized deletion.
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        // SAFETY: the source zero-reference transition owns the allocation and
        // casts the base address to the concrete `const T*` before deletion.
        drop(Box::from_raw(ptr as *mut Self));
    }
}

// template <typename T> class RefCnt
// {
// public:
//     RefCnt() : m_refcnt(1) {}
//
//     void ref() const
//     {
//         (void)m_refcnt.fetch_add(+1, std::memory_order_relaxed);
//     }
//
//     void unref() const
//     {
//         if (1 == m_refcnt.fetch_add(-1, std::memory_order_acq_rel))
//         {
//             static_cast<const T*>(this)->onRefCntReachedZero();
//         }
//     }
//
//     // not reliable in actual threaded scenarios, but useful (perhaps) for
//     // debugging
//     int32_t debugging_refcnt() const
//     {
//         return m_refcnt.load(std::memory_order_relaxed);
//     }
//
// protected:
//     // Can be overloaded in the subclass if specialized delete behavior is
//     // required.
//     void onRefCntReachedZero() const { delete static_cast<const T*>(this); }
//
// private:
//     // mutable, so can be changed even on a const object
//     mutable std::atomic<int32_t> m_refcnt;
//
//     RefCnt(RefCnt&&) = delete;
//     RefCnt(const RefCnt&) = delete;
//     RefCnt& operator=(RefCnt&&) = delete;
//     RefCnt& operator=(const RefCnt&) = delete;
// };

#[repr(C)]
pub struct RefCnt<T: RefCntTarget> {
    // mutable std::atomic<int32_t> m_refcnt;
    // `PhantomData` carries only the source template identity; the atomic is
    // the sole source state and uses the exact C++ memory orders below.
    m_refcnt: AtomicI32,
    _target: PhantomData<fn() -> T>,
}

impl<T: RefCntTarget> RefCnt<T> {
    // RefCnt() : m_refcnt(1) {}
    pub const fn new() -> Self {
        Self {
            m_refcnt: AtomicI32::new(1),
            _target: PhantomData,
        }
    }

    // void ref() const
    // {
    //     (void)m_refcnt.fetch_add(+1, std::memory_order_relaxed);
    // }
    pub fn r#ref(&self) {
        let _ = self.m_refcnt.fetch_add(1, Ordering::Relaxed);
    }

    // void unref() const
    // {
    //     if (1 == m_refcnt.fetch_add(-1, std::memory_order_acq_rel))
    //     {
    //         static_cast<const T*>(this)->onRefCntReachedZero();
    //     }
    // }
    pub unsafe fn unref(&self) {
        if 1 == self.m_refcnt.fetch_add(-1, Ordering::AcqRel) {
            // SAFETY: this is the source static_cast<const T*>(this) at the
            // sole zero-reference transition; the concrete target owns the
            // allocation and its specialized zero hook, if any.
            unsafe { self.onRefCntReachedZero() };
        }
    }

    // not reliable in actual threaded scenarios, but useful (perhaps) for
    // debugging
    // int32_t debugging_refcnt() const
    // {
    //     return m_refcnt.load(std::memory_order_relaxed);
    // }
    pub fn debugging_refcnt(&self) -> i32 {
        self.m_refcnt.load(Ordering::Relaxed)
    }

    // protected:
    // // Can be overloaded in the subclass if specialized delete behavior is
    // // required.
    // void onRefCntReachedZero() const { delete static_cast<const T*>(this); }
    pub unsafe fn onRefCntReachedZero(&self) {
        // SAFETY: the C++ base-to-derived static_cast and delete are the source
        // operation. The target trait is the explicit Rust representation of
        // that derived zero-reference callback.
        T::onRefCntReachedZero(self as *const Self as *const T);
    }
}

// private:
// // mutable, so can be changed even on a const object
// mutable std::atomic<int32_t> m_refcnt;
//
// RefCnt(RefCnt&&) = delete;
// RefCnt(const RefCnt&) = delete;
// RefCnt& operator=(RefCnt&&) = delete;
// RefCnt& operator=(const RefCnt&) = delete;
// Rust move semantics and the absence of a Clone/Copy implementation preserve
// all four deleted source operations.

// template <typename T> static inline T* safe_ref(T* obj)
// {
//     if (obj)
//     {
//         obj->ref();
//     }
//     return obj;
// }
#[inline]
pub unsafe fn safe_ref<T: RefCntTarget>(obj: *mut T) -> *mut T {
    if !obj.is_null() {
        // SAFETY: a non-null source pointer is a live RefCnt-derived object for
        // the duration of this retain operation.
        unsafe { (&*obj).r#ref() };
    }
    obj
}

// template <typename T> static inline void safe_unref(T* obj)
// {
//     if (obj)
//     {
//         obj->unref();
//     }
// }
#[inline]
pub unsafe fn safe_unref<T: RefCntTarget>(obj: *mut T) {
    if !obj.is_null() {
        // SAFETY: a non-null source pointer owns one logical retain at this
        // call site, and unref performs the source atomic release.
        unsafe { (&*obj).unref() };
    }
}

// rcp : smart point template for holding subclasses of RefCnt

// template <typename T> class rcp
// {
// public:
//     constexpr rcp() : m_ptr(nullptr) {}
//     constexpr rcp(std::nullptr_t) : m_ptr(nullptr) {}
//     explicit rcp(T* ptr) : m_ptr(ptr) {}
//
//     rcp(const rcp<T>& other) : m_ptr(safe_ref(other.get())) {}
//     rcp(rcp<T>&& other) : m_ptr(other.release()) {}
//
//     template <typename U,
//               typename = typename std::enable_if<
//                   std::is_convertible<U*, T*>::value>::type>
//     rcp(const rcp<U>& other) : m_ptr(safe_ref(other.get()))
//     {}
//
//     template <typename U,
//               typename = typename std::enable_if<
//                   std::is_convertible<U*, T*>::value>::type>
//     rcp(rcp<U>&& other) : m_ptr(other.release())
//     {}
//
//     /**
//      *  Calls unref() on the underlying object pointer.
//      */
//     ~rcp() { safe_unref(m_ptr); }
//
//     rcp<T>& operator=(std::nullptr_t)
//     {
//         this->reset();
//         return *this;
//     }
//
//     rcp<T>& operator=(const rcp<T>& other)
//     {
//         if (this != &other)
//         {
//             this->reset(safe_ref(other.get()));
//         }
//         return *this;
//     }
//
//     // move assignment operator
//     rcp<T>& operator=(rcp<T>&& other)
//     {
//         this->reset(other.release());
//         return *this;
//     }
//
//     T& operator*() const
//     {
//         assert(this->get() != nullptr);
//         return *this->get();
//     }
//
//     explicit operator bool() const { return this->get() != nullptr; }
//
//     T* get() const { return m_ptr; }
//     T* operator->() const { return m_ptr; }
//
//     // Unrefs the current pointer, and accepts the new pointer, but
//     // DOES NOT increment ownership of the new pointer.
//     void reset(T* ptr = nullptr)
//     {
//         // Calling m_ptr->unref() may call this->~() or this->reset(T*).
//         // http://wg21.cmeerw.net/lwg/issue998
//         // http://wg21.cmeerw.net/lwg/issue2262
//         T* oldPtr = m_ptr;
//         m_ptr = ptr;
//         safe_unref(oldPtr);
//     }
//
//     // This returns the bare point WITHOUT CHANGING ITS REFCNT, but removes it
//     // from this object, so the caller must manually manage its count.
//     T* release()
//     {
//         T* ptr = m_ptr;
//         m_ptr = nullptr;
//         return ptr;
//     }
//
//     void swap(rcp<T>& other) { std::swap(m_ptr, other.m_ptr); }
//
// private:
//     T* m_ptr;
// };

pub struct rcp<T: RefCntTarget> {
    // T* m_ptr;
    // Nullable raw pointer is intentional: nullptr is the source empty state,
    // while each non-null value owns exactly one intrusive retain.
    m_ptr: *mut T,
}

// SAFETY: the wrapper mutates only its own pointer. Cross-thread clones and
// drops touch the pointee solely through RefCnt's atomic operations, and the
// pointee itself must satisfy the corresponding Send/Sync contract.
unsafe impl<T: RefCntTarget + Send + Sync> Send for rcp<T> {}
unsafe impl<T: RefCntTarget + Send + Sync> Sync for rcp<T> {}

impl<T: RefCntTarget> rcp<T> {
    // constexpr rcp() : m_ptr(nullptr) {}
    pub const fn new() -> Self {
        Self {
            m_ptr: ptr::null_mut(),
        }
    }

    // constexpr rcp(std::nullptr_t) : m_ptr(nullptr) {}
    pub const fn from_null(_: Option<*mut T>) -> Self {
        Self {
            m_ptr: ptr::null_mut(),
        }
    }

    // explicit rcp(T* ptr) : m_ptr(ptr) {}
    pub const unsafe fn from_ptr(ptr: *mut T) -> Self {
        Self { m_ptr: ptr }
    }

    // rcp(const rcp<T>& other) : m_ptr(safe_ref(other.get())) {}
    pub fn copy_ctor(other: &Self) -> Self {
        unsafe { Self::from_ptr(safe_ref(other.get())) }
    }

    // rcp(rcp<T>&& other) : m_ptr(other.release()) {}
    pub fn move_ctor(other: &mut Self) -> Self {
        unsafe { Self::from_ptr(other.release()) }
    }

    // template <typename U,
    //           typename = typename std::enable_if<
    //               std::is_convertible<U*, T*>::value>::type>
    // rcp(const rcp<U>& other) : m_ptr(safe_ref(other.get()))
    // {}
    pub unsafe fn converting_copy_ctor<U: RefCntTarget>(other: &rcp<U>) -> Self {
        // SAFETY: this is the source `std::is_convertible<U*, T*>` conversion
        // represented as a raw-pointer cast at the mechanical boundary.
        let ptr = safe_ref(other.get());
        Self::from_ptr(ptr as *mut T)
    }

    // template <typename U,
    //           typename = typename std::enable_if<
    //               std::is_convertible<U*, T*>::value>::type>
    // rcp(rcp<U>&& other) : m_ptr(other.release())
    // {}
    pub unsafe fn converting_move_ctor<U: RefCntTarget>(other: &mut rcp<U>) -> Self {
        // SAFETY: this is the source `std::is_convertible<U*, T*>` move
        // conversion; release first transfers the one logical retain.
        Self::from_ptr(other.release() as *mut T)
    }

    // /**
    //  *  Calls unref() on the underlying object pointer.
    //  */
    // ~rcp() { safe_unref(m_ptr); }
}

impl<T: RefCntTarget> Drop for rcp<T> {
    fn drop(&mut self) {
        unsafe { safe_unref(self.m_ptr) };
    }
}

impl<T: RefCntTarget> Clone for rcp<T> {
    fn clone(&self) -> Self {
        Self::copy_ctor(self)
    }
}

impl<T: RefCntTarget> rcp<T> {
    // rcp<T>& operator=(std::nullptr_t)
    // {
    //     this->reset();
    //     return *this;
    // }
    pub fn operator_assign_null(&mut self) -> &mut Self {
        // SAFETY: null transfers no new retain into this owner.
        unsafe { self.reset(ptr::null_mut()) };
        self
    }

    // rcp<T>& operator=(const rcp<T>& other)
    // {
    //     if (this != &other)
    //     {
    //         this->reset(safe_ref(other.get()));
    //     }
    //     return *this;
    // }
    pub fn operator_assign_copy(&mut self, other: &Self) -> &mut Self {
        if !core::ptr::eq(self, other) {
            unsafe { self.reset(safe_ref(other.get())) };
        }
        self
    }

    // move assignment operator
    // rcp<T>& operator=(rcp<T>&& other)
    // {
    //     this->reset(other.release());
    //     return *this;
    // }
    pub fn operator_assign_move(&mut self, other: &mut Self) -> &mut Self {
        // SAFETY: release transfers the source owner's existing logical
        // retain into this owner without incrementing it.
        unsafe { self.reset(other.release()) };
        self
    }

    // T& operator*() const
    // {
    //     assert(this->get() != nullptr);
    //     return *this->get();
    // }
    pub unsafe fn operator_deref(&self) -> &T {
        debug_assert!(!self.get().is_null());
        // SAFETY: the source assertion establishes a non-null pointee.
        unsafe { &*self.get() }
    }

    // explicit operator bool() const { return this->get() != nullptr; }
    pub fn operator_bool(&self) -> bool {
        !self.get().is_null()
    }

    // T* get() const { return m_ptr; }
    pub fn get(&self) -> *mut T {
        self.m_ptr
    }

    // T* operator->() const { return m_ptr; }
    pub fn operator_arrow(&self) -> *mut T {
        self.m_ptr
    }

    // Unrefs the current pointer, and accepts the new pointer, but
    // DOES NOT increment ownership of the new pointer.
    // void reset(T* ptr = nullptr)
    // {
    //     // Calling m_ptr->unref() may call this->~() or this->reset(T*).
    //     // http://wg21.cmeerw.net/lwg/issue998
    //     // http://wg21.cmeerw.net/lwg/issue2262
    //     T* oldPtr = m_ptr;
    //     m_ptr = ptr;
    //     safe_unref(oldPtr);
    // }
    pub unsafe fn reset(&mut self, ptr: *mut T) {
        let old_ptr = self.m_ptr;
        self.m_ptr = ptr;
        // SAFETY: the replaced pointer carried this owner's logical retain.
        unsafe { safe_unref(old_ptr) };
    }

    // This returns the bare point WITHOUT CHANGING ITS REFCNT, but removes it
    // from this object, so the caller must manually manage its count.
    // T* release()
    // {
    //     T* ptr = m_ptr;
    //     m_ptr = nullptr;
    //     return ptr;
    // }
    pub fn release(&mut self) -> *mut T {
        let ptr = self.m_ptr;
        self.m_ptr = ptr::null_mut();
        ptr
    }

    // void swap(rcp<T>& other) { std::swap(m_ptr, other.m_ptr); }
    pub fn swap(&mut self, other: &mut Self) {
        core::mem::swap(&mut self.m_ptr, &mut other.m_ptr);
    }
}

// template <typename T> inline void swap(rcp<T>& a, rcp<T>& b) { a.swap(b); }
#[inline]
pub fn swap<T: RefCntTarget>(a: &mut rcp<T>, b: &mut rcp<T>) {
    a.swap(b);
}

// template <typename T, typename... Args> rcp<T> inline make_rcp(Args&&... args)
// {
//     return rcp<T>(new T(std::forward<Args>(args)...));
// }
#[inline]
pub fn make_rcp<T: RefCntTarget, F: FnOnce() -> T>(constructor: F) -> rcp<T> {
    // SAFETY: Box publishes one stable allocation with the source's initial
    // count of one.
    unsafe { rcp::from_ptr(Box::into_raw(Box::new(constructor()))) }
}

// template <typename T> rcp<T> inline ref_rcp(T* ptr)
// {
//     return rcp<T>(safe_ref(ptr));
// }
#[inline]
pub unsafe fn ref_rcp<T: RefCntTarget>(ptr: *mut T) -> rcp<T> {
    rcp::from_ptr(safe_ref(ptr))
}

// template <typename U,
//           typename T,
//           typename =
//               typename std::enable_if<std::is_convertible<U*, T*>::value>::type>
// rcp<U> static_rcp_cast(rcp<T> ptr)
// {
//     return rcp<U>(static_cast<U*>(ptr.release()));
// }
#[inline]
pub unsafe fn static_rcp_cast<U: RefCntTarget, T: RefCntTarget>(mut ptr: rcp<T>) -> rcp<U> {
    // SAFETY: this is the source static_cast<U*>(ptr.release()) conversion;
    // the caller supplies the source-proven convertible target types.
    rcp::from_ptr(ptr.release() as *mut U)
}

// == variants

// template <typename T> inline bool operator==(const rcp<T>& a, std::nullptr_t)
// {
//     return !a;
// }
#[inline]
pub fn operator_eq_rcp_null<T: RefCntTarget>(a: &rcp<T>) -> bool {
    !a.operator_bool()
}

// template <typename T> inline bool operator==(std::nullptr_t, const rcp<T>& b)
// {
//     return !b;
// }
#[inline]
pub fn operator_eq_null_rcp<T: RefCntTarget>(b: &rcp<T>) -> bool {
    !b.operator_bool()
}

// template <typename T, typename U>
// inline bool operator==(const rcp<T>& a, const rcp<U>& b)
// {
//     return a.get() == b.get();
// }
#[inline]
pub fn operator_eq_rcp<T: RefCntTarget, U: RefCntTarget>(a: &rcp<T>, b: &rcp<U>) -> bool {
    a.get() as *const () == b.get() as *const ()
}

// != variants

// template <typename T> inline bool operator!=(const rcp<T>& a, std::nullptr_t)
// {
//     return static_cast<bool>(a);
// }
#[inline]
pub fn operator_ne_rcp_null<T: RefCntTarget>(a: &rcp<T>) -> bool {
    a.operator_bool()
}

// template <typename T> inline bool operator!=(std::nullptr_t, const rcp<T>& b)
// {
//     return static_cast<bool>(b);
// }
#[inline]
pub fn operator_ne_null_rcp<T: RefCntTarget>(b: &rcp<T>) -> bool {
    b.operator_bool()
}

// template <typename T, typename U>
// inline bool operator!=(const rcp<T>& a, const rcp<U>& b)
// {
//     return a.get() != b.get();
// }
#[inline]
pub fn operator_ne_rcp<T: RefCntTarget, U: RefCntTarget>(a: &rcp<T>, b: &rcp<U>) -> bool {
    !operator_eq_rcp(a, b)
}

// } // namespace rive

// namespace std
// {
// template <typename T> struct hash<rive::rcp<T>>
// {
//     using result_type = std::size_t;
//     using argument_type = rive::rcp<T>;
//
//     std::size_t operator()(const rive::rcp<T>& up) const
//     {
//         return hash<T*>()(up.get());
//     }
// };
//
// } // namespace std
// #endif

// Rust's orphan rules prevent a `std::hash<rive::rcp<T>>` specialization, so
// the source hash operation is represented by the equivalent `Hash` impl on
// the local source-shaped owner.
impl<T: RefCntTarget> Hash for rcp<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.get().hash(state);
    }
}

impl<T: RefCntTarget> PartialEq for rcp<T> {
    fn eq(&self, other: &Self) -> bool {
        self.get() == other.get()
    }
}

impl<T: RefCntTarget> Eq for rcp<T> {}
