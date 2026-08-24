/*
 * Copyright 2025 Rive
 */

// #pragma once
// #include "rive/refcnt.hpp"
// #include <deque>

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/gpu_resource.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

use std::any::{Any, TypeId};
use std::collections::VecDeque;
use std::fmt;
use std::marker::PhantomData;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use super::ore::ore_bind_group_layout_hpp::BindGroupLayout;
use super::ore::ore_buffer_hpp::{BufferApi, BufferUpdateError};
use super::ore::ore_pipeline_hpp::Pipeline;
use super::ore::ore_texture_hpp::{TextureApi, TextureUploadError};
use super::ore::ore_types_hpp::{BufferUsage, TextureDataDesc, TextureFormat, TextureType};

// namespace rive::gpu

pub(crate) const SHUTDOWN_FRAME_NUMBER: u64 = u64::MAX;

/// Rust safety sidecar shared by one source `Context` and any backend
/// execution authority that must finish concrete resource destruction after
/// the ORE context itself has gone away.
struct ResourceDomainFinalReleaseState {
    owner_thread: std::thread::ThreadId,
    always_defer: Arc<AtomicBool>,
    closed: AtomicBool,
    drain_active: AtomicBool,
    bound_execution_domain: AtomicU64,
    deferred_final_releases: Mutex<VecDeque<DeferredFinalRelease>>,
    wake: Mutex<Option<Arc<dyn ResourceFinalReleaseWake>>>,
}

/// Host/event-loop notification used when an owner-thread final release is
/// enqueued. Implementations must post asynchronously to the creation thread;
/// they must never invoke the installed drain ingress inline from `post()`.
pub trait ResourceFinalReleaseWake: Send + Sync {
    fn post(&self);
}

enum DeferredFinalRelease {
    Resource(ResourceOwner),
    OwnerThread(OwnerThreadFinalRelease),
}

/// Type-erased final destruction that may touch a thread-affine backend only
/// after its execution authority has made the owner thread current.
///
/// Dropping this value is intentionally inert. If its route is already closed
/// or gone, callers quarantine the pointed-to allocation by abandoning this
/// token rather than executing the callback on the wrong thread.
pub struct OwnerThreadFinalRelease {
    payload: usize,
    release: unsafe fn(usize),
}

// SAFETY: the payload is never dereferenced while crossing threads. `release`
// runs only from ResourceFinalReleaseDrain::drain on the recorded owner thread.
unsafe impl Send for OwnerThreadFinalRelease {}

impl OwnerThreadFinalRelease {
    /// # Safety
    /// `payload` must remain valid until `release(payload)` runs, and the
    /// callback must consume that ownership exactly once.
    pub unsafe fn new(payload: usize, release: unsafe fn(usize)) -> Self {
        Self { payload, release }
    }

    unsafe fn run(self) {
        unsafe { (self.release)(self.payload) };
    }
}

/// Opaque execution-domain identity for safe Rust resource entry points.
/// Pinned C++ carries this as the implicit `Context*`/backend precondition;
/// the identity weak token preserves that non-owning relationship without
/// widening it. The separate final-release route may intentionally outlive
/// that source-context identity.
#[derive(Clone)]
pub struct ResourceDomain {
    identity: Weak<()>,
    final_release_route: Weak<ResourceDomainFinalReleaseState>,
    always_defer_final_release: Arc<AtomicBool>,
}

impl ResourceDomain {
    pub(crate) fn new(identity: &Arc<()>, owner: &ResourceFinalReleaseDrain) -> Self {
        Self {
            identity: Arc::downgrade(identity),
            final_release_route: Arc::downgrade(&owner.state),
            always_defer_final_release: Arc::clone(&owner.state.always_defer),
        }
    }

    fn matches(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.identity, &other.identity) && self.identity.strong_count() != 0
    }

    fn should_defer_final_release(&self, is_recording_thread: bool) -> bool {
        !is_recording_thread || self.always_defer_final_release.load(Ordering::Acquire)
    }

    fn assert_recording_thread(&self) {
        let Some(state) = self.final_release_route.upgrade() else {
            panic!("cannot construct a GPU resource after its final-release route expired");
        };
        assert_eq!(
            state.owner_thread,
            std::thread::current().id(),
            "domain GPU resources must be constructed on the recording thread"
        );
        assert!(
            !state.closed.load(Ordering::Acquire),
            "cannot construct a GPU resource after final-release shutdown"
        );
    }

    fn defer_final_release(&self, owner: ResourceOwner) -> Result<(), ResourceOwner> {
        let Some(state) = self.final_release_route.upgrade() else {
            return Err(owner);
        };
        match state.enqueue(DeferredFinalRelease::Resource(owner)) {
            Ok(()) => Ok(()),
            Err(DeferredFinalRelease::Resource(owner)) => Err(owner),
            Err(DeferredFinalRelease::OwnerThread(_)) => unreachable!(),
        }
    }
}

impl ResourceDomainFinalReleaseState {
    fn enqueue(&self, release: DeferredFinalRelease) -> Result<(), DeferredFinalRelease> {
        let wake = {
            let mut queue = self
                .deferred_final_releases
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if self.closed.load(Ordering::Acquire)
                && !(self.drain_active.load(Ordering::Acquire)
                    && self.owner_thread == std::thread::current().id())
            {
                return Err(release);
            }
            queue.push_back(release);
            self.wake
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone()
        };
        if let Some(wake) = wake {
            wake.post();
        }
        Ok(())
    }

    fn close(&self) {
        let _queue = self
            .deferred_final_releases
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        self.closed.store(true, Ordering::Release);
    }
}

/// Cloneable Rust safety-sidecar handle for owner-thread final destruction.
///
/// Backends whose API teardown must run under an additional execution scope
/// retain this handle beyond the source `Context`, then call `drain()` inside
/// that scope before destroying the device/context. The caller must stop all
/// final-release producers before its terminal drain.
#[derive(Clone)]
pub struct ResourceFinalReleaseDrain {
    state: Arc<ResourceDomainFinalReleaseState>,
}

/// Non-forgeable authority returned exactly once when a backend binds a
/// final-release route to its execution domain. Raw drain clones deliberately
/// do not carry this token, so they cannot close or consume a GL-owned FIFO.
pub struct ResourceFinalReleaseExecutionDomain {
    state: Weak<ResourceDomainFinalReleaseState>,
    domain: u64,
}

/// Send-safe weak route installed in source-shaped intrusive owners whose
/// concrete destructor is thread-affine. It never keeps the backend alive by
/// itself and never executes a callback while enqueueing.
#[derive(Clone)]
pub struct OwnerThreadFinalReleaseRoute {
    state: Weak<ResourceDomainFinalReleaseState>,
}

impl OwnerThreadFinalReleaseRoute {
    /// Preserve source RAII order on the owning thread and route only a
    /// genuinely cross-thread last release through the host FIFO. The
    /// concrete callback is allowed to establish any additional backend
    /// execution scope retained by its allocation.
    pub fn release_or_defer(
        &self,
        release: OwnerThreadFinalRelease,
    ) -> Result<(), OwnerThreadFinalRelease> {
        let Some(state) = self.state.upgrade() else {
            return Err(release);
        };
        if state.owner_thread == std::thread::current().id() {
            {
                let _queue = state
                    .deferred_final_releases
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if state.closed.load(Ordering::Acquire)
                    && !state.drain_active.load(Ordering::Acquire)
                {
                    return Err(release);
                }
            }
            unsafe { release.run() };
            return Ok(());
        }
        match state.enqueue(DeferredFinalRelease::OwnerThread(release)) {
            Ok(()) => Ok(()),
            Err(DeferredFinalRelease::OwnerThread(release)) => Err(release),
            Err(DeferredFinalRelease::Resource(_)) => unreachable!(),
        }
    }

