/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/renderer/ore/ore_buffer.hpp"
// #import <Metal/Metal.h>
// #include <string>
// #include <vector>

// Mechanical translation of the complete pinned source header
// renderer/src/ore/metal/ore_buffer_metal.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;
use std::mem::ManuallyDrop;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::{
    GPUResource, GpuResourcePayload,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::BufferUsage;

// `id<MTLBuffer>` is a nullable, strong Objective-C owner under ARC. Rust's
// `Retained<T>` is the corresponding strong owner; `Option` preserves the
// source `nil` state. The mechanical header is source-shaped and is not wired
// into the runtime module, but the non-Apple stand-in keeps this translation's
// declaration shape available to tools that inspect it off Apple.
#[cfg(target_vendor = "apple")]
use objc2::rc::Retained;
#[cfg(target_vendor = "apple")]
use objc2::runtime::ProtocolObject;
#[cfg(target_vendor = "apple")]
use objc2_metal::{MTLBuffer, MTLDevice};

#[cfg(target_vendor = "apple")]
pub(super) type NativeMetalBuffer = Option<Retained<ProtocolObject<dyn MTLBuffer>>>;

#[cfg(not(target_vendor = "apple"))]
pub(super) type NativeMetalBuffer = Option<()>;

#[cfg(target_vendor = "apple")]
type NativeMetalDevice = Retained<ProtocolObject<dyn MTLDevice>>;

#[cfg(not(target_vendor = "apple"))]
type NativeMetalDevice = ();

pub trait BufferErrorSink: Send + Sync {
    fn setBufferError(&self, message: &str);
}

/// The exact ContextMetal subset consumed by BufferMetal and completion
/// handlers. It replaces the source raw context pointer without retaining the
/// context/resource cycle.
pub struct BufferMetalContextState {
    current: Arc<AtomicU64>,
    completed: Arc<AtomicU64>,
    errorSink: Option<Weak<dyn BufferErrorSink>>,
}

impl BufferMetalContextState {
    pub fn new(errorSink: Option<Weak<dyn BufferErrorSink>>) -> Arc<Self> {
        Self::fromSerials(
            Arc::new(AtomicU64::new(0)),
            Arc::new(AtomicU64::new(0)),
            errorSink,
        )
    }

    pub(crate) fn fromSerials(
        current: Arc<AtomicU64>,
        completed: Arc<AtomicU64>,
        errorSink: Option<Weak<dyn BufferErrorSink>>,
    ) -> Arc<Self> {
        Arc::new(Self {
            current,
            completed,
            errorSink,
        })
    }

    pub fn currentSerial(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    pub fn completedSerial(&self) -> u64 {
        self.completed.load(Ordering::Relaxed)
    }

    pub fn isSerialComplete(&self, serial: u64) -> bool {
        // Literal source ordering: a backing is reusable iff its ordinary
        // uint64 serial is at or below the highest completed serial.
        serial <= self.completedSerial()
    }

    pub(crate) fn setCurrentSerial(&self, serial: u64) {
        let previous = self.current.swap(serial, Ordering::Relaxed);
        debug_assert_eq!(serial, previous.wrapping_add(1));
    }

    #[cfg(test)]
    pub(crate) fn completeSerial(&self, serial: u64) {
        let mut completed = self.completed.load(Ordering::Relaxed);
        while serial > completed {
            match self.completed.compare_exchange_weak(
                completed,
                serial,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(observed) => completed = observed,
            }
        }
    }

    pub fn setLastError(&self, message: &str) {
        if let Some(sink) = self.errorSink.as_ref().and_then(Weak::upgrade) {
            sink.setBufferError(message);
        }
    }
}

// namespace rive::ore

// class ContextMetal;
// class RenderPassMetal;
// Rust retains only the ContextMetal state actually consumed by this buffer;
// complete context and pass definitions belong to their own translation unit.

// Pool of native backings so an update() after a bind orphans onto a fresh
// backing instead of racing the GPU still reading the bound one. Bindings
// resolve the live backing at encode time. Immutable buffers stay at one.
// class BufferMetal : public LITE_RTTI_OVERRIDE(Buffer, BufferMetal)
// {
// Rust has no class inheritance. `base` is the first field to preserve the
// source Buffer base-subobject order. `LITE_RTTI_OVERRIDE(Buffer, BufferMetal)`
// remains the source lite-RTTI identity/override seam and is not duplicated
// as a payload field.
#[repr(C)]
pub struct BufferMetal {
    pub(crate) base: ManuallyDrop<Buffer>,
    // struct Backing
    // {
    //     id<MTLBuffer> mtl = nil;
    //     uint64_t serial = 0; // serial it was last bound in
    // };
    // Rust cannot nest a named struct inside the owner, so `Backing` is the
    // source-shaped sibling immediately below this declaration.
    // Prepared adaptation of `ContextMetal* m_ctx`: retain the thread-safe
    // serial subset and device, and observe the context error sink weakly.
    pub(super) m_state: ManuallyDrop<Mutex<BufferMetalState>>,
    pub(super) m_contextState: ManuallyDrop<Arc<BufferMetalContextState>>,
    pub(super) m_device: ManuallyDrop<NativeMetalDevice>,
}

impl Drop for BufferMetal {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_device);
            ManuallyDrop::drop(&mut self.m_contextState);
            ManuallyDrop::drop(&mut self.m_state);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

// SAFETY: Metal buffer/device retains may cross the completion-thread final
// release. All mutable backing selection and CPU writes are serialized by
// `m_state`; Metal device allocation and retain/release are thread-safe.
unsafe impl Send for BufferMetal {}

unsafe impl GpuResourcePayload for BufferMetal {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base.base
    }
}

pub(super) struct BufferMetalState {
    // Declaration order models source reverse destruction: label first, then
    // scalar state, then the retained backing pool.
    pub(super) m_label: String,
    pub(super) m_boundSinceUpdate: bool,
    pub(super) m_currentIndex: usize,
    pub(super) m_pool: Vec<Backing>,
}

// Source nested `BufferMetal::Backing`; field order and defaults match the
// pinned declaration. `NativeMetalBuffer` retains every non-nil Objective-C
// backing until the enclosing pool drops it.
#[derive(Clone)]
pub(super) struct Backing {
    pub(super) mtl: NativeMetalBuffer,
    pub(super) serial: u64,
}

impl Default for Backing {
    fn default() -> Self {
        Self {
            mtl: None,
            serial: 0,
        }
    }
}

impl BufferMetal {
    // public:
    // BufferMetal(uint32_t size, BufferUsage usage) :
    //     lite_rtti_override(size, usage)
    // {}
    // The source lite-RTTI initializer delegates to the Buffer base
    // constructor and records the concrete BufferMetal identity.
    pub(crate) fn new(
        size: u32,
        usage: BufferUsage,
        device: NativeMetalDevice,
        contextState: Arc<BufferMetalContextState>,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(Buffer::new(size, usage)),
            m_state: ManuallyDrop::new(Mutex::new(BufferMetalState {
                m_label: String::new(),
                m_boundSinceUpdate: false,
                m_currentIndex: 0,
                m_pool: Vec::new(),
            })),
            m_contextState: ManuallyDrop::new(contextState),
            m_device: ManuallyDrop::new(device),
        }
    }

    // ~BufferMetal() override = default; // ARC releases the backings
    // Rust's default drop glue releases the retained native backing owners in
    // `m_pool` and then the remaining source-shaped fields.

    // void update(const void* data, uint32_t size, uint32_t offset) override;
    // The paired ore_buffer_metal.mm translation owns the complete update
    // implementation. The source `const void*` is represented there as a
    // borrowed byte slice while `size` and `offset` remain explicit.

    // Backing to bind right now.
    // id<MTLBuffer> current() const { return m_pool[m_currentIndex].mtl; }
    pub fn current(&self) -> NativeMetalBuffer {
        let state = self.lockState();
        state.m_pool[state.m_currentIndex].mtl.clone()
    }

    pub(super) fn lockState(&self) -> MutexGuard<'_, BufferMetalState> {
        self.m_state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    pub(crate) fn initializeBacking(&mut self, mtl: NativeMetalBuffer, label: Option<&str>) {
        let state = self
            .m_state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        state.m_label = label.unwrap_or_default().to_owned();
        state.m_pool.push(Backing { mtl, serial: 0 });
    }

    // Tag the current backing with this frame's serial so a later update()
    // orphans instead of overwriting in-flight memory.
    // void markBound();
    // Defined by the paired ore_buffer_metal.mm translation.

    // private:
    // friend class ContextMetal;
    // friend class RenderPassMetal;

    // Move to a fresh backing, copying current contents so a partial update
    // keeps untouched bytes. The pending write's range lets a full-buffer
    // update skip the copy.
    // Returns false if a fresh backing could not be allocated, in which case
    // the current backing is kept.
    // bool acquireFreshBacking(uint32_t writeOffset, uint32_t writeSize);
    // Defined by the paired ore_buffer_metal.mm translation.
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_resource::GPUResourceManagerOwner;
    use crate::types::BufferUsage;

    #[cfg(target_vendor = "apple")]
    fn live_buffer(size: u32, label: Option<&str>) -> Option<BufferMetal> {
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice, MTLResourceOptions};

        let Some(device) = MTLCreateSystemDefaultDevice() else {
            crate::live_metal_test_unavailable("system Metal device for ORE buffer");
            return None;
        };
        let initial = device
            .newBufferWithLength_options(size as usize, MTLResourceOptions::StorageModeShared)?;
        let mut buffer = BufferMetal::new(
            size,
            BufferUsage::uniform,
            device,
            BufferMetalContextState::new(None),
        );
        buffer.initializeBacking(Some(initial), label);
        Some(buffer)
    }

    #[cfg(target_vendor = "apple")]
    fn current_bytes(buffer: &BufferMetal) -> Vec<u8> {
        let native = buffer.current().expect("initialized Metal backing");
        unsafe {
            std::slice::from_raw_parts(
                native.contents().as_ptr().cast::<u8>(),
                buffer.base.size() as usize,
            )
            .to_vec()
        }
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn live_unbound_update_writes_the_current_backing_in_place() {
        let Some(buffer) = live_buffer(8, None) else {
            return;
        };
        let before = buffer.current().expect("current backing");
        let resource: &dyn BufferApi = &buffer;
        assert_eq!(resource.size(), 8);
        assert_eq!(resource.usage(), BufferUsage::uniform);
        resource.update(&[1, 2, 3, 4], 4, 2).expect("update");
        let after = buffer.current().expect("current backing");
        assert_eq!(Retained::as_ptr(&before), Retained::as_ptr(&after));
        assert_eq!(current_bytes(&buffer), [0, 0, 1, 2, 3, 4, 0, 0]);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn live_bound_partial_update_orphans_copies_and_reuses_completed_backing() {
        let Some(buffer) = live_buffer(8, Some("versioned")) else {
            return;
        };
        buffer
            .update(&[1, 2, 3, 4, 5, 6, 7, 8], 8, 0)
            .expect("seed contents");
        let first = buffer.current().expect("current backing");

        buffer.m_contextState.setCurrentSerial(1);
        buffer.markBound();
        buffer.update(&[9, 10], 2, 3).expect("partial orphan");
        let second = buffer.current().expect("current backing");
        assert_ne!(Retained::as_ptr(&first), Retained::as_ptr(&second));
        assert_eq!(buffer.lockState().m_label, "versioned");
        assert_eq!(current_bytes(&buffer), [1, 2, 3, 9, 10, 6, 7, 8]);

        buffer.m_contextState.setCurrentSerial(2);
        buffer.markBound();
        buffer.m_contextState.completeSerial(1);
        buffer
            .update(&[11, 12, 13, 14, 15, 16, 17, 18], 8, 0)
            .expect("reuse completed backing");
        let third = buffer.current().expect("current backing");
        assert_eq!(Retained::as_ptr(&first), Retained::as_ptr(&third));
        assert_eq!(current_bytes(&buffer), [11, 12, 13, 14, 15, 16, 17, 18]);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn bound_backing_token_remains_the_exact_backing_after_an_orphaning_update() {
        let Some(buffer) = live_buffer(8, None) else {
            return;
        };
        let bound = buffer.currentAndMarkBound().expect("bound backing");

        buffer
            .update(&[9], 1, 0)
            .expect("a post-bind partial update orphans the backing");

        assert_ne!(
            Retained::as_ptr(&bound),
            Retained::as_ptr(&buffer.current().expect("current backing")),
            "the encode token must keep naming the backing marked bound"
        );
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn update_rejects_out_of_bounds_without_mutating_contents() {
        let Some(buffer) = live_buffer(8, None) else {
            return;
        };
        let error = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            buffer.update(&[1, 2], 2, 7)
        }));
        assert!(
            error.is_err(),
            "debug source assert rejects an out-of-bounds update"
        );
        assert_eq!(current_bytes(&buffer), [0; 8]);

        let overflow = buffer
            .update(&[1, 2], 2, u32::MAX)
            .expect_err("offset addition must fail closed");
        assert_eq!(overflow, BufferUpdateError::RangeOverflow);
        assert_eq!(current_bytes(&buffer), [0; 8]);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn allocation_failure_reports_error_keeps_current_and_retries_next_update() {
        let Some(buffer) = live_buffer(4, None) else {
            return;
        };
        buffer.update(&[1, 2, 3, 4], 4, 0).expect("seed contents");
        let first = buffer.current().expect("current backing");
        buffer.m_contextState.setCurrentSerial(1);
        buffer.markBound();
        buffer.update(&[9], 1, 0).expect("update still writes");
        assert_ne!(
            Retained::as_ptr(&first),
            Retained::as_ptr(&buffer.current().expect("current backing"))
        );
        assert_eq!(current_bytes(&buffer), [9, 2, 3, 4]);
        buffer
            .update(&[8], 1, 1)
            .expect("next update remains source-valid");
        assert_eq!(current_bytes(&buffer), [9, 8, 3, 4]);
    }

    #[test]
    fn serials_preserve_literal_source_ordering_through_rollover() {
        let context_state = BufferMetalContextState::new(None);
        context_state.setCurrentSerial(1);
        context_state.setCurrentSerial(2);
        context_state.setCurrentSerial(3);
        context_state.setCurrentSerial(4);
        context_state.completeSerial(2);
        context_state.completeSerial(1);
        assert_eq!(context_state.currentSerial(), 4);
        assert_eq!(context_state.completedSerial(), 2);

        context_state.setCurrentSerial(5);
        context_state.completeSerial(5);
        assert_eq!(context_state.currentSerial(), 5);
        assert_eq!(context_state.completedSerial(), 5);
    }

    #[cfg(target_vendor = "apple")]
    #[test]
    fn buffer_resource_retains_manager_and_rejects_wrong_backend() {
        let Some(buffer) = live_buffer(4, None) else {
            return;
        };
        let resource: &dyn BufferApi = &buffer;
        assert_eq!(resource.size(), 4);
        assert_eq!(resource.usage(), BufferUsage::uniform);

        let context_state = Arc::downgrade(&*buffer.m_contextState);
        let owner = GPUResourceManagerOwner::new();
        let handle = crate::gpu_resource::ResourceHandle::new(Some(owner.manager()), buffer);
        assert!(handle.manager().is_some());
        let clone = handle.clone();
        assert!(handle.ptr_eq(&clone));
        drop(handle);
        assert!(context_state.upgrade().is_some());
        drop(clone);
        assert!(
            context_state.upgrade().is_some(),
            "manager purgatory retains the buffer payload before the first safe frame"
        );
        owner.shutdown();
        assert!(context_state.upgrade().is_none());
    }
}
