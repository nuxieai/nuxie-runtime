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
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard, Weak};

use super::ore::ore_buffer_hpp::{BufferApi, BufferUpdateError};
use super::ore::ore_texture_hpp::{TextureApi, TextureUploadError};
use super::ore::ore_types_hpp::{BufferUsage, TextureDataDesc, TextureFormat, TextureType};

// namespace rive::gpu

pub(crate) const SHUTDOWN_FRAME_NUMBER: u64 = u64::MAX;

/// Opaque execution-domain identity for safe Rust resource entry points.
/// Pinned C++ carries this as the implicit `Context*`/backend precondition;
/// the weak token preserves that non-owning relationship without widening it.
#[derive(Clone)]
pub struct ResourceDomain(Weak<()>);

impl ResourceDomain {
    pub(crate) fn new(owner: &Arc<()>) -> Self {
        Self(Arc::downgrade(owner))
    }

    fn matches(&self, other: &Self) -> bool {
        Weak::ptr_eq(&self.0, &other.0) && self.0.strong_count() != 0
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
pub unsafe trait GpuResourcePayload: Any + Send {
    fn gpu_resource(&self) -> &GPUResource;
    fn gpu_resource_mut(&mut self) -> &mut GPUResource;
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
    for<'a> unsafe fn(NonNull<GPUResource>, &TextureDataDesc<'a>) -> Result<(), TextureUploadError>;

#[derive(Clone, Copy)]
pub(crate) struct ResourceVTable {
    type_id: fn() -> TypeId,
    destroy: DestroyResource,
    buffer_info: Option<BufferInfoDispatch>,
    buffer_update: Option<BufferUpdateDispatch>,
    texture_info: Option<TextureInfoDispatch>,
    texture_upload: Option<TextureUploadDispatch>,
}

unsafe fn destroy_resource<T: GpuResourcePayload>(base: NonNull<GPUResource>) {
    unsafe { drop(Box::from_raw(base.cast::<T>().as_ptr())) };
}

fn type_id<T: GpuResourcePayload>() -> TypeId {
    TypeId::of::<T>()
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
    base: NonNull<GPUResource>,
    data: &TextureDataDesc<'_>,
) -> Result<(), TextureUploadError> {
    unsafe { base.cast::<T>().as_ref() }.upload(data)
}

fn plain_vtable<T: GpuResourcePayload>() -> ResourceVTable {
    ResourceVTable {
        type_id: type_id::<T>,
        destroy: destroy_resource::<T>,
        buffer_info: None,
        buffer_update: None,
        texture_info: None,
        texture_upload: None,
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
}

impl Drop for ResourceOwner {
    fn drop(&mut self) {
        if let Some(pointer) = self.pointer.take() {
            unsafe { (pointer.vtable.destroy)(pointer.base) };
        }
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
        unsafe { dispatch(pointer.base, data) }
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