    pub fn defer(&self, release: OwnerThreadFinalRelease) -> Result<(), OwnerThreadFinalRelease> {
        let Some(state) = self.state.upgrade() else {
            return Err(release);
        };
        match state.enqueue(DeferredFinalRelease::OwnerThread(release)) {
            Ok(()) => Ok(()),
            Err(DeferredFinalRelease::OwnerThread(release)) => Err(release),
            Err(DeferredFinalRelease::Resource(_)) => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceFinalReleaseBindError {
    AlreadyBound { existing_domain: u64 },
}

/// A final-release drain is rejected without touching its FIFO when invoked
/// from any thread other than the domain's recording thread.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceFinalReleaseDrainError {
    WrongThread,
    WrongExecutionDomain {
        expected_domain: u64,
        actual_domain: u64,
    },
}

impl ResourceFinalReleaseDrain {
    #[doc(hidden)]
    pub fn new() -> Self {
        Self {
            state: Arc::new(ResourceDomainFinalReleaseState {
                owner_thread: std::thread::current().id(),
                always_defer: Arc::new(AtomicBool::new(false)),
                closed: AtomicBool::new(false),
                drain_active: AtomicBool::new(false),
                bound_execution_domain: AtomicU64::new(0),
                deferred_final_releases: Mutex::new(VecDeque::new()),
                wake: Mutex::new(None),
            }),
        }
    }

    pub(crate) fn resource_domain(&self, identity: &Arc<()>) -> ResourceDomain {
        ResourceDomain::new(identity, self)
    }

    /// Select the WebGL-style routing policy. This is deliberately one-way:
    /// once a backend requires an ambient execution scope for destruction,
    /// every final release remains queued for that scope.
    pub fn set_always_defer(&self) {
        self.state.always_defer.store(true, Ordering::Release);
    }

    /// Install the backend host notification that schedules an asynchronous
    /// owner-thread drain. Installation is one-shot and deliberately separate
    /// from the non-forgeable close/drain authority.
    pub fn install_wake(&self, wake: Arc<dyn ResourceFinalReleaseWake>) {
        let mut installed = self
            .state
            .wake
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert!(
            installed.is_none(),
            "a final-release route accepts one host wake"
        );
        *installed = Some(wake);
    }

    pub fn bind_execution_domain(
        &self,
        domain: u64,
    ) -> Result<ResourceFinalReleaseExecutionDomain, ResourceFinalReleaseBindError> {
        assert_ne!(domain, 0, "zero is not a GL execution-domain identity");
        match self.state.bound_execution_domain.compare_exchange(
            0,
            domain,
            Ordering::AcqRel,
            Ordering::Acquire,
        ) {
            Ok(_) => Ok(ResourceFinalReleaseExecutionDomain {
                state: Arc::downgrade(&self.state),
                domain,
            }),
            Err(existing_domain) => {
                Err(ResourceFinalReleaseBindError::AlreadyBound { existing_domain })
            }
        }
    }

    pub fn owner_thread_route(&self) -> OwnerThreadFinalReleaseRoute {
        OwnerThreadFinalReleaseRoute {
            state: Arc::downgrade(&self.state),
        }
    }

    /// Atomically rejects every future external producer. Already-enqueued
    /// releases remain available to the terminal owner-thread drain, and a
    /// destructor running inside that drain may append its own nested release
    /// to the same FIFO tail.
    pub fn close(&self) -> Result<(), ResourceFinalReleaseDrainError> {
        self.reject_bound_execution_domain()?;
        self.validate_owner_thread()?;
        self.state.close();
        Ok(())
    }

    /// Close a bound route without consuming it. The same authority remains
    /// valid for the terminal drain and for reentrant releases appended by a
    /// destructor already running inside that drain.
    pub fn close_in_execution_domain(
        &self,
        execution_domain: &ResourceFinalReleaseExecutionDomain,
    ) -> Result<(), ResourceFinalReleaseDrainError> {
        self.validate_execution_domain(execution_domain)?;
        self.validate_owner_thread()?;
        self.state.close();
        Ok(())
    }

    /// Destroy every currently queued payload exactly once, in enqueue order.
    /// Destructors run outside the queue lock so reentrant releases append at
    /// the FIFO tail and are drained by the same call.
    pub fn drain(&self) -> Result<usize, ResourceFinalReleaseDrainError> {
        self.reject_bound_execution_domain()?;
        self.drain_authorized()
    }

    /// Destroy a bound route's FIFO under its backend's retained execution
    /// authority. This intentionally does not require a live ambient GL scope:
    /// lost-context shutdown must still release stale Rust ownership while the
    /// concrete destructors suppress invalid GL commands by generation stamp.
    pub fn drain_in_execution_domain(
        &self,
        execution_domain: &ResourceFinalReleaseExecutionDomain,
    ) -> Result<usize, ResourceFinalReleaseDrainError> {
        self.validate_execution_domain(execution_domain)?;
        self.drain_authorized()
    }

    fn reject_bound_execution_domain(&self) -> Result<(), ResourceFinalReleaseDrainError> {
        let expected_domain = self.state.bound_execution_domain.load(Ordering::Acquire);
        if expected_domain != 0 {
            return Err(ResourceFinalReleaseDrainError::WrongExecutionDomain {
                expected_domain,
                actual_domain: 0,
            });
        }
        Ok(())
    }

    fn validate_execution_domain(
        &self,
        execution_domain: &ResourceFinalReleaseExecutionDomain,
    ) -> Result<(), ResourceFinalReleaseDrainError> {
        let expected_domain = self.state.bound_execution_domain.load(Ordering::Acquire);
        let same_route = Weak::ptr_eq(&execution_domain.state, &Arc::downgrade(&self.state));
        if expected_domain == 0
            || execution_domain.domain != expected_domain
            || !same_route
            || execution_domain.state.strong_count() == 0
        {
            return Err(ResourceFinalReleaseDrainError::WrongExecutionDomain {
                expected_domain,
                actual_domain: execution_domain.domain,
            });
        }
        Ok(())
    }

    fn drain_authorized(&self) -> Result<usize, ResourceFinalReleaseDrainError> {
        self.validate_owner_thread()?;
        if self.state.drain_active.swap(true, Ordering::AcqRel) {
            return Ok(0);
        }
        struct DrainGuard<'a>(&'a AtomicBool);
        impl Drop for DrainGuard<'_> {
            fn drop(&mut self) {
                self.0.store(false, Ordering::Release);
            }
        }
        let _guard = DrainGuard(&self.state.drain_active);

        let mut drained = 0;
        loop {
            let owner = self
                .state
                .deferred_final_releases
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .pop_front();
            let Some(owner) = owner else {
                break;
            };
            match owner {
                DeferredFinalRelease::Resource(owner) => owner.destroy_now(),
                DeferredFinalRelease::OwnerThread(release) => unsafe { release.run() },
            }
            drained += 1;
        }
        Ok(drained)
    }

    fn validate_owner_thread(&self) -> Result<(), ResourceFinalReleaseDrainError> {
        if std::thread::current().id() != self.state.owner_thread {
            return Err(ResourceFinalReleaseDrainError::WrongThread);
        }
        Ok(())
    }
}

/// Source `RefCnt<GPUResource>::m_refcnt`. This count is deliberately not
/// an allocation-owner count: at logical zero the same concrete allocation
/// moves into manager purgatory, and a pool suspends it at logical count one.
#[repr(transparent)]
pub(crate) struct LogicalRefCount(AtomicUsize);

impl LogicalRefCount {
    pub(crate) fn new() -> Self {
        Self(AtomicUsize::new(1))
    }

    pub(crate) fn retain(&self) {
        let result = self.update(Ordering::Relaxed, Ordering::Relaxed, |count| {
            count.checked_add(1)
        });
        assert!(result.is_ok(), "GPU resource reference count overflow");
    }

    /// Returns true only for the source one-to-zero transition.
    pub(crate) fn release(&self) -> bool {
        let previous = self.update(Ordering::AcqRel, Ordering::Acquire, |count| {
            count.checked_sub(1)
        });
        match previous {
            Ok(1) => true,
            Ok(_) => false,
            Err(_) => panic!("GPU resource reference count underflow"),
        }
    }

    pub(crate) fn load(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }

    // `AtomicUsize::fetch_update` is not available under the nightly
    // build-std toolchain used by the tvOS/visionOS matrix. This is the same
    // compare-exchange transition and ordering as the source-shaped retain
    // and release above, expressed in the common stable/nightly subset.
    fn update(
        &self,
        set_order: Ordering,
        fetch_order: Ordering,
        mut update: impl FnMut(usize) -> Option<usize>,
    ) -> Result<usize, usize> {
        let mut previous = self.0.load(fetch_order);
        loop {
            let Some(next) = update(previous) else {
                return Err(previous);
            };
            match self
                .0
                .compare_exchange_weak(previous, next, set_order, fetch_order)
            {
                Ok(value) => return Ok(value),
                Err(value) => previous = value,
            }
        }
    }
}

/// Exact intrusive source base. Every ORE resource class embeds this at
/// offset zero through its authored base chain. The allocation therefore has
/// one identity from construction through retain, purgatory, pooling, and
/// final derived-to-base destruction.
#[repr(C)]
pub struct GPUResource {
    m_refcnt: LogicalRefCount,
    // Source: `const rcp<GPUResourceManager> m_manager`.
    m_manager: Option<GPUResourceManager>,
}

impl GPUResource {
    pub(crate) fn new(manager: Option<GPUResourceManager>) -> Self {
        Self {
            m_refcnt: LogicalRefCount::new(),
            m_manager: manager,
        }
    }

    pub fn manager(&self) -> Option<&GPUResourceManager> {
        self.m_manager.as_ref()
    }

    pub fn debugging_refcnt(&self) -> usize {
        self.m_refcnt.load()
    }

    pub(crate) fn retain(&self) {
        self.m_refcnt.retain();
    }

    pub(crate) fn release(&self) -> bool {
        self.m_refcnt.release()
    }

    pub(crate) fn install_manager(&mut self, manager: Option<GPUResourceManager>) {
        assert!(
            self.m_manager.is_none(),
            "GPUResource manager is installed exactly once before publication"
        );
        self.m_manager = manager;
    }
}

#[cfg(test)]
thread_local! {
    static RESOURCE_DROP_TRACE: std::cell::RefCell<Vec<&'static str>> = const {
        std::cell::RefCell::new(Vec::new())
    };
}

#[cfg(test)]
pub(crate) fn record_resource_drop_stage(stage: &'static str) {
    RESOURCE_DROP_TRACE.with(|trace| trace.borrow_mut().push(stage));
}

#[cfg(test)]
pub(crate) fn take_resource_drop_trace() -> Vec<&'static str> {
    RESOURCE_DROP_TRACE.with(|trace| core::mem::take(&mut *trace.borrow_mut()))
}

impl Drop for GPUResource {
    fn drop(&mut self) {
        #[cfg(test)]
        record_resource_drop_stage("GPUResource");
    }
}

/// Contract implemented by complete source resource objects. Implementors
/// certify that `gpu_resource()` returns the exact offset-zero inherited base;
/// construction checks the address before publishing the intrusive handle.
///
/// # Safety
/// The complete object and its `GPUResource` base must have the same address,
/// and the object must remain valid until the final intrusive release invokes
/// its registered concrete destructor.
pub unsafe trait GpuResourcePayload: Any {
    fn gpu_resource(&self) -> &GPUResource;
    fn gpu_resource_mut(&mut self) -> &mut GPUResource;

    /// Optional projection to the source's offset-zero `ore::Pipeline` base.
    ///
    /// C++ render passes accept `Pipeline*` before performing backend RTTI.
    /// The erased Rust owner therefore needs this base-kind projection in
    /// addition to exact concrete `TypeId` downcasts. Non-pipeline resources
    /// deliberately return `None`.
    fn pipeline_base(&self) -> Option<&Pipeline> {
        None
    }

    /// Optional projection to the source's offset-zero `ore::BindGroupLayout`
    /// base. Backend layouts retain this base through type erasure, so source
    /// APIs must not require an exact concrete Rust downcast to recover it.
    fn bind_group_layout_base(&self) -> Option<&BindGroupLayout> {
        None
    }
}

#[derive(Clone, Copy)]
pub(crate) struct BufferInfo {
    size: u32,
    usage: BufferUsage,
}

#[derive(Clone, Copy)]
pub(crate) struct TextureInfo {
    width: u32,
    height: u32,
    depthOrArrayLayers: u32,
    format: TextureFormat,
    r#type: TextureType,
    numMipmaps: u32,
    sampleCount: u32,
    renderTarget: bool,
}

type DestroyResource = unsafe fn(NonNull<GPUResource>);
type BufferInfoDispatch = unsafe fn(NonNull<GPUResource>) -> BufferInfo;
type BufferUpdateDispatch =
    unsafe fn(NonNull<GPUResource>, &[u8], u32, u32) -> Result<(), BufferUpdateError>;
type TextureInfoDispatch = unsafe fn(NonNull<GPUResource>) -> TextureInfo;
type TextureUploadDispatch =
    for<'a> unsafe fn(&ResourcePointer, &TextureDataDesc<'a>) -> Result<(), TextureUploadError>;
