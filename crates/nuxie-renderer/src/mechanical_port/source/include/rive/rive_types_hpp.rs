/*
 * Copyright 2022 Rive
 */

// This should always be included by any other rive files,
// as it performs basic self-consistency checks, and provides
// shared common types and macros.

// #ifndef _RIVE_TYPES_HPP_
// #define _RIVE_TYPES_HPP_

// #include <memory>   // For unique_ptr.
// #include <string.h> // For memcpy.

// Mechanical translation of the complete pinned source header
// include/rive/rive_types.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use core::ptr;

// #if defined(DEBUG) && defined(NDEBUG)
// #error "can't determine if we're debug or release"
// #endif
// Rust's debug_assertions configuration is the source DEBUG/NDEBUG choice;
// the contradictory preprocessor branch therefore has no satisfiable cfg.
#[cfg(all(debug_assertions, not(debug_assertions)))]
compile_error!("can't determine if we're debug or release");

// #if !defined(DEBUG) && !defined(NDEBUG)
// // we have to make a decision what mode we're in
// // historically this has been to look for NDEBUG, and in its
// // absence assume we're DEBUG.
// #define DEBUG 1
// // fyi - Xcode seems to set DEBUG (or not), so the above guess
// // doesn't work for them - so our projects may need to explicitly
// // set NDEBUG in our 'release' builds.
// #endif
// Rust always supplies one side of this source decision.
#[cfg(debug_assertions)]
pub const DEBUG: i32 = 1;

#[cfg(not(debug_assertions))]
pub const NDEBUG: i32 = 1;

// #ifdef NDEBUG
// #ifndef RELEASE
// #define RELEASE 1
// #endif
// #else // debug mode
// #ifndef DEBUG
// #define DEBUG 1
// #endif
// #endif
#[cfg(not(debug_assertions))]
pub const RELEASE: i32 = 1;

// Some checks to guess what platform we're building for

// #ifdef __APPLE__
// #define RIVE_BUILD_FOR_APPLE
// #include <TargetConditionals.h>

#[cfg(target_vendor = "apple")]
pub const RIVE_BUILD_FOR_APPLE: bool = true;

// #if TARGET_OS_IPHONE
// #define RIVE_BUILD_FOR_IOS
// #elif TARGET_OS_MAC
// #define RIVE_BUILD_FOR_OSX
// #endif
#[cfg(all(
    target_vendor = "apple",
    any(
        target_os = "ios",
        target_os = "tvos",
        target_os = "watchos",
        target_os = "visionos"
    )
))]
pub const RIVE_BUILD_FOR_IOS: bool = true;

#[cfg(all(target_vendor = "apple", target_os = "macos"))]
pub const RIVE_BUILD_FOR_OSX: bool = true;

// #define RIVE_NO_STD_SYSTEM
#[cfg(target_vendor = "apple")]
pub const RIVE_NO_STD_SYSTEM: bool = true;

// #endif

// We really like these headers, so we include them all the time.

// #include <algorithm>
// #include <cassert>
// #include <cstddef>
// #include <cstdint>
// #include <memory>
// #include <type_traits>

// Annotations to assert unreachable control flow.
// #if defined(__GNUC__) || defined(__clang__)
// #define RIVE_UNREACHABLE                                                       \
//     assert(!(bool)"unreachable reached");                                      \
//     __builtin_unreachable
// #elif _MSC_VER
// #define RIVE_UNREACHABLE()                                                     \
//     assert(!(bool)"unreachable reached");                                      \
//     __assume(0)
// #else
// #define RIVE_UNREACHABLE()                                                     \
//     do                                                                         \
//     {                                                                          \
//         assert(!(bool)"unreachable reached");                                  \
//     } while (0)
// #endif
// The three source compiler branches all preserve the assertion followed by
// an unreachable operation. Rust's intrinsic is the direct source-shaped
// correspondence for the active compiler path.
#[inline(always)]
pub fn RIVE_UNREACHABLE() -> ! {
    debug_assert!(!true, "unreachable reached");
    // SAFETY: callers use this helper only on the source-proven unreachable
    // path, matching __builtin_unreachable and __assume(0).
    unsafe { core::hint::unreachable_unchecked() }
}

