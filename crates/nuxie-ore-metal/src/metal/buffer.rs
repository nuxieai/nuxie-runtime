// Mechanical translation of:
// - renderer/src/ore/metal/ore_buffer_metal.hpp
// - renderer/src/ore/metal/ore_buffer_metal.mm
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2025 Rive

#![allow(non_snake_case)]

use std::any::Any;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(any(target_os = "ios", target_os = "macos"))]
use std::sync::{Mutex, MutexGuard};

use crate::buffer::{BufferBase, BufferUpdateError};
use crate::gpu_resource::{GpuResourceManager, ResourceHandle};
use crate::types::{BackendId, Buffer as BufferResource, BufferUsage};

use super::MetalBackend;

#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::rc::Retained;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2::runtime::ProtocolObject;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_foundation::NSString;
#[cfg(any(target_os = "ios", target_os = "macos"))]
use objc2_metal::{MTLBuffer, MTLDevice, MTLResource, MTLResourceOptions, MTLStorageMode};

#[cfg(any(test, target_os = "ios", target_os = "macos"))]
const ALLOCATION_FAILURE: &str =
    "ore: Metal buffer backing allocation failed; reusing in flight backing for this update";

/// Serial/error state shared with the pending `ContextMetal` translation.
///
/// Upstream stores a raw `ContextMetal*` in each buffer. Rust retains only the
/// exact state the buffer reads or writes, avoiding a context/resource cycle
/// and keeping completion-thread serial publication alive independently.
pub(crate) struct BufferMetalContextState {
    current: AtomicU64,
    completed: AtomicU64,
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    last_error: Mutex<Option<String>>,
}