type PipelineBaseDispatch = unsafe fn(NonNull<GPUResource>) -> Option<NonNull<Pipeline>>;
type BindGroupLayoutBaseDispatch =
    unsafe fn(NonNull<GPUResource>) -> Option<NonNull<BindGroupLayout>>;

#[derive(Clone, Copy)]
pub(crate) struct ResourceVTable {
    type_id: fn() -> TypeId,
    destroy: DestroyResource,
    buffer_info: Option<BufferInfoDispatch>,
    buffer_update: Option<BufferUpdateDispatch>,
    texture_info: Option<TextureInfoDispatch>,
    texture_upload: Option<TextureUploadDispatch>,
    pipeline_base: PipelineBaseDispatch,
    bind_group_layout_base: BindGroupLayoutBaseDispatch,
}

unsafe fn destroy_resource<T: GpuResourcePayload>(base: NonNull<GPUResource>) {
    unsafe { drop(Box::from_raw(base.cast::<T>().as_ptr())) };
}

fn type_id<T: GpuResourcePayload>() -> TypeId {
    TypeId::of::<T>()
}

unsafe fn pipeline_base<T: GpuResourcePayload>(
    base: NonNull<GPUResource>,
) -> Option<NonNull<Pipeline>> {
    let payload = unsafe { base.cast::<T>().as_ref() };
    payload.pipeline_base().map(NonNull::from)
}

unsafe fn bind_group_layout_base<T: GpuResourcePayload>(
    base: NonNull<GPUResource>,
) -> Option<NonNull<BindGroupLayout>> {
    let payload = unsafe { base.cast::<T>().as_ref() };
    payload.bind_group_layout_base().map(NonNull::from)
}

unsafe fn buffer_info<T: GpuResourcePayload + BufferApi>(base: NonNull<GPUResource>) -> BufferInfo {
    let payload = unsafe { base.cast::<T>().as_ref() };
    BufferInfo {
        size: payload.size(),
        usage: payload.usage(),
    }
}

unsafe fn dispatch_buffer_update<T: GpuResourcePayload + BufferApi>(
    base: NonNull<GPUResource>,
    data: &[u8],
    size: u32,
    offset: u32,
) -> Result<(), BufferUpdateError> {
    unsafe { base.cast::<T>().as_ref() }.update(data, size, offset)
}

unsafe fn texture_info<T: GpuResourcePayload + TextureApi>(
    base: NonNull<GPUResource>,
) -> TextureInfo {
    let payload = unsafe { base.cast::<T>().as_ref() };
    TextureInfo {
        width: payload.width(),
        height: payload.height(),
        depthOrArrayLayers: payload.depthOrArrayLayers(),
        format: payload.format(),
        r#type: payload.r#type(),
        numMipmaps: payload.numMipmaps(),
        sampleCount: payload.sampleCount(),
        renderTarget: payload.isRenderTarget(),
    }
}

unsafe fn dispatch_texture_upload<T: GpuResourcePayload + TextureApi>(
    pointer: &ResourcePointer,
    data: &TextureDataDesc<'_>,
) -> Result<(), TextureUploadError> {
    pointer.base().retain();
    let owner = AnyResourceHandle {
        pointer: Some(pointer.clone()),
    };
    unsafe { pointer.base.cast::<T>().as_ref() }.uploadWithOwner(data, owner)
}

fn plain_vtable<T: GpuResourcePayload>() -> ResourceVTable {
    ResourceVTable {
        type_id: type_id::<T>,
        destroy: destroy_resource::<T>,
        buffer_info: None,
        buffer_update: None,
        texture_info: None,
        texture_upload: None,
        pipeline_base: pipeline_base::<T>,
        bind_group_layout_base: bind_group_layout_base::<T>,
    }
}

fn buffer_vtable<T: GpuResourcePayload + BufferApi>() -> ResourceVTable {
    ResourceVTable {
        buffer_info: Some(buffer_info::<T>),
        buffer_update: Some(dispatch_buffer_update::<T>),
        ..plain_vtable::<T>()
    }
}

fn texture_vtable<T: GpuResourcePayload + TextureApi>() -> ResourceVTable {
    ResourceVTable {
        texture_info: Some(texture_info::<T>),
        texture_upload: Some(dispatch_texture_upload::<T>),
        ..plain_vtable::<T>()
    }
}

/// A retaining pointer to the exact offset-zero source base. All additional
/// fields are nonowning Rust safety/dispatch sidecars.
#[derive(Clone)]
pub(crate) struct ResourcePointer {
    pub(crate) base: NonNull<GPUResource>,
    pub(crate) vtable: ResourceVTable,
    pub(crate) domain: Option<ResourceDomain>,
    pub(crate) recording_thread: std::thread::ThreadId,
}

// SAFETY: only intrusive refcounting and final destruction cross threads.
// Payload access is checked against `recording_thread`.
unsafe impl Send for ResourcePointer {}
unsafe impl Sync for ResourcePointer {}

impl ResourcePointer {
    pub(crate) fn base(&self) -> &GPUResource {
        unsafe { self.base.as_ref() }
    }

    pub(crate) fn is_recording_thread(&self) -> bool {
        self.recording_thread == std::thread::current().id()
    }
}

/// Unique deletion authority for an allocation whose intrusive owner was
/// transferred into purgatory or a recycle pool.
pub(crate) struct ResourceOwner {
    pointer: Option<ResourcePointer>,
}

unsafe impl Send for ResourceOwner {}
unsafe impl Sync for ResourceOwner {}

impl ResourceOwner {
    pub(crate) fn new(pointer: ResourcePointer) -> Self {
        Self {
            pointer: Some(pointer),
        }
    }

    pub(crate) fn pointer(&self) -> &ResourcePointer {
        self.pointer
            .as_ref()
            .expect("a live resource owner carries one allocation")
    }

    fn into_pointer(mut self) -> ResourcePointer {
        self.pointer
            .take()
            .expect("resource owner transfers one allocation")
    }

    fn destroy_now(mut self) {
        let pointer = self
            .pointer
            .take()
            .expect("resource owner destroys one allocation");
        assert!(
            pointer.is_recording_thread(),
            "deferred GPU resource destruction is confined to its recording thread"
        );
        unsafe { (pointer.vtable.destroy)(pointer.base) };
    }
}

impl Drop for ResourceOwner {
    fn drop(&mut self) {
        let Some(pointer) = self.pointer.take() else {
            return;
        };
        let is_recording_thread = pointer.is_recording_thread();
        let domain = pointer.domain.clone();
        if let Some(domain) = domain
            && domain.should_defer_final_release(is_recording_thread)
        {
            match domain.defer_final_release(ResourceOwner::new(pointer)) {
                Ok(()) => return,
                Err(owner) => {
                    // The backend violated the drain-handle lifetime contract:
                    // the only authority capable of restoring the recording
                    // execution scope is already gone. Quarantine the complete
                    // allocation instead of ever invoking a concrete GPU/API
                    // destructor on this wrong thread. Correct teardown retains
                    // the drain and therefore never enters this terminal path.
                    std::mem::forget(owner);
                    return;
                }
            }
        }
        unsafe { (pointer.vtable.destroy)(pointer.base) };
    }
}

/// Source `rcp<T>` for a known concrete complete object.
pub struct ResourceHandle<T: GpuResourcePayload> {
    pub(crate) pointer: Option<ResourcePointer>,
    pub(crate) marker: PhantomData<fn() -> T>,
}

impl<T: GpuResourcePayload> ResourceHandle<T> {
    pub fn new(manager: Option<GPUResourceManager>, payload: T) -> Self {
        Self::new_with_vtable(manager, None, payload, plain_vtable::<T>())
    }

    pub fn new_in_domain(
        manager: Option<GPUResourceManager>,
        domain: ResourceDomain,
        payload: T,
    ) -> Self {
        Self::new_with_vtable(manager, Some(domain), payload, plain_vtable::<T>())
    }

    pub fn new_buffer(manager: Option<GPUResourceManager>, payload: T) -> Self
    where
        T: BufferApi,
    {
        Self::new_with_vtable(manager, None, payload, buffer_vtable::<T>())
    }

    pub fn new_buffer_in_domain(
        manager: Option<GPUResourceManager>,
        domain: ResourceDomain,
        payload: T,
    ) -> Self
    where
        T: BufferApi,
    {
        Self::new_with_vtable(manager, Some(domain), payload, buffer_vtable::<T>())
    }

    pub fn new_texture(manager: Option<GPUResourceManager>, payload: T) -> Self
    where
        T: TextureApi,
    {
        Self::new_with_vtable(manager, None, payload, texture_vtable::<T>())
    }

    pub fn new_texture_in_domain(
        manager: Option<GPUResourceManager>,
        domain: ResourceDomain,
        payload: T,
    ) -> Self
    where
        T: TextureApi,
    {
        Self::new_with_vtable(manager, Some(domain), payload, texture_vtable::<T>())
    }

    fn new_with_vtable(
        manager: Option<GPUResourceManager>,
        domain: Option<ResourceDomain>,
        mut payload: T,
        vtable: ResourceVTable,
    ) -> Self {
        if let Some(domain) = domain.as_ref() {
            domain.assert_recording_thread();
        }
        payload.gpu_resource_mut().install_manager(manager);
        let complete = NonNull::from(Box::leak(Box::new(payload)));
        let base = NonNull::from(unsafe { complete.as_ref() }.gpu_resource());
        assert_eq!(
            complete.as_ptr().cast::<()>(),
            base.as_ptr().cast::<()>(),
            "GPUResource must be the exact offset-zero source base"
        );
        Self {
            pointer: Some(ResourcePointer {
                base,
                vtable,
                domain,
                recording_thread: std::thread::current().id(),
            }),
            marker: PhantomData,
        }
    }

    pub fn manager(&self) -> Option<&GPUResourceManager> {
        self.pointer().base().manager()
    }

    pub fn debugging_refcnt(&self) -> usize {
        self.pointer().base().debugging_refcnt()
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.pointer().base == other.pointer().base
    }

    #[cfg(test)]
    pub(crate) fn allocationAddress(&self) -> *const () {
        self.pointer().base.as_ptr().cast()
    }

    pub fn erase(mut self) -> AnyResourceHandle {
        AnyResourceHandle {
            pointer: self.pointer.take(),
        }
    }

    pub(crate) fn pointer(&self) -> &ResourcePointer {
        self.pointer
            .as_ref()
            .expect("a live resource handle always owns an allocation")
    }
}