// #if __cplusplus >= 201703L
// #define RIVE_MAYBE_UNUSED [[maybe_unused]]
// #else
// #define RIVE_MAYBE_UNUSED
// #endif
// Rust applies the equivalent attribute at each translated declaration.
// This marker retains the source macro name for source-shaped translation.
pub const RIVE_MAYBE_UNUSED: () = ();

// #if __cplusplus >= 201703L
// #define RIVE_FALLTHROUGH [[fallthrough]]
// #elif defined(__clang__)
// #define RIVE_FALLTHROUGH [[clang::fallthrough]]
// #else
// #define RIVE_FALLTHROUGH
// #endif
// Rust's explicit match arms make fallthrough impossible; this marker keeps
// the source declaration available to later mechanical translations.
pub const RIVE_FALLTHROUGH: () = ();

// #if defined(__GNUC__) || defined(__clang__)
// #define RIVE_ALWAYS_INLINE inline __attribute__((always_inline))
// #else
// #define RIVE_ALWAYS_INLINE inline
// #endif
// Rust's direct equivalent is #[inline(always)] on each translated function.
#[inline(always)]
pub const fn RIVE_ALWAYS_INLINE() {}

// Exempts a function from Clang's -Wthread-safety analysis. Use only for
// intentional, correct locking patterns the analyzer cannot statically verify
// (e.g. an asymmetric lock acquired in one function and released elsewhere).
// #ifndef __has_attribute
// #define __has_attribute(x) 0
// #endif
#[allow(non_snake_case)]
pub const fn __has_attribute(_: &str) -> i32 {
    0
}

// #if defined(__clang__) && __has_attribute(no_thread_safety_analysis)
// #define RIVE_NO_THREAD_SAFETY_ANALYSIS                                         \
//     __attribute__((no_thread_safety_analysis))
// #else
// #define RIVE_NO_THREAD_SAFETY_ANALYSIS
// #endif
// Rust has no thread-safety-analysis attribute; retain the source marker and
// apply the corresponding review annotation at translated call sites.
pub const RIVE_NO_THREAD_SAFETY_ANALYSIS: () = ();

// #if defined(__GNUC__) || defined(__clang__)
// // Recommended in
// // https://clang.llvm.org/docs/LanguageExtensions.html#feature-checking-macros
// #ifndef __has_builtin
// #define __has_builtin(x) 0
// #endif
// #else
// #define __has_builtin(x) 0
// #endif
// #if __has_builtin(__builtin_memcpy)
// #define RIVE_INLINE_MEMCPY __builtin_memcpy
// #else
// #define RIVE_INLINE_MEMCPY memcpy
// #endif
// Rust's ptr::copy_nonoverlapping is the source-shaped memcpy operation.
#[inline(always)]
pub unsafe fn RIVE_INLINE_MEMCPY(dst: *mut u8, src: *const u8, size: usize) {
    // SAFETY: the caller supplies the same valid, non-overlapping byte ranges
    // required by the source memcpy/__builtin_memcpy call.
    ptr::copy_nonoverlapping(src, dst, size);
}

// #ifdef DEBUG
// #define RIVE_DEBUG_CODE(...) __VA_ARGS__
// #else
// #define RIVE_DEBUG_CODE(...)
// #endif
#[cfg(debug_assertions)]
macro_rules! RIVE_DEBUG_CODE {
    ($($tokens:tt)*) => {
        $($tokens)*
    };
}

#[cfg(not(debug_assertions))]
macro_rules! RIVE_DEBUG_CODE {
    ($($tokens:tt)*) => {};
}

// #endif // rive_types
