/*
 * Copyright 2023 Rive
 */

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/texture.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

// #pragma once

// #include "rive/refcnt.hpp"

#![allow(dead_code)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use crate::mechanical_port::source::include::rive::refcnt_hpp::{RefCnt, RefCntTarget};
use core::ffi::c_void;
#[cfg(any(
    feature = "native-webgpu-experimental",
    feature = "ore-gl"
))]
use nuxie_ore_metal::gpu_resource::{OwnerThreadFinalRelease, OwnerThreadFinalReleaseRoute};

pub type NativeHandleFn = unsafe fn(*const Texture) -> *mut c_void;

pub(crate) unsafe fn null_native_handle(_: *const Texture) -> *mut c_void {
    core::ptr::null_mut()
}

// namespace rive::gpu
// {
// class Texture : public RefCnt<Texture>
// {
// public:
//     Texture(uint32_t width, uint32_t height);
//     virtual ~Texture() {}
//
//     Texture(const Texture&) = delete;
//     Texture& operator=(const Texture&) = delete;
//
//     uint32_t width() const { return m_width; }
//     uint32_t height() const { return m_height; }
//
//     // Quazi-unique identifier of the underlying GPU texture resource managed by
//     // this class.
//     uint32_t textureResourceHash() const { return m_textureResourceHash; }
//
//     // Returns the backend-native texture handle (id<MTLTexture>, GLuint,
//     // VkImage, etc.) as a void pointer.  Used by
//     // ore::Context::wrapRiveTexture() to bridge Rive renderer images into the
//     // Ore GPU abstraction. Default returns nullptr (backend must override).
//     virtual void* nativeHandle() const { return nullptr; }
//
// protected:
//     uint32_t m_width;
//     uint32_t m_height;
//     uint32_t m_textureResourceHash;
// };
//
// } // namespace rive::gpu

// Rust has no C++ derived-class base subobject. The first field is the
// intrusive `RefCnt<Texture>` base, preserving the source base topology. The
// source fields remain protected through crate visibility and retain their
// declaration order. The field authority records the resource identity as a
// nonzero value allocated by the paired source constructor; the accessor keeps
// the pinned `uint32_t` return type.
#[repr(C)]
pub struct Texture {
    // public RefCnt<Texture> base class
    pub(crate) base: RefCnt<Texture>,
    pub(crate) destroy_complete: unsafe fn(*mut Texture),

    // uint32_t m_width;
    pub(crate) m_width: u32,
    // uint32_t m_height;
    pub(crate) m_height: u32,
    // uint32_t m_textureResourceHash;
    pub(crate) m_textureResourceHash: u32,

    // Explicit Rust vtable slot for the source virtual nativeHandle(). The
    // base constructor installs the null implementation; a derived backend
    // installs its override while constructing the complete object.
    pub(crate) m_nativeHandle: NativeHandleFn,

    // Rust-only safety sidecars after the complete source prefix. GL textures
    // are erased behind `rcp<Texture>` and can reach zero on a worker. The
    // weak route returns destruction to the owner thread, while the scalar
    // identity allows raw native-name adoption to reject a stale or foreign
    // context before invoking `nativeHandle()`.
    #[cfg(any(
        feature = "native-webgpu-experimental",
        feature = "ore-gl"
    ))]
    pub(crate) rust_final_release_route: Option<OwnerThreadFinalReleaseRoute>,
    #[cfg(any(
        feature = "native-webgpu-experimental",
        feature = "ore-gl"
    ))]
    pub(crate) rust_execution_identity: Option<(u64, u64)>,
}

// SAFETY: `base` is the first `#[repr(C)]` field and final release dispatches
// through `destroy_complete` to the live complete texture allocation.
unsafe impl RefCntTarget for Texture {
    // RefCnt's inherited `ref()` operation.
    fn r#ref(&self) {
        self.base.r#ref();
    }

    // RefCnt's inherited `unref()` operation.
    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }

    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        let ptr = ptr.cast_mut();
        #[cfg(any(
            feature = "native-webgpu-experimental",
            feature = "ore-gl"
        ))]
        if let Some(route) = unsafe { &*ptr }.rust_final_release_route.as_ref() {
            unsafe fn destroy_on_owner_thread(payload: usize) {
                let ptr = payload as *mut Texture;
                unsafe { ((*ptr).destroy_complete)(ptr) };
            }
            let release =
                unsafe { OwnerThreadFinalRelease::new(ptr as usize, destroy_on_owner_thread) };
            // A closed route deliberately quarantines the complete allocation
            // rather than running an Rc-backed concrete destructor here.
            let _ = route.release_or_defer(release);
            return;
        }
        unsafe { ((*ptr).destroy_complete)(ptr) };
    }
}

impl Drop for Texture {
    // virtual ~Texture() {}
    // Rust's empty Drop body preserves the source virtual-destruction boundary;
    // the intrusive zero-reference hook owns the allocation release.
    fn drop(&mut self) {}
}

impl Texture {
    // Texture(uint32_t width, uint32_t height);
    // The out-of-line definition belongs to the paired
    // renderer/src/rive_render_image.cpp translation.

    // Texture(const Texture&) = delete;
    // Texture& operator=(const Texture&) = delete;
    // Rust's non-Copy move semantics preserve both deleted source operations.

    // uint32_t width() const { return m_width; }
    pub fn width(&self) -> u32 {
        self.m_width
    }

    // uint32_t height() const { return m_height; }
    pub fn height(&self) -> u32 {
        self.m_height
    }

    // uint32_t textureResourceHash() const { return m_textureResourceHash; }
    pub fn textureResourceHash(&self) -> u32 {
        self.m_textureResourceHash
    }

    // virtual void* nativeHandle() const { return nullptr; }
    // The default backend-native handle is null; concrete texture owners may
    // provide the corresponding `TextureContract` override.
    pub fn nativeHandle(&self) -> *mut c_void {
        unsafe { (self.m_nativeHandle)(self) }
    }

    pub(crate) fn setNativeHandleDispatch(&mut self, dispatch: NativeHandleFn) {
        self.m_nativeHandle = dispatch;
    }

    #[cfg(any(
        feature = "native-webgpu-experimental",
        feature = "ore-gl"
    ))]
    pub(crate) fn install_owner_thread_execution(
        &mut self,
        route: OwnerThreadFinalReleaseRoute,
        domain: u64,
        generation: u64,
    ) {
        assert!(
            self.rust_final_release_route.is_none() && self.rust_execution_identity.is_none(),
            "Texture accepts one owner-thread execution identity"
        );
        self.rust_final_release_route = Some(route);
        self.rust_execution_identity = Some((domain, generation));
    }

    #[cfg(any(
        feature = "native-webgpu-experimental",
        feature = "ore-gl"
    ))]
    pub(crate) fn belongs_to_owner_thread_execution(
        &self,
        domain: u64,
        generation: u64,
    ) -> bool {
        self.rust_execution_identity == Some((domain, generation))
    }
}

// Rust has no virtual member slots. This source-shaped contract preserves the
// overridable native-handle seam while keeping the base implementation's exact
// null default visible to callers of `Texture` itself.
pub trait TextureContract {
    fn nativeHandle(&self) -> *mut c_void {
        core::ptr::null_mut()
    }
}

impl TextureContract for Texture {
    fn nativeHandle(&self) -> *mut c_void {
        Texture::nativeHandle(self)
    }
}