impl<T: GpuResourcePayload> Deref for ResourceHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        assert!(
            self.pointer().is_recording_thread(),
            "GPU resource payload access is confined to its recording thread"
        );
        unsafe { self.pointer().base.cast::<T>().as_ref() }
    }
}

/// Type-erased source `rcp<GPUResource>` used for base-polymorphic APIs,
/// purgatory, and pool transfer. Checked downcasts retain the exact concrete
/// allocation and never cast an embedded base-field address.
pub struct AnyResourceHandle {
    pub(crate) pointer: Option<ResourcePointer>,
}

impl AnyResourceHandle {
    pub fn belongsTo(&self, domain: &ResourceDomain) -> bool {
        self.pointer()
            .domain
            .as_ref()
            .is_some_and(|resource_domain| resource_domain.matches(domain))
    }

    /// Checked source-base projection used before backend-specific pipeline
    /// RTTI. Payload access remains confined to the recording thread.
    pub fn pipelineBase(&self) -> Option<&Pipeline> {
        let pointer = self.pointer();
        if !pointer.is_recording_thread() {
            return None;
        }
        unsafe { (pointer.vtable.pipeline_base)(pointer.base).map(|base| base.as_ref()) }
    }

    /// Checked source-base projection for a retained backend bind-group
    /// layout. Payload access remains confined to the recording thread.
    pub fn bindGroupLayoutBase(&self) -> Option<&BindGroupLayout> {
        let pointer = self.pointer();
        if !pointer.is_recording_thread() {
            return None;
        }
        unsafe { (pointer.vtable.bind_group_layout_base)(pointer.base).map(|base| base.as_ref()) }
    }

    pub fn size(&self) -> Option<u32> {
        self.bufferInfo().map(|info| info.size)
    }

    pub fn usage(&self) -> Option<BufferUsage> {
        self.bufferInfo().map(|info| info.usage)
    }

    pub fn width(&self) -> Option<u32> {
        self.textureInfo().map(|info| info.width)
    }

    pub fn height(&self) -> Option<u32> {
        self.textureInfo().map(|info| info.height)
    }

    pub fn depthOrArrayLayers(&self) -> Option<u32> {
        self.textureInfo().map(|info| info.depthOrArrayLayers)
    }

    pub fn format(&self) -> Option<TextureFormat> {
        self.textureInfo().map(|info| info.format)
    }

    pub fn r#type(&self) -> Option<TextureType> {
        self.textureInfo().map(|info| info.r#type)
    }

    pub fn numMipmaps(&self) -> Option<u32> {
        self.textureInfo().map(|info| info.numMipmaps)
    }

    pub fn sampleCount(&self) -> Option<u32> {
        self.textureInfo().map(|info| info.sampleCount)
    }

    pub fn isRenderTarget(&self) -> Option<bool> {
        self.textureInfo().map(|info| info.renderTarget)
    }

    pub fn update(&self, data: &[u8], size: u32, offset: u32) -> Result<(), BufferUpdateError> {
        if !self.pointer().is_recording_thread() {
            return Err(BufferUpdateError::WrongExecutionDomain);
        }
        let pointer = self.pointer();
        let Some(dispatch) = pointer.vtable.buffer_update else {
            return Err(BufferUpdateError::WrongResourceKind);
        };
        unsafe { dispatch(pointer.base, data, size, offset) }
    }

    pub fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError> {
        if !self.pointer().is_recording_thread() {
            return Err(TextureUploadError::WrongExecutionDomain);
        }
        let pointer = self.pointer();
        let Some(dispatch) = pointer.vtable.texture_upload else {
            return Err(TextureUploadError::WrongResourceKind);
        };
        unsafe { dispatch(pointer, data) }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.pointer().base == other.pointer().base
    }

    #[cfg(test)]
    pub(crate) fn allocationAddress(&self) -> *const () {
        self.pointer().base.as_ptr().cast()
    }

    pub fn downcast<T: GpuResourcePayload>(mut self) -> Result<ResourceHandle<T>, Self> {
        if (self.pointer().vtable.type_id)() != TypeId::of::<T>() {
            return Err(self);
        }
        Ok(ResourceHandle {
            pointer: self.pointer.take(),
            marker: PhantomData,
        })
    }

    pub fn downcast_ref<T: GpuResourcePayload>(&self) -> Option<&T> {
        if !self.pointer().is_recording_thread()
            || (self.pointer().vtable.type_id)() != TypeId::of::<T>()
        {
            return None;
        }
        Some(unsafe { self.pointer().base.cast::<T>().as_ref() })
    }

    pub fn manager(&self) -> Option<&GPUResourceManager> {
        self.pointer().base().manager()
    }

    pub fn debugging_refcnt(&self) -> usize {
        self.pointer().base().debugging_refcnt()
    }

    pub(crate) fn from_suspended(owner: ResourceOwner) -> Self {
        debug_assert_eq!(
            owner.pointer().base().debugging_refcnt(),
            1,
            "a pooled resource must resume with exactly one logical owner"
        );
        Self {
            pointer: Some(owner.into_pointer()),
        }
    }

    pub(crate) fn take_owner(&mut self) -> ResourceOwner {
        ResourceOwner::new(
            self.pointer
                .take()
                .expect("a live resource handle always owns an allocation"),
        )
    }

    pub(crate) fn pointer(&self) -> &ResourcePointer {
        self.pointer
            .as_ref()
            .expect("a live resource handle always owns an allocation")
    }

    fn bufferInfo(&self) -> Option<BufferInfo> {
        let pointer = self.pointer();
        if !pointer.is_recording_thread() {
            return None;
        }
        pointer
            .vtable
            .buffer_info
            .map(|dispatch| unsafe { dispatch(pointer.base) })
    }

    fn textureInfo(&self) -> Option<TextureInfo> {
        let pointer = self.pointer();
        if !pointer.is_recording_thread() {
            return None;
        }
        pointer
            .vtable
            .texture_info
            .map(|dispatch| unsafe { dispatch(pointer.base) })
    }
}

/// Compatibility spelling for source declarations retaining a known concrete
/// payload. Base-polymorphic ORE owners use `AnyResourceHandle` directly.
pub type rcp<T> = Option<ResourceHandle<T>>;

#[cfg(test)]
#[repr(C)]
pub(crate) struct TestGPUResource<T: Any + Send> {
    base: GPUResource,
    pub(crate) value: T,
}

#[cfg(test)]
impl<T: Any + Send> TestGPUResource<T> {
    pub(crate) fn new(value: T) -> Self {
        Self {
            base: GPUResource::new(None),
            value,
        }
    }
}

#[cfg(test)]
unsafe impl<T: Any + Send> GpuResourcePayload for TestGPUResource<T> {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

pub(crate) struct ZombieResource {
    pub(crate) resource: ResourceOwner,
    pub(crate) lastFrameNumber: u64,
}

pub(crate) struct ManagerState {
    pub(crate) currentFrameNumber: u64,
    pub(crate) safeFrameNumber: u64,
    pub(crate) didAdvanceFrameNumber: bool,
    pub(crate) resourcePurgatory: VecDeque<ZombieResource>,
}

impl ManagerState {
    fn new() -> Self {
        Self {
            currentFrameNumber: 0,
            safeFrameNumber: 0,
            didAdvanceFrameNumber: false,
            resourcePurgatory: VecDeque::new(),
        }
    }
}

pub(crate) struct GPUResourceManagerInner {
    pub(crate) state: Mutex<ManagerState>,
}

impl GPUResourceManagerInner {
    pub(crate) fn lock_state(&self) -> MutexGuard<'_, ManagerState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Cloneable manager retain stored by each managed resource allocation.
#[derive(Clone)]
pub struct GPUResourceManager {
    pub(crate) inner: Arc<GPUResourceManagerInner>,
}

impl GPUResourceManager {
    pub(crate) fn new() -> Self {
        Self {
            inner: Arc::new(GPUResourceManagerInner {
                state: Mutex::new(ManagerState::new()),
            }),
        }
    }

    pub fn currentFrameNumber(&self) -> u64 {
        self.inner.lock_state().currentFrameNumber
    }

    pub fn safeFrameNumber(&self) -> u64 {
        self.inner.lock_state().safeFrameNumber
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }
}

/// Non-clone root for the manager ownership graph. The caller must explicitly
/// invoke `shutdown()` after all command buffers have completed.
pub struct GPUResourceManagerOwner {
    pub(crate) manager: GPUResourceManager,
}

impl GPUResourceManagerOwner {
    pub fn new() -> Self {
        Self {
            manager: GPUResourceManager::new(),
        }
    }

    pub fn manager(&self) -> GPUResourceManager {
        self.manager.clone()
    }

    pub fn shutdown(&self) {
        self.manager.shutdown();
    }
}

impl Default for GPUResourceManagerOwner {
    fn default() -> Self {
        Self::new()
    }
}

/// Manual FIFO recycling pool. Entries suspend a sole logical owner at count
/// one without changing the concrete allocation identity.
#[repr(C)]
pub struct GPUResourcePoolMembers {
    pub(crate) m_maxPoolCount: usize,
    pub(crate) m_pool: ManuallyDrop<Mutex<VecDeque<ZombieResource>>>,
}

#[repr(C)]
pub struct GPUResourcePool {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) members: ManuallyDrop<GPUResourcePoolMembers>,
}

impl Deref for GPUResourcePool {
    type Target = GPUResourcePoolMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for GPUResourcePool {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for GPUResourcePool {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_pool);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl GPUResourcePool {
    fn newPayload(_manager: &GPUResourceManager, maxPoolSize: usize) -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            members: ManuallyDrop::new(GPUResourcePoolMembers {
                m_maxPoolCount: maxPoolSize,
                m_pool: ManuallyDrop::new(Mutex::new(VecDeque::new())),
            }),
        }
    }

    pub(crate) fn inheritedManager(&self) -> GPUResourceManager {
        self.base
            .manager()
            .expect("a live GPUResourcePool retains its inherited manager")
            .clone()
    }

    /// Source public `GPUResourcePool(rcp<GPUResourceManager>, size_t)`.
    /// Returning the intrusive handle preserves the inherited GPUResource's
    /// single strong manager retain; the payload stores only its weak
    /// backpointer and therefore cannot introduce a second owner.
    pub fn new(manager: GPUResourceManager, maxPoolSize: usize) -> ResourceHandle<Self> {
        let pool = Self::newPayload(&manager, maxPoolSize);
        ResourceHandle::new(Some(manager), pool)
    }
}

unsafe impl GpuResourcePayload for GPUResourcePool {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl fmt::Debug for GPUResourceManager {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GPUResourceManager")
            .field("currentFrameNumber", &self.currentFrameNumber())
            .field("safeFrameNumber", &self.safeFrameNumber())
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Barrier, Weak};
    use std::thread;