impl BufferMetalContextState {
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the pending context unit creates shared serial state"
        )
    )]
    pub(crate) fn new() -> Arc<Self> {
        Arc::new(Self {
            current: AtomicU64::new(0),
            completed: AtomicU64::new(0),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            last_error: Mutex::new(None),
        })
    }

    pub(crate) fn current_serial(&self) -> u64 {
        self.current.load(Ordering::Relaxed)
    }

    #[cfg(any(test, target_os = "ios", target_os = "macos"))]
    pub(crate) fn completed_serial(&self) -> u64 {
        self.completed.load(Ordering::Relaxed)
    }

    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the pending context unit advances this serial")
    )]
    pub(crate) fn set_current_serial(&self, serial: u64) {
        let result = self
            .current
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |previous| {
                (serial >= previous).then_some(serial)
            });
        assert!(result.is_ok(), "Metal buffer serial must be monotonic");
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the pending context completion handler advances this serial"
        )
    )]
    pub(crate) fn complete_serial(&self, serial: u64) {
        assert!(
            serial <= self.current_serial(),
            "completed Metal buffer serial cannot exceed current serial"
        );
        self.completed.fetch_max(serial, Ordering::Relaxed);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the pending context unit consumes this error")
    )]
    pub(crate) fn take_last_error(&self) -> Option<String> {
        self.lock_last_error().take()
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    fn set_last_error(&self, message: &str) {
        *self.lock_last_error() = Some(message.to_owned());
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    fn lock_last_error(&self) -> MutexGuard<'_, Option<String>> {
        self.last_error
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Default for BufferMetalContextState {
    fn default() -> Self {
        Self {
            current: AtomicU64::new(0),
            completed: AtomicU64::new(0),
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            last_error: Mutex::new(None),
        }
    }
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct RetainedMetalBuffer(Retained<ProtocolObject<dyn MTLBuffer>>);

// SAFETY: MTLBuffer supports concurrent retain/release and GPU binding. CPU
// writes and backing selection remain serialized by `BufferMetal::state`.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Send for RetainedMetalBuffer {}
// SAFETY: Same ownership and mutation invariant as the `Send` implementation.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Sync for RetainedMetalBuffer {}

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct RetainedMetalDevice(Retained<ProtocolObject<dyn MTLDevice>>);

// SAFETY: MTLDevice is a process-wide thread-safe factory. The wrapper only
// exposes shared allocation calls and concurrent retain/release.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Send for RetainedMetalDevice {}
// SAFETY: Same invariant as the `Send` implementation above.
#[cfg(any(target_os = "ios", target_os = "macos"))]
unsafe impl Sync for RetainedMetalDevice {}

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct Backing {
    mtl: RetainedMetalBuffer,
    serial: u64,
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
struct BufferMetalState {
    pool: Vec<Backing>,
    current_index: usize,
    bound_since_update: bool,
    label: String,
    #[cfg(test)]
    fail_next_allocation: bool,
}

/// Concrete Metal buffer with orphan-on-update backing reuse.
pub struct BufferMetal {
    base: BufferBase,
    context_state: Arc<BufferMetalContextState>,
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    device: RetainedMetalDevice,
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    state: Mutex<BufferMetalState>,
}

impl BufferMetal {
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the pending context unit constructs native buffers"
        )
    )]
    pub(crate) fn with_native_buffer(
        size: u32,
        usage: BufferUsage,
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        initial: Retained<ProtocolObject<dyn MTLBuffer>>,
        context_state: Arc<BufferMetalContextState>,
        label: Option<&str>,
    ) -> Self {
        assert_eq!(
            initial.length(),
            size as usize,
            "ContextMetal must publish a backing with the declared buffer size"
        );
        assert_eq!(
            initial.storageMode(),
            MTLStorageMode::Shared,
            "ContextMetal must publish a shared-storage buffer"
        );
        Self {
            base: BufferBase::new(size, usage),
            context_state,
            device: RetainedMetalDevice(device),
            state: Mutex::new(BufferMetalState {
                pool: vec![Backing {
                    mtl: RetainedMetalBuffer(initial),
                    serial: 0,
                }],
                current_index: 0,
                bound_since_update: false,
                label: label.unwrap_or_default().to_owned(),
                #[cfg(test)]
                fail_next_allocation: false,
            }),
        }
    }

    pub fn base(&self) -> &BufferBase {
        &self.base
    }

    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the pending context unit retains shared serial state"
        )
    )]
    pub(crate) fn context_state(&self) -> &Arc<BufferMetalContextState> {
        &self.context_state
    }

    pub fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[expect(
        clippy::indexing_slicing,
        reason = "current_index is initialized to an existing backing and only reassigned to an existing or newly pushed backing"
    )]
    fn update_inner(&self, data: &[u8], offset: u32) -> Result<(), BufferUpdateError> {
        let write_size = u32::try_from(data.len()).map_err(|_| BufferUpdateError::SizeOverflow)?;
        let end = offset
            .checked_add(write_size)
            .ok_or(BufferUpdateError::OutOfBounds {
                offset,
                size: write_size,
                buffer_size: self.base.size(),
            })?;
        if end > self.base.size() {
            return Err(BufferUpdateError::OutOfBounds {
                offset,
                size: write_size,
                buffer_size: self.base.size(),
            });
        }

        let mut state = self.lock_state();
        if state.bound_since_update && self.acquire_fresh_backing(&mut state, offset, write_size) {
            state.bound_since_update = false;
        }

        let current = &state.pool[state.current_index].mtl.0;
        // SAFETY: range validation above proves `offset + data.len()` is at
        // most the native buffer length created from `base.size()`. Metal
        // shared-storage contents stays valid for the retained buffer, and the
        // state lock serializes this CPU write with backing selection.
        unsafe {
            std::ptr::copy_nonoverlapping(
                data.as_ptr(),
                current
                    .contents()
                    .as_ptr()
                    .cast::<u8>()
                    .add(offset as usize),
                data.len(),
            );
        }
        Ok(())
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the pending render-pass unit marks bound buffers")
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "current_index always names an owned backing in the nonempty pool"
    )]
    pub(crate) fn mark_bound(&self) {
        let mut state = self.lock_state();
        let current_index = state.current_index;
        state.pool[current_index].serial = self.context_state.current_serial();
        state.bound_since_update = true;
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[cfg_attr(
        not(test),
        expect(dead_code, reason = "the pending render-pass unit binds this owner")
    )]
    #[expect(
        clippy::indexing_slicing,
        reason = "current_index always names an owned backing in the nonempty pool"
    )]
    pub(crate) fn current_buffer(&self) -> Retained<ProtocolObject<dyn MTLBuffer>> {
        let state = self.lock_state();
        state.pool[state.current_index].mtl.0.clone()
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[expect(
        clippy::indexing_slicing,
        reason = "current_index is valid on entry and fresh is either an existing index or the index appended immediately below"
    )]
    fn acquire_fresh_backing(
        &self,
        state: &mut BufferMetalState,
        write_offset: u32,
        write_size: u32,
    ) -> bool {
        let old = state.pool[state.current_index].mtl.0.clone();
        let completed = self.context_state.completed_serial();
        let mut fresh = state.pool.len();
        for (index, backing) in state.pool.iter().enumerate() {
            if index != state.current_index && backing.serial <= completed {
                fresh = index;
                break;
            }
        }

        if fresh == state.pool.len() {
            #[cfg(test)]
            if std::mem::take(&mut state.fail_next_allocation) {
                self.context_state.set_last_error(ALLOCATION_FAILURE);
                return false;
            }
            let Some(mtl) = self.device.0.newBufferWithLength_options(
                self.base.size() as usize,
                MTLResourceOptions::StorageModeShared,
            ) else {
                self.context_state.set_last_error(ALLOCATION_FAILURE);
                return false;
            };
            if !state.label.is_empty() {
                mtl.setLabel(Some(&NSString::from_str(&state.label)));
            }
            state.pool.push(Backing {
                mtl: RetainedMetalBuffer(mtl),
                serial: 0,
            });
        }

        state.current_index = fresh;
        if !(write_offset == 0 && write_size == self.base.size()) {
            let current = &state.pool[state.current_index].mtl.0;
            // SAFETY: both retained Metal buffers were allocated at exactly
            // `base.size()` bytes in shared storage. They are distinct pool
            // entries, and the state lock prevents concurrent CPU mutation.
            unsafe {
                std::ptr::copy_nonoverlapping(
                    old.contents().as_ptr().cast::<u8>(),
                    current.contents().as_ptr().cast::<u8>(),
                    self.base.size() as usize,
                );
            }
        }
        true
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    fn lock_state(&self) -> MutexGuard<'_, BufferMetalState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    #[cfg(all(test, any(target_os = "ios", target_os = "macos")))]
    fn current_bytes(&self) -> Vec<u8> {
        let current = self.current_buffer();
        // SAFETY: the retained buffer owns `base.size()` bytes of shared
        // storage for the duration of this immediate test snapshot.
        unsafe {
            std::slice::from_raw_parts(
                current.contents().as_ptr().cast::<u8>(),
                self.base.size() as usize,
            )
            .to_vec()
        }
    }

    #[cfg(all(test, any(target_os = "ios", target_os = "macos")))]
    fn fail_next_allocation_for_test(&self) {
        self.lock_state().fail_next_allocation = true;
    }
}

impl BufferResource for BufferMetal {
    fn backend_id(&self) -> BackendId {
        BackendId::of::<MetalBackend>()
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }

    fn size(&self) -> u32 {
        self.base.size()
    }

    fn usage(&self) -> BufferUsage {
        self.base.usage()
    }

    fn update(&self, data: &[u8], offset: u32) -> Result<(), BufferUpdateError> {
        #[cfg(any(target_os = "ios", target_os = "macos"))]
        {
            self.update_inner(data, offset)
        }

        #[cfg(not(any(target_os = "ios", target_os = "macos")))]
        {
            let _ = (data, offset);
            Err(BufferUpdateError::UnsupportedPlatform)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gpu_resource::GpuResourceManagerOwner;
    use crate::types::BufferUsage;

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    fn live_buffer(size: u32, label: Option<&str>) -> Option<BufferMetal> {
        use objc2_metal::{MTLCreateSystemDefaultDevice, MTLDevice, MTLResourceOptions};

        let device = MTLCreateSystemDefaultDevice()?;
        let initial = device
            .newBufferWithLength_options(size as usize, MTLResourceOptions::StorageModeShared)?;
        Some(BufferMetal::with_native_buffer(
            size,
            BufferUsage::uniform,
            device,
            initial,
            BufferMetalContextState::new(),
            label,
        ))
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_unbound_update_writes_the_current_backing_in_place() {
        let Some(buffer) = live_buffer(8, None) else {
            return;
        };
        let before = buffer.current_buffer();
        let resource: &dyn BufferResource = &buffer;
        assert_eq!(resource.size(), 8);
        assert_eq!(resource.usage(), BufferUsage::uniform);
        resource.update(&[1, 2, 3, 4], 2).expect("update");
        let after = buffer.current_buffer();
        assert_eq!(Retained::as_ptr(&before), Retained::as_ptr(&after));
        assert_eq!(buffer.current_bytes(), [0, 0, 1, 2, 3, 4, 0, 0]);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn live_bound_partial_update_orphans_copies_and_reuses_completed_backing() {
        let Some(buffer) = live_buffer(8, Some("versioned")) else {
            return;
        };
        buffer
            .update(&[1, 2, 3, 4, 5, 6, 7, 8], 0)
            .expect("seed contents");
        let first = buffer.current_buffer();

        buffer.context_state().set_current_serial(1);
        buffer.mark_bound();
        buffer.update(&[9, 10], 3).expect("partial orphan");
        let second = buffer.current_buffer();
        assert_ne!(Retained::as_ptr(&first), Retained::as_ptr(&second));
        assert_eq!(
            second.label().as_deref().map(ToString::to_string),
            Some("versioned".to_owned())
        );
        assert_eq!(buffer.current_bytes(), [1, 2, 3, 9, 10, 6, 7, 8]);

        buffer.context_state().set_current_serial(2);
        buffer.mark_bound();
        buffer.context_state().complete_serial(1);
        buffer
            .update(&[11, 12, 13, 14, 15, 16, 17, 18], 0)
            .expect("reuse completed backing");
        let third = buffer.current_buffer();
        assert_eq!(Retained::as_ptr(&first), Retained::as_ptr(&third));
        assert_eq!(buffer.current_bytes(), [11, 12, 13, 14, 15, 16, 17, 18]);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn update_rejects_out_of_bounds_without_mutating_contents() {
        let Some(buffer) = live_buffer(8, None) else {
            return;
        };
        let error = buffer.update(&[1, 2], 7).expect_err("range must fail");
        assert_eq!(
            error,
            BufferUpdateError::OutOfBounds {
                offset: 7,
                size: 2,
                buffer_size: 8,
            }
        );
        assert_eq!(buffer.current_bytes(), [0; 8]);

        let overflow = buffer
            .update(&[1, 2], u32::MAX)
            .expect_err("offset addition must fail closed");
        assert_eq!(
            overflow,
            BufferUpdateError::OutOfBounds {
                offset: u32::MAX,
                size: 2,
                buffer_size: 8,
            }
        );
        assert_eq!(buffer.current_bytes(), [0; 8]);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn allocation_failure_reports_error_keeps_current_and_retries_next_update() {
        let Some(buffer) = live_buffer(4, None) else {
            return;
        };
        buffer.update(&[1, 2, 3, 4], 0).expect("seed contents");
        let first = buffer.current_buffer();
        buffer.context_state().set_current_serial(1);
        buffer.mark_bound();
        buffer.fail_next_allocation_for_test();

        buffer
            .update(&[9], 0)
            .expect("degraded update still writes");
        assert_eq!(
            Retained::as_ptr(&first),
            Retained::as_ptr(&buffer.current_buffer())
        );
        assert_eq!(buffer.current_bytes(), [9, 2, 3, 4]);
        assert_eq!(
            buffer.context_state().take_last_error().as_deref(),
            Some(ALLOCATION_FAILURE)
        );

        buffer
            .update(&[8], 1)
            .expect("next update retries allocation");
        assert_ne!(
            Retained::as_ptr(&first),
            Retained::as_ptr(&buffer.current_buffer())
        );
        assert_eq!(buffer.current_bytes(), [9, 8, 3, 4]);
    }

    #[test]
    fn serials_are_monotonic_and_reject_impossible_completion() {
        let context_state = BufferMetalContextState::new();
        context_state.set_current_serial(4);
        context_state.complete_serial(2);
        context_state.complete_serial(1);
        assert_eq!(context_state.current_serial(), 4);
        assert_eq!(context_state.completed_serial(), 2);

        assert!(
            std::panic::catch_unwind({
                let context_state = Arc::clone(&context_state);
                move || context_state.set_current_serial(3)
            })
            .is_err()
        );
        assert_eq!(context_state.current_serial(), 4);
        assert!(
            std::panic::catch_unwind({
                let context_state = Arc::clone(&context_state);
                move || context_state.complete_serial(5)
            })
            .is_err()
        );
        assert_eq!(context_state.completed_serial(), 2);
    }

    #[cfg(any(target_os = "ios", target_os = "macos"))]
    #[test]
    fn buffer_resource_retains_manager_and_rejects_wrong_backend() {
        let Some(buffer) = live_buffer(4, None) else {
            return;
        };
        let resource: &dyn BufferResource = &buffer;
        assert_eq!(resource.backend_id(), BackendId::of::<MetalBackend>());
        enum OtherBackend {}
        assert!(
            resource
                .downcast_ref::<BufferMetal>(BackendId::of::<OtherBackend>())
                .is_none()
        );

        let context_state = Arc::downgrade(buffer.context_state());
        let owner = GpuResourceManagerOwner::new();
        let handle = buffer.into_resource(Some(owner.manager()));
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