    macro_rules! impl_test_gpu_resource {
        ($type:ty) => {
            unsafe impl GpuResourcePayload for $type {
                fn gpu_resource(&self) -> &GPUResource {
                    &self.base
                }

                fn gpu_resource_mut(&mut self) -> &mut GPUResource {
                    &mut self.base
                }
            }
        };
    }

    #[repr(C)]
    struct DropProbe {
        base: GPUResource,
        id: usize,
        drops: Arc<Mutex<Vec<usize>>>,
    }

    impl DropProbe {
        fn new(id: usize, drops: &Arc<Mutex<Vec<usize>>>) -> Self {
            Self {
                base: GPUResource::new(None),
                id,
                drops: Arc::clone(drops),
            }
        }
    }
    impl_test_gpu_resource!(DropProbe);

    impl Drop for DropProbe {
        fn drop(&mut self) {
            self.drops
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push(self.id);
        }
    }

    fn dropped(drops: &Arc<Mutex<Vec<usize>>>) -> Vec<usize> {
        drops
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    #[test]
    fn ore_classes_preserve_offset_zero_base_chains_and_authored_member_order() {
        use core::mem::offset_of;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_hpp::{BindGroup, BindGroupMembers};
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_layout_hpp::{BindGroupLayout, BindGroupLayoutMembers};
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_buffer_hpp::{Buffer, BufferMembers};
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_context_metal_hpp::{ContextMetal, ContextMetalMembers};
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_pipeline_hpp::{Pipeline, PipelineMembers};
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_render_pass_hpp::{RenderPass, RenderPassMembers};
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_sampler_hpp::Sampler;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_shader_module_hpp::{ShaderModule, ShaderModuleMembers};
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_texture_hpp::{Texture, TextureMembers, TextureView, TextureViewMembers};
        use crate::mechanical_port::source::renderer::src::ore::metal::ore_bind_group_metal_hpp::BindGroupMetal;
        use crate::mechanical_port::source::renderer::src::ore::metal::ore_buffer_metal_hpp::BufferMetal;
        use crate::mechanical_port::source::renderer::src::ore::metal::ore_pipeline_metal_hpp::PipelineMetal;
        use crate::mechanical_port::source::renderer::src::ore::metal::ore_sampler_metal_hpp::SamplerMetal;
        use crate::mechanical_port::source::renderer::src::ore::metal::ore_shader_module_metal_hpp::ShaderModuleMetal;
        use crate::mechanical_port::source::renderer::src::ore::metal::ore_texture_metal_hpp::{TextureMetal, TextureViewMetal};

        fn ascending(offsets: &[usize]) {
            assert!(
                offsets.windows(2).all(|pair| pair[0] < pair[1]),
                "{offsets:?}"
            );
        }

        assert_eq!(offset_of!(BindGroupLayout, base), 0);
        ascending(&[
            offset_of!(BindGroupLayoutMembers, m_groupIndex),
            offset_of!(BindGroupLayoutMembers, m_entries),
            offset_of!(BindGroupLayoutMembers, m_context),
        ]);
        assert_eq!(offset_of!(Buffer, base), 0);
        ascending(&[
            offset_of!(BufferMembers, m_size),
            offset_of!(BufferMembers, m_usage),
        ]);
        assert_eq!(offset_of!(Texture, base), 0);
        ascending(&[
            offset_of!(TextureMembers, m_width),
            offset_of!(TextureMembers, m_height),
            offset_of!(TextureMembers, m_depthOrArrayLayers),
            offset_of!(TextureMembers, m_format),
            offset_of!(TextureMembers, m_type),
            offset_of!(TextureMembers, m_renderTarget),
            offset_of!(TextureMembers, m_numMipmaps),
            offset_of!(TextureMembers, m_sampleCount),
        ]);
        assert_eq!(offset_of!(TextureView, base), 0);
        ascending(&[
            offset_of!(TextureViewMembers, m_texture),
            offset_of!(TextureViewMembers, m_dimension),
            offset_of!(TextureViewMembers, m_aspect),
            offset_of!(TextureViewMembers, m_baseMipLevel),
            offset_of!(TextureViewMembers, m_mipCount),
            offset_of!(TextureViewMembers, m_baseLayer),
            offset_of!(TextureViewMembers, m_layerCount),
        ]);
        assert_eq!(offset_of!(Sampler, base), 0);
        assert_eq!(offset_of!(ShaderModule, base), 0);
        let mut shader_offsets = vec![
            offset_of!(ShaderModuleMembers, m_textureSamplerPairs),
            offset_of!(ShaderModuleMembers, m_bindingMap),
        ];
        #[cfg(feature = "track-rive-shader-id")]
        shader_offsets.push(offset_of!(ShaderModuleMembers, m_shaderAssetId));
        shader_offsets.push(offset_of!(ShaderModuleMembers, m_glFixup));
        ascending(&shader_offsets);
        assert_eq!(offset_of!(Pipeline, base), 0);
        ascending(&[
            offset_of!(PipelineMembers, m_bindingMap),
            offset_of!(PipelineMembers, m_layouts),
            offset_of!(PipelineMembers, m_desc),
        ]);
        assert_eq!(offset_of!(BindGroup, base), 0);
        ascending(&[
            offset_of!(BindGroupMembers, m_dynamicOffsetCount),
            offset_of!(BindGroupMembers, m_layoutRef),
            offset_of!(BindGroupMembers, m_retainedBuffers),
            offset_of!(BindGroupMembers, m_retainedViews),
            offset_of!(BindGroupMembers, m_retainedSamplers),
            offset_of!(BindGroupMembers, m_context),
        ]);
        assert_eq!(offset_of!(ContextMetal, base), 0);
        ascending(&[
            offset_of!(ContextMetalMembers, m_mtlDevice),
            offset_of!(ContextMetalMembers, m_mtlQueue),
            offset_of!(ContextMetalMembers, m_mtlCommandBuffer),
            offset_of!(ContextMetalMembers, m_deferredBindGroups),
            offset_of!(ContextMetalMembers, m_currentSerial),
            offset_of!(ContextMetalMembers, m_completedSerial),
        ]);
        assert_eq!(offset_of!(RenderPass, members), 0);
        ascending(&[
            offset_of!(RenderPassMembers, m_finished),
            offset_of!(RenderPassMembers, m_colorFormats),
            offset_of!(RenderPassMembers, m_colorCount),
            offset_of!(RenderPassMembers, m_depthFormat),
            offset_of!(RenderPassMembers, m_hasDepth),
            offset_of!(RenderPassMembers, m_sampleCount),
            offset_of!(RenderPassMembers, m_context),
            offset_of!(RenderPassMembers, m_boundGroups),
        ]);

        assert_eq!(offset_of!(BufferMetal, base), 0);
        assert_eq!(offset_of!(TextureMetal, base), 0);
        assert_eq!(offset_of!(TextureViewMetal, base), 0);
        assert_eq!(offset_of!(SamplerMetal, base), 0);
        assert_eq!(offset_of!(ShaderModuleMetal, base), 0);
        assert_eq!(offset_of!(PipelineMetal, base), 0);
        assert_eq!(offset_of!(BindGroupMetal, base), 0);
    }

    #[test]
    fn intrusive_identity_survives_retain_erase_purgatory_pool_and_checked_rtti() {
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_layout_hpp::BindGroupLayout;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_sampler_hpp::Sampler;

        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(1, 1);
        let pool = GPUResourcePool::new(manager.clone(), 4);
        let resource = ResourceHandle::new(Some(manager), BindGroupLayout::new());
        let allocation = resource.allocationAddress();
        assert_eq!(
            (&*resource as *const BindGroupLayout).cast::<()>(),
            allocation
        );

        let retained = resource.clone();
        assert_eq!(retained.allocationAddress(), allocation);
        assert_eq!(retained.debugging_refcnt(), 2);
        drop(retained);

        let erased = resource.erase();
        assert_eq!(erased.allocationAddress(), allocation);
        assert!(erased.downcast_ref::<BindGroupLayout>().is_some());
        assert!(erased.downcast_ref::<Sampler>().is_none());
        pool.recycle(Some(erased));

        let recycled = pool.acquire().expect("safe pooled resource");
        assert_eq!(recycled.allocationAddress(), allocation);
        let typed = recycled
            .downcast::<BindGroupLayout>()
            .expect("exact source RTTI identity");
        assert_eq!(typed.allocationAddress(), allocation);
        drop(typed);
        owner.shutdown();
    }

    #[test]
    fn ore_classes_execute_exact_reverse_member_then_base_destruction() {
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_hpp::BindGroup;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_bind_group_layout_hpp::BindGroupLayout;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_buffer_hpp::Buffer;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_context_metal_hpp::ContextMetal;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_pipeline_hpp::Pipeline;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_render_pass_hpp::RenderPass;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_sampler_hpp::Sampler;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_shader_module_hpp::ShaderModule;
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_texture_hpp::{Texture, TextureView};
        use crate::mechanical_port::source::renderer::include::rive::renderer::ore::ore_types_hpp::{BufferUsage, PipelineDesc, TextureDesc, TextureViewDesc};

        fn assert_drop(value: impl Sized, expected: &[&'static str]) {
            let _ = take_resource_drop_trace();
            drop(value);
            assert_eq!(take_resource_drop_trace(), expected);
        }

        assert_drop(
            BindGroupLayout::new(),
            &[
                "BindGroupLayout.context",
                "BindGroupLayout.entries",
                "BindGroupLayout.base",
                "GPUResource",
            ],
        );
        assert_drop(
            Buffer::new(8, BufferUsage::uniform),
            &["Buffer.base", "GPUResource"],
        );
        assert_drop(
            Texture::new(&TextureDesc::default()),
            &["Texture.base", "GPUResource"],
        );
        assert_drop(Sampler::new(), &["Sampler.base", "GPUResource"]);
        assert_drop(
            ShaderModule::new(),
            &[
                "ShaderModule.glFixup",
                "ShaderModule.bindingMap",
                "ShaderModule.textureSamplerPairs",
                "ShaderModule.base",
                "GPUResource",
            ],
        );
        assert_drop(
            Pipeline::new(&PipelineDesc::default()).expect("default pipeline snapshot"),
            &[
                "Pipeline.desc",
                "Pipeline.layouts",
                "Pipeline.bindingMap",
                "Pipeline.base",
                "GPUResource",
            ],
        );
        assert_drop(
            BindGroup::new(),
            &[
                "BindGroup.context",
                "BindGroup.samplers",
                "BindGroup.views",
                "BindGroup.buffers",
                "BindGroup.layout",
                "BindGroup.base",
                "GPUResource",
            ],
        );

        let texture = ResourceHandle::new(None, Texture::new(&TextureDesc::default())).erase();
        let view = TextureView::new(texture, &TextureViewDesc::default());
        assert_drop(
            view,
            &[
                "TextureView.texture",
                "Texture.base",
                "GPUResource",
                "TextureView.base",
                "GPUResource",
            ],
        );

        assert_drop(
            ContextMetal::new(),
            &[
                "ContextMetal.commandBuffer",
                "ContextMetal.queue",
                "ContextMetal.device",
                "ContextMetal.bufferStateAdapter",
                "ContextMetal.completedSerial",
                "ContextMetal.currentSerial",
                "ContextMetal.deferredBindGroups",
                "ContextMetal.base",
            ],
        );
        assert_drop(
            RenderPass::new(Weak::new()),
            &["RenderPass.boundGroups", "RenderPass.context"],
        );
    }

    fn erased<T: GpuResourcePayload>(resource: ResourceHandle<T>) -> AnyResourceHandle {
        resource.erase()
    }

    #[test]
    fn unmanaged_resource_deletes_immediately_at_zero() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let resource = ResourceHandle::new(None, DropProbe::new(1, &drops));
        let clone = resource.clone();
        assert_eq!(resource.debugging_refcnt(), 2);
        drop(clone);
        assert!(dropped(&drops).is_empty());
        drop(resource);
        assert_eq!(dropped(&drops), [1]);
    }

    #[test]
    fn pre_first_frame_release_waits_even_when_frame_zero_is_caught_up() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(1, &drops),
        ));
        assert!(dropped(&drops).is_empty());

        manager.advanceFrameNumber(0, 0);
        assert_eq!(dropped(&drops), [1]);
        owner.shutdown();
    }

    #[test]
    fn caught_up_manager_releases_immediately() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(3, 3);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(1, &drops),
        ));
        assert_eq!(dropped(&drops), [1]);
        owner.shutdown();
    }

    #[test]
    fn in_flight_resources_wait_for_their_release_frame_in_fifo_order() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(2, 0);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(1, &drops),
        ));
        manager.advanceFrameNumber(3, 0);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(2, &drops),
        ));

        manager.advanceFrameNumber(4, 1);
        assert!(dropped(&drops).is_empty());
        manager.advanceFrameNumber(4, 2);
        assert_eq!(dropped(&drops), [1]);
        manager.advanceFrameNumber(4, 3);
        assert_eq!(dropped(&drops), [1, 2]);
        owner.shutdown();
    }

    #[test]
    fn pool_recycles_oldest_safe_resource_without_changing_ref_count() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(2, 0);
        let resource = ResourceHandle::new(Some(manager.clone()), DropProbe::new(7, &drops));
        let pool = GPUResourcePool::new(manager.clone(), 4);
        pool.recycle(Some(resource.erase()));
        assert!(pool.acquire().is_none());

        manager.advanceFrameNumber(2, 2);
        let resource = pool.acquire().expect("safe pooled resource");
        assert_eq!(resource.debugging_refcnt(), 1);
        assert_eq!(
            resource.downcast_ref::<DropProbe>().map(|probe| probe.id),
            Some(7)
        );
        drop(resource);
        assert_eq!(dropped(&drops), [7]);
        drop(pool);
        owner.shutdown();
    }

    #[test]
    fn pool_trims_only_safe_excess_after_successful_acquire() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(1, 0);
        let pool = GPUResourcePool::new(manager.clone(), 2);
        for id in 0..4 {
            pool.recycle(Some(erased(ResourceHandle::new(
                Some(manager.clone()),
                DropProbe::new(id, &drops),
            ))));
        }
        assert_eq!(pool.m_pool.lock().unwrap().len(), 4);

        manager.advanceFrameNumber(1, 1);
        let acquired = pool.acquire().expect("oldest safe resource");
        assert_eq!(
            acquired.downcast_ref::<DropProbe>().map(|probe| probe.id),
            Some(0)
        );
        assert_eq!(pool.m_pool.lock().unwrap().len(), 2);
        assert_eq!(dropped(&drops), [1]);

        drop(pool);
        assert_eq!(dropped(&drops), [1, 2, 3]);
        drop(acquired);
        assert_eq!(dropped(&drops), [1, 2, 3, 0]);
        owner.shutdown();
    }

    #[test]
    fn pool_is_type_erased_but_downcast_is_checked_and_destructors_are_concrete() {
        #[repr(C)]
        struct OtherProbe {
            base: GPUResource,
            drops: Arc<AtomicUsize>,
        }
        impl_test_gpu_resource!(OtherProbe);
        impl Drop for OtherProbe {
            fn drop(&mut self) {
                self.drops.fetch_add(1, Ordering::Relaxed);
            }
        }

        let drops = Arc::new(Mutex::new(Vec::new()));
        let other_drops = Arc::new(AtomicUsize::new(0));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(1, 1);
        let pool = GPUResourcePool::new(manager.clone(), 4);
        pool.recycle(Some(erased(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(4, &drops),
        ))));
        pool.recycle(Some(erased(ResourceHandle::new(
            Some(manager.clone()),
            OtherProbe {
                base: GPUResource::new(None),
                drops: Arc::clone(&other_drops),
            },
        ))));

        let first = pool.acquire().expect("first resource");
        let first = first
            .downcast::<DropProbe>()
            .expect("checked concrete type");
        assert_eq!(first.id, 4);
        let second = pool.acquire().expect("second resource");
        assert!(second.downcast_ref::<DropProbe>().is_none());
        let second = second
            .downcast::<OtherProbe>()
            .expect("checked concrete type");
        drop(first);
        drop(second);
        assert_eq!(dropped(&drops), [4]);
        assert_eq!(other_drops.load(Ordering::Relaxed), 1);
        drop(pool);
        owner.shutdown();
    }

    #[test]
    fn managed_pool_itself_waits_until_its_safe_frame_before_dropping_entries() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(2, 0);
        let pool = GPUResourcePool::new(manager.clone(), 4);
        pool.recycle(Some(erased(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(9, &drops),
        ))));

        drop(pool);
        assert!(dropped(&drops).is_empty());
        manager.advanceFrameNumber(2, 2);
        assert_eq!(dropped(&drops), [9]);
        owner.shutdown();
    }

    #[test]
    fn shutdown_purges_zombies_and_makes_future_releases_immediate() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(5, 2);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(1, &drops),
        ));
        assert!(dropped(&drops).is_empty());

        owner.shutdown();
        owner.shutdown();
        assert_eq!(dropped(&drops), [1]);
        assert_eq!(manager.currentFrameNumber(), u64::MAX);
        assert_eq!(manager.safeFrameNumber(), u64::MAX);

        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(2, &drops),
        ));
        assert_eq!(dropped(&drops), [1, 2]);
    }

    #[test]
    fn reference_count_is_thread_safe_and_releases_once() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        let resource = ResourceHandle::new(Some(manager.clone()), DropProbe::new(1, &drops));
        let mut threads = Vec::new();
        for _ in 0..8 {
            let clone = resource.clone();
            threads.push(thread::spawn(move || drop(clone)));
        }
        for thread in threads {
            assert!(thread.join().is_ok());
        }
        assert_eq!(resource.debugging_refcnt(), 1);
        drop(resource);
        assert!(dropped(&drops).is_empty());
        owner.shutdown();
        assert_eq!(dropped(&drops), [1]);
    }

    #[test]
    fn final_release_on_worker_thread_enters_purgatory_once() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(3, 1);
        let resource = ResourceHandle::new(Some(manager), DropProbe::new(5, &drops));

        assert!(thread::spawn(move || drop(resource)).join().is_ok());
        assert!(dropped(&drops).is_empty());
        owner.shutdown();
        assert_eq!(dropped(&drops), [5]);
    }

    #[test]
    fn domain_defers_worker_final_release_until_owner_drain() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let drain = ResourceFinalReleaseDrain::new();
        let identity = Arc::new(());
        let resource = ResourceHandle::new_in_domain(
            None,
            drain.resource_domain(&identity),
            DropProbe::new(11, &drops),
        );

        thread::spawn(move || drop(resource))
            .join()
            .expect("worker final release");
        assert!(dropped(&drops).is_empty());
        assert_eq!(drain.drain(), Ok(1));
        assert_eq!(dropped(&drops), [11]);
    }

    #[test]
    fn owner_drain_is_exactly_once_fifo_and_rejects_wrong_thread() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let drain = ResourceFinalReleaseDrain::new();
        let identity = Arc::new(());
        let domain = drain.resource_domain(&identity);
        let resources = (1..=3)
            .map(|id| {
                ResourceHandle::new_in_domain(None, domain.clone(), DropProbe::new(id, &drops))
            })
            .collect::<Vec<_>>();

        thread::spawn(move || {
            for resource in resources {
                drop(resource);
            }
        })
        .join()
        .expect("ordered worker final releases");

        let worker_drain = drain.clone();
        assert_eq!(
            thread::spawn(move || worker_drain.drain())
                .join()
                .expect("wrong-thread drain result"),
            Err(ResourceFinalReleaseDrainError::WrongThread)
        );
        assert!(dropped(&drops).is_empty());

        assert_eq!(drain.drain(), Ok(3));
        assert_eq!(dropped(&drops), [1, 2, 3]);
        assert_eq!(drain.drain(), Ok(0));
        assert_eq!(dropped(&drops), [1, 2, 3]);
    }

    #[test]
    fn drain_handle_outlives_context_state_without_orphaning_manager_release() {
        use crate::context::ContextState;

        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(1, 1);
        let context = ContextState::new(crate::types::Features::default(), Some(manager.clone()));
        let drain = context.resourceFinalReleaseDrain();
        let domain = context.resourceDomain();
        let resource = ResourceHandle::new_in_domain(
            Some(manager),
            domain.clone(),
            DropProbe::new(17, &drops),
        )
        .erase();
        assert!(resource.belongsTo(&domain));
        drop(context);
        assert!(
            !resource.belongsTo(&domain),
            "source Context identity expires independently of its retained drain route"
        );

        thread::spawn(move || drop(resource))
            .join()
            .expect("manager release after ORE Context drop");
        assert!(dropped(&drops).is_empty());
        assert_eq!(drain.drain(), Ok(1));
        assert_eq!(dropped(&drops), [17]);
        owner.shutdown();
    }

    #[test]
    fn expired_context_and_drain_never_destroy_payload_on_worker() {
        use crate::context::ContextState;

        let drops = Arc::new(Mutex::new(Vec::new()));
        let context = ContextState::new(crate::types::Features::default(), None);
        let resource = ResourceHandle::new_in_domain(
            None,
            context.resourceDomain(),
            DropProbe::new(18, &drops),
        );
        drop(context);

        thread::spawn(move || drop(resource))
            .join()
            .expect("expired-domain worker final release");
        assert!(
            dropped(&drops).is_empty(),
            "an expired drain route must quarantine rather than destroy on a worker"
        );
    }

    #[test]
    fn expired_always_defer_route_never_destroys_on_unscoped_owner() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let identity = Arc::new(());
        let drain = ResourceFinalReleaseDrain::new();
        drain.set_always_defer();
        let resource = ResourceHandle::new_in_domain(
            None,
            drain.resource_domain(&identity),
            DropProbe::new(181, &drops),
        );
        drop(drain);

        drop(resource);
        assert!(
            dropped(&drops).is_empty(),
            "always-deferred payloads require their backend execution scope even on the owner thread"
        );
    }

    #[test]
    fn worker_pool_owner_destruction_routes_nested_payloads_to_domain_drain() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(1, 1);
        let drain = ResourceFinalReleaseDrain::new();
        let identity = Arc::new(());
        let domain = drain.resource_domain(&identity);
        let pool = ResourceHandle::new_in_domain(
            Some(manager.clone()),
            domain.clone(),
            GPUResourcePool::newPayload(&manager, 1),
        );
        pool.recycle(Some(
            ResourceHandle::new_in_domain(Some(manager), domain, DropProbe::new(19, &drops))
                .erase(),
        ));

        thread::spawn(move || drop(pool))
            .join()
            .expect("worker pool final release");
        assert!(dropped(&drops).is_empty());
        assert_eq!(drain.drain(), Ok(1));
        assert_eq!(dropped(&drops), [19]);
        owner.shutdown();
    }

    #[test]
    fn always_defer_routes_owner_release_through_execution_scope_drain() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let drain = ResourceFinalReleaseDrain::new();
        let identity = Arc::new(());
        drain.set_always_defer();
        let resource = ResourceHandle::new_in_domain(
            None,
            drain.resource_domain(&identity),
            DropProbe::new(23, &drops),
        );

        drop(resource);
        assert!(dropped(&drops).is_empty());
        assert_eq!(drain.drain(), Ok(1));
        assert_eq!(dropped(&drops), [23]);
    }

    #[test]
    fn owner_thread_final_release_route_defers_opaque_destructor() {
        unsafe fn destroy_probe(payload: usize) {
            unsafe { drop(Box::from_raw(payload as *mut DropProbe)) };
        }

        let drops = Arc::new(Mutex::new(Vec::new()));
        let drain = ResourceFinalReleaseDrain::new();
        let route = drain.owner_thread_route();
        let pointer = Box::into_raw(Box::new(DropProbe::new(29, &drops)));
        let release = unsafe { OwnerThreadFinalRelease::new(pointer as usize, destroy_probe) };

        assert!(
            thread::spawn(move || route.defer(release))
                .join()
                .expect("worker final-release enqueue")
                .is_ok()
        );
        assert!(dropped(&drops).is_empty());
        assert_eq!(drain.drain(), Ok(1));
        assert_eq!(dropped(&drops), [29]);
    }

    #[test]
    fn release_or_defer_preserves_owner_raii_and_routes_worker_release() {
        unsafe fn destroy_probe(payload: usize) {
            unsafe { drop(Box::from_raw(payload as *mut DropProbe)) };
        }

        let drops = Arc::new(Mutex::new(Vec::new()));
        let drain = ResourceFinalReleaseDrain::new();
        let route = drain.owner_thread_route();
        let ownerPointer = Box::into_raw(Box::new(DropProbe::new(30, &drops)));
        let ownerRelease =
            unsafe { OwnerThreadFinalRelease::new(ownerPointer as usize, destroy_probe) };
        assert!(route.release_or_defer(ownerRelease).is_ok());
        assert_eq!(dropped(&drops), [30]);

        let workerPointer = Box::into_raw(Box::new(DropProbe::new(301, &drops)));
        let workerRelease =
            unsafe { OwnerThreadFinalRelease::new(workerPointer as usize, destroy_probe) };
        assert!(
            thread::spawn(move || route.release_or_defer(workerRelease))
                .join()
                .expect("worker final-release enqueue")
                .is_ok()
        );
        assert_eq!(dropped(&drops), [30]);
        assert_eq!(drain.drain(), Ok(1));
        assert_eq!(dropped(&drops), [30, 301]);
    }

    #[test]
    fn closed_owner_thread_route_rejects_future_producers() {
        unsafe fn destroy_probe(payload: usize) {
            unsafe { drop(Box::from_raw(payload as *mut DropProbe)) };
        }

        let drops = Arc::new(Mutex::new(Vec::new()));
        let drain = ResourceFinalReleaseDrain::new();
        let route = drain.owner_thread_route();
        assert_eq!(drain.close(), Ok(()));
        let pointer = Box::into_raw(Box::new(DropProbe::new(31, &drops)));
        let release = unsafe { OwnerThreadFinalRelease::new(pointer as usize, destroy_probe) };
        let rejected = route
            .defer(release)
            .expect_err("closed route must reject the producer");
        assert_eq!(drain.drain(), Ok(0));
        assert!(dropped(&drops).is_empty());

        // The production caller quarantines this allocation. The test owns
        // the callback and runs it directly on the recorded owner only to
        // reclaim its probe allocation.
        unsafe { rejected.run() };
        assert_eq!(dropped(&drops), [31]);
    }

    #[test]
    fn closed_terminal_drain_accepts_owner_thread_reentrant_release() {
        struct ReentrantProbe {
            route: OwnerThreadFinalReleaseRoute,
            drops: Arc<Mutex<Vec<usize>>>,
        }

        unsafe fn destroy_leaf(payload: usize) {
            unsafe { drop(Box::from_raw(payload as *mut DropProbe)) };
        }

        unsafe fn destroy_parent(payload: usize) {
            let parent = unsafe { Box::from_raw(payload as *mut ReentrantProbe) };
            parent.drops.lock().unwrap().push(41);
            let leaf = Box::into_raw(Box::new(DropProbe::new(42, &parent.drops)));
            let release = unsafe { OwnerThreadFinalRelease::new(leaf as usize, destroy_leaf) };
            assert!(
                parent.route.defer(release).is_ok(),
                "a terminal owner-thread destructor may append its nested release"
            );
        }

        let drops = Arc::new(Mutex::new(Vec::new()));
        let drain = ResourceFinalReleaseDrain::new();
        let route = drain.owner_thread_route();
        let parent = Box::into_raw(Box::new(ReentrantProbe {
            route: route.clone(),
            drops: Arc::clone(&drops),
        }));
        assert!(
            route
                .defer(unsafe { OwnerThreadFinalRelease::new(parent as usize, destroy_parent) })
                .is_ok()
        );

        assert_eq!(drain.close(), Ok(()));
        assert_eq!(drain.drain(), Ok(2));
        assert_eq!(dropped(&drops), [41, 42]);
        assert_eq!(drain.drain(), Ok(0));
    }

    #[test]
    fn final_release_drain_binds_to_exactly_one_execution_domain() {
        let drain = ResourceFinalReleaseDrain::new();
        let executionDomain = drain
            .bind_execution_domain(41)
            .expect("first execution-domain bind succeeds");
        assert!(matches!(
            drain.bind_execution_domain(43),
            Err(ResourceFinalReleaseBindError::AlreadyBound {
                existing_domain: 41,
            })
        ));
        assert_eq!(
            drain.close(),
            Err(ResourceFinalReleaseDrainError::WrongExecutionDomain {
                expected_domain: 41,
                actual_domain: 0,
            })
        );
        assert_eq!(
            drain.drain(),
            Err(ResourceFinalReleaseDrainError::WrongExecutionDomain {
                expected_domain: 41,
                actual_domain: 0,
            })
        );
        assert_eq!(drain.close_in_execution_domain(&executionDomain), Ok(()));
        assert_eq!(drain.drain_in_execution_domain(&executionDomain), Ok(0));
    }

    #[test]
    fn final_release_close_rejects_wrong_thread_for_raw_and_bound_routes() {
        let rawDrain = ResourceFinalReleaseDrain::new();
        let rawWorkerDrain = rawDrain.clone();
        assert_eq!(
            thread::spawn(move || rawWorkerDrain.close())
                .join()
                .expect("wrong-thread raw close result"),
            Err(ResourceFinalReleaseDrainError::WrongThread)
        );
        assert_eq!(rawDrain.close(), Ok(()));

        let boundDrain = ResourceFinalReleaseDrain::new();
        let executionDomain = boundDrain
            .bind_execution_domain(47)
            .expect("execution-domain bind succeeds");
        let workerDrain = boundDrain.clone();
        let (executionDomain, closeResult) = thread::spawn(move || {
            let result = workerDrain.close_in_execution_domain(&executionDomain);
            (executionDomain, result)
        })
        .join()
        .expect("wrong-thread bound close result");
        assert_eq!(
            closeResult,
            Err(ResourceFinalReleaseDrainError::WrongThread)
        );
        assert_eq!(
            boundDrain.close_in_execution_domain(&executionDomain),
            Ok(())
        );
    }

    #[test]
    fn domain_resource_construction_rejects_foreign_thread() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let drain = ResourceFinalReleaseDrain::new();
        let identity = Arc::new(());
        let domain = drain.resource_domain(&identity);
        let worker_drops = Arc::clone(&drops);
        assert!(
            thread::spawn(move || {
                ResourceHandle::new_in_domain(None, domain, DropProbe::new(37, &worker_drops))
            })
            .join()
            .is_err()
        );
        assert_eq!(dropped(&drops), [37]);
    }

    #[test]
    fn erased_metadata_dispatch_never_borrows_payload_off_recording_thread() {
        #[repr(C)]
        struct ThreadBoundBuffer {
            base: GPUResource,
        }
        impl_test_gpu_resource!(ThreadBoundBuffer);

        impl BufferApi for ThreadBoundBuffer {
            fn size(&self) -> u32 {
                64
            }

            fn usage(&self) -> BufferUsage {
                BufferUsage::uniform
            }

            fn update(
                &self,
                _data: &[u8],
                _size: u32,
                _offset: u32,
            ) -> Result<(), BufferUpdateError> {
                Ok(())
            }
        }

        let resource = ResourceHandle::new_buffer(
            None,
            ThreadBoundBuffer {
                base: GPUResource::new(None),
            },
        )
        .erase();
        let result = thread::spawn(move || {
            (
                resource.size(),
                resource.usage(),
                resource.update(&[], 0, 0),
            )
        })
        .join()
        .expect("worker must not dereference the recording-thread payload");

        assert_eq!(result.0, None);
        assert_eq!(result.1, None);
        assert_eq!(result.2, Err(BufferUpdateError::WrongExecutionDomain));
    }

    #[test]
    fn manager_purge_drops_reentrant_payload_outside_state_lock() {
        #[repr(C)]
        struct ReentrantProbe {
            base: GPUResource,
            manager: GPUResourceManager,
            drops: Arc<AtomicUsize>,
        }
        impl_test_gpu_resource!(ReentrantProbe);

        impl Drop for ReentrantProbe {
            fn drop(&mut self) {
                let _ = self.manager.currentFrameNumber();
                let _ = self.manager.safeFrameNumber();
                self.drops.fetch_add(1, Ordering::Relaxed);
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(2, 0);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            ReentrantProbe {
                base: GPUResource::new(None),
                manager,
                drops: Arc::clone(&drops),
            },
        ));

        owner.shutdown();
        assert_eq!(drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn manager_reclaims_one_inline_entry_before_reentrant_safe_frame_advance() {
        #[repr(C)]
        struct ReentrantAdvanceProbe {
            base: GPUResource,
            id: usize,
            manager: GPUResourceManager,
            reentrant_safe_frame: Option<u64>,
            order: Arc<Mutex<Vec<usize>>>,
        }
        impl_test_gpu_resource!(ReentrantAdvanceProbe);

        impl Drop for ReentrantAdvanceProbe {
            fn drop(&mut self) {
                self.order
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(self.id);
                if let Some(safe_frame) = self.reentrant_safe_frame {
                    self.manager.advanceFrameNumber(3, safe_frame);
                }
            }
        }

        let order = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        for (frame, id, reentrant_safe_frame) in [(1, 1, Some(3)), (2, 2, None), (3, 3, None)] {
            manager.advanceFrameNumber(frame, 0);
            drop(ResourceHandle::new(
                Some(manager.clone()),
                ReentrantAdvanceProbe {
                    base: GPUResource::new(None),
                    id,
                    manager: manager.clone(),
                    reentrant_safe_frame,
                    order: Arc::clone(&order),
                },
            ));
        }

        manager.advanceFrameNumber(3, 2);
        assert_eq!(dropped(&order), [1, 2, 3]);
        owner.shutdown();
    }

    #[test]
    fn pool_trim_and_release_drop_reentrant_payloads_outside_both_locks() {
        #[repr(C)]
        struct ReentrantProbe {
            base: GPUResource,
            id: usize,
            manager: GPUResourceManager,
            drops: Arc<Mutex<Vec<usize>>>,
        }
        impl_test_gpu_resource!(ReentrantProbe);

        impl Drop for ReentrantProbe {
            fn drop(&mut self) {
                let _ = self.manager.safeFrameNumber();
                self.drops
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(self.id);
            }
        }

        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(1, 0);
        let pool = GPUResourcePool::new(manager.clone(), 1);
        for id in 0..3 {
            pool.recycle(Some(erased(ResourceHandle::new(
                Some(manager.clone()),
                ReentrantProbe {
                    base: GPUResource::new(None),
                    id,
                    manager: manager.clone(),
                    drops: Arc::clone(&drops),
                },
            ))));
        }

        manager.advanceFrameNumber(1, 1);
        let acquired = pool.acquire().expect("oldest safe resource");
        assert_eq!(dropped(&drops), [1]);
        drop(acquired);
        assert_eq!(dropped(&drops), [1, 0]);
        drop(pool);
        assert_eq!(dropped(&drops), [1, 0, 2]);
        owner.shutdown();
    }

    #[test]
    fn pool_trims_one_inline_entry_before_reentrant_acquire() {
        #[repr(C)]
        struct ReentrantAcquireProbe {
            base: GPUResource,
            id: usize,
            pool: Option<Weak<ResourceHandle<GPUResourcePool>>>,
            order: Arc<Mutex<Vec<usize>>>,
        }
        impl_test_gpu_resource!(ReentrantAcquireProbe);

        impl Drop for ReentrantAcquireProbe {
            fn drop(&mut self) {
                self.order
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(self.id);
                let Some(pool) = self.pool.as_ref().and_then(Weak::upgrade) else {
                    return;
                };
                if let Some(resource) = pool.acquire() {
                    drop(resource);
                }
            }
        }

        let order = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(1, 0);
        let pool = Arc::new(GPUResourcePool::new(manager.clone(), 1));
        let weak_pool = Arc::downgrade(&pool);
        for (id, reentrant) in [(0, false), (1, true), (2, false), (3, false)] {
            pool.recycle(Some(erased(ResourceHandle::new(
                Some(manager.clone()),
                ReentrantAcquireProbe {
                    base: GPUResource::new(None),
                    id,
                    pool: reentrant.then(|| weak_pool.clone()),
                    order: Arc::clone(&order),
                },
            ))));
        }

        manager.advanceFrameNumber(1, 1);
        let returned = pool.acquire().expect("outer acquire returns entry zero");
        assert_eq!(dropped(&order), [1, 2]);
        assert_eq!(pool.m_pool.lock().unwrap().len(), 1);
        drop(returned);
        assert_eq!(dropped(&order), [1, 2, 0]);
        drop(pool);
        assert_eq!(dropped(&order), [1, 2, 0, 3]);
        owner.shutdown();
    }

    #[test]
    fn concurrent_frame_advance_and_recycle_preserve_fifo_invariants() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        let pool = GPUResourcePool::new(manager.clone(), 256);
        let barrier = Arc::new(Barrier::new(5));
        let mut workers = Vec::new();

        {
            let manager = manager.clone();
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                for frame in 1..=200 {
                    manager.advanceFrameNumber(frame, frame.saturating_sub(1));
                }
                Vec::new()
            }));
        }
        for lane in 0..4 {
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                (0..50).map(|offset| lane * 50 + offset).collect::<Vec<_>>()
            }));
        }
        let mut ids = Vec::with_capacity(200);
        for worker in workers {
            ids.extend(worker.join().expect("recording-thread worker"));
        }

        // The source payload is recording-thread confined. Workers therefore
        // only produce operation identities; the exact pool owner is retained
        // and recycled on its recording thread after the concurrent manager
        // advance has completed.
        manager.advanceFrameNumber(200, 200);
        for id in ids {
            pool.recycle(Some(erased(ResourceHandle::new(
                Some(manager.clone()),
                DropProbe::new(id, &drops),
            ))));
        }
        while let Some(resource) = pool.acquire() {
            drop(resource);
        }
        drop(pool);
        assert_eq!(dropped(&drops).len(), 200);
        owner.shutdown();
    }

    #[test]
    fn safe_handle_api_prevents_double_release_and_counter_rejects_underflow() {
        let count = LogicalRefCount::new();
        assert!(count.release());
        assert!(catch_unwind(AssertUnwindSafe(|| count.release())).is_err());

        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(1, 1);
        let resource = ResourceHandle::new(Some(manager.clone()), DropProbe::new(1, &drops));
        let clone = resource.clone();
        let pool = GPUResourcePool::new(manager.clone(), 1);
        let result = catch_unwind(AssertUnwindSafe(|| pool.recycle(Some(resource.erase()))));
        assert!(result.is_err());
        assert_eq!(clone.debugging_refcnt(), 1);
        drop(clone);
        assert_eq!(dropped(&drops), [1]);
        drop(pool);
        owner.shutdown();
    }

    #[test]
    fn frame_counters_are_monotonic_and_safe_never_exceeds_current() {
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advanceFrameNumber(4, 2);
        assert!(catch_unwind(AssertUnwindSafe(|| manager.advanceFrameNumber(3, 2))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| manager.advanceFrameNumber(4, 1))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| manager.advanceFrameNumber(4, 5))).is_err());
        owner.shutdown();
    }

    #[test]
    fn owner_drop_breaks_nonempty_purgatory_cycle_and_drops_payload() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        let weak_manager = Arc::downgrade(&manager.inner);
        manager.advanceFrameNumber(2, 0);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(8, &drops),
        ));
        drop(manager);

        assert!(weak_manager.upgrade().is_some());
        assert!(dropped(&drops).is_empty());
        owner.shutdown();
        drop(owner);
        assert_eq!(dropped(&drops), [8]);
        assert!(weak_manager.upgrade().is_none());
    }

    #[test]
    fn pooled_payload_drops_before_its_last_manager_field() {
        #[repr(C)]
        struct ManagerOrderProbe {
            base: GPUResource,
            manager: Weak<GPUResourceManagerInner>,
            observed_alive: Arc<AtomicUsize>,
        }
        impl_test_gpu_resource!(ManagerOrderProbe);

        impl Drop for ManagerOrderProbe {
            fn drop(&mut self) {
                if self.manager.upgrade().is_some() {
                    self.observed_alive.store(1, Ordering::Relaxed);
                }
            }
        }

        let observed_alive = Arc::new(AtomicUsize::new(0));
        let owner = GPUResourceManagerOwner::new();
        let manager = owner.manager();
        let weak_manager = Arc::downgrade(&manager.inner);
        let pool = GPUResourcePool::new(manager.clone(), 1);
        pool.recycle(Some(erased(ResourceHandle::new(
            Some(manager),
            ManagerOrderProbe {
                base: GPUResource::new(None),
                manager: weak_manager.clone(),
                observed_alive: Arc::clone(&observed_alive),
            },
        ))));

        owner.shutdown();
        drop(owner);
        drop(pool);
        assert_eq!(observed_alive.load(Ordering::Relaxed), 1);
        assert!(weak_manager.upgrade().is_none());
    }

    #[test]
    fn public_owners_preserve_send_sync_contracts() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ResourceHandle<DropProbe>>();
        assert_send_sync::<AnyResourceHandle>();
        assert_send_sync::<GPUResourceManager>();
        assert_send_sync::<GPUResourceManagerOwner>();
        assert_send_sync::<GPUResourcePool>();
    }
}

// This avoids a second embedded owner/count while preserving virtual drop.
