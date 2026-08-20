// Mechanical translation of:
// - include/rive/refcnt.hpp
// - renderer/include/rive/renderer/gpu_resource.hpp
// - renderer/src/gpu_resource.cpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2021, 2025 Rive

use std::any::Any;
use std::collections::VecDeque;
use std::fmt;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

const SHUTDOWN_FRAME_NUMBER: u64 = u64::MAX;

/// Concrete value stored in a [`ResourceHandle`].
///
/// Upstream deletes `GPUResource` through a virtual destructor. `Any` provides
/// the corresponding checked type-erasure boundary, while `Send + Sync`
/// preserves `RefCnt`'s thread-safe handle contract.
pub trait GpuResourcePayload: Any + Send + Sync {}

impl<T: Any + Send + Sync> GpuResourcePayload for T {}

/// The intrusive count is intentionally independent from `Arc`'s allocation
/// count. At logical count zero, the last handle moves its allocation owner to
/// manager purgatory. Reconstructing an `Arc` after its strong count reached
/// zero would be unsound and would not model the upstream ownership transfer.
struct LogicalRefCount(AtomicUsize);

impl LogicalRefCount {
    fn new() -> Self {
        Self(AtomicUsize::new(1))
    }

    fn retain(&self) {
        let result = self
            .0
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |count| {
                count.checked_add(1)
            });
        assert!(result.is_ok(), "GPU resource reference count overflow");
    }

    /// Returns true when this release transitions the count from one to zero.
    fn release(&self) -> bool {
        let previous = self
            .0
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                count.checked_sub(1)
            });
        match previous {
            Ok(1) => true,
            Ok(_) => false,
            Err(_) => panic!("GPU resource reference count underflow"),
        }
    }

    fn load(&self) -> usize {
        self.0.load(Ordering::Relaxed)
    }
}

trait ErasedResourceAllocation: Any + Send + Sync {
    fn logical_ref_count(&self) -> &LogicalRefCount;
    fn manager(&self) -> Option<&GpuResourceManager>;
    fn payload(&self) -> &(dyn Any + Send + Sync);
}

struct ResourceAllocation<T: GpuResourcePayload> {
    logical_ref_count: LogicalRefCount,
    // C++ destroys the derived payload before the GPUResource base and its
    // manager field. Rust drops fields in declaration order, so payload must
    // precede manager here as well.
    payload: T,
    manager: Option<GpuResourceManager>,
}

impl<T: GpuResourcePayload> ErasedResourceAllocation for ResourceAllocation<T> {
    fn logical_ref_count(&self) -> &LogicalRefCount {
        &self.logical_ref_count
    }

    fn manager(&self) -> Option<&GpuResourceManager> {
        self.manager.as_ref()
    }

    fn payload(&self) -> &(dyn Any + Send + Sync) {
        &self.payload
    }
}

type ErasedAllocation = Arc<dyn ErasedResourceAllocation>;

fn clone_allocation(allocation: &ErasedAllocation) -> ErasedAllocation {
    allocation.logical_ref_count().retain();
    Arc::clone(allocation)
}

fn release_allocation(allocation: ErasedAllocation) {
    if !allocation.logical_ref_count().release() {
        return;
    }

    let manager = allocation.manager().cloned();
    if let Some(manager) = manager {
        manager.on_rendering_resource_released(allocation);
    }
    // Without a manager, or when the manager decides reclamation is already
    // safe, `allocation` drops here and invokes the concrete payload's Drop.
}

/// Thread-safe intrusive owner for a concrete GPU resource payload.
///
/// Cloning and dropping this value match upstream `rcp<T>::ref/unref`. The
/// allocation can outlive the last handle while queued in manager purgatory.
pub struct ResourceHandle<T: GpuResourcePayload> {
    allocation: Option<ErasedAllocation>,
    marker: PhantomData<fn() -> T>,
}

impl<T: GpuResourcePayload> ResourceHandle<T> {
    pub fn new(manager: Option<GpuResourceManager>, payload: T) -> Self {
        Self {
            allocation: Some(Arc::new(ResourceAllocation {
                logical_ref_count: LogicalRefCount::new(),
                payload,
                manager,
            })),
            marker: PhantomData,
        }
    }

    pub fn manager(&self) -> Option<&GpuResourceManager> {
        self.allocation().manager()
    }

    /// Mirrors upstream's debugging-only `debugging_refcnt()` query.
    pub fn debugging_ref_count(&self) -> usize {
        self.allocation().logical_ref_count().load()
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(self.allocation(), other.allocation())
    }

    pub fn erase(mut self) -> AnyResourceHandle {
        AnyResourceHandle {
            allocation: self.allocation.take(),
        }
    }

    fn allocation(&self) -> &ErasedAllocation {
        match self.allocation.as_ref() {
            Some(allocation) => allocation,
            None => unreachable!("a live resource handle always owns an allocation"),
        }
    }
}

impl<T: GpuResourcePayload> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            allocation: Some(clone_allocation(self.allocation())),
            marker: PhantomData,
        }
    }
}

impl<T: GpuResourcePayload> Deref for ResourceHandle<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self.allocation().payload().downcast_ref::<T>() {
            Some(payload) => payload,
            None => unreachable!("typed resource handle carries the wrong allocation type"),
        }
    }
}

impl<T: GpuResourcePayload> Drop for ResourceHandle<T> {
    fn drop(&mut self) {
        if let Some(allocation) = self.allocation.take() {
            release_allocation(allocation);
        }
    }
}

impl<T: GpuResourcePayload> fmt::Debug for ResourceHandle<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ResourceHandle")
            .field("payload_type", &std::any::type_name::<T>())
            .field("logical_ref_count", &self.debugging_ref_count())
            .finish_non_exhaustive()
    }
}

/// Type-erased `rcp<GPUResource>` equivalent used by purgatory and pools.
pub struct AnyResourceHandle {
    allocation: Option<ErasedAllocation>,
}

impl AnyResourceHandle {
    pub fn downcast<T: GpuResourcePayload>(mut self) -> Result<ResourceHandle<T>, Self> {
        let is_type = self.allocation().payload().downcast_ref::<T>().is_some();
        if !is_type {
            return Err(self);
        }

        Ok(ResourceHandle {
            allocation: self.allocation.take(),
            marker: PhantomData,
        })
    }

    pub fn downcast_ref<T: GpuResourcePayload>(&self) -> Option<&T> {
        self.allocation().payload().downcast_ref::<T>()
    }

    pub fn manager(&self) -> Option<&GpuResourceManager> {
        self.allocation().manager()
    }

    pub fn debugging_ref_count(&self) -> usize {
        self.allocation().logical_ref_count().load()
    }

    fn from_suspended(allocation: ErasedAllocation) -> Self {
        assert_eq!(
            allocation.logical_ref_count().load(),
            1,
            "a pooled resource must resume with exactly one logical owner"
        );
        Self {
            allocation: Some(allocation),
        }
    }

    fn take_allocation(&mut self) -> ErasedAllocation {
        match self.allocation.take() {
            Some(allocation) => allocation,
            None => unreachable!("a live resource handle always owns an allocation"),
        }
    }

    fn allocation(&self) -> &ErasedAllocation {
        match self.allocation.as_ref() {
            Some(allocation) => allocation,
            None => unreachable!("a live resource handle always owns an allocation"),
        }
    }
}

impl Clone for AnyResourceHandle {
    fn clone(&self) -> Self {
        Self {
            allocation: Some(clone_allocation(self.allocation())),
        }
    }
}

impl Drop for AnyResourceHandle {
    fn drop(&mut self) {
        if let Some(allocation) = self.allocation.take() {
            release_allocation(allocation);
        }
    }
}

impl fmt::Debug for AnyResourceHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AnyResourceHandle")
            .field("logical_ref_count", &self.debugging_ref_count())
            .finish_non_exhaustive()
    }
}

struct ZombieResource {
    allocation: ErasedAllocation,
    last_frame_number: u64,
}

struct ManagerState {
    current_frame_number: u64,
    safe_frame_number: u64,
    did_advance_frame_number: bool,
    resource_purgatory: VecDeque<ZombieResource>,
}

impl ManagerState {
    fn new() -> Self {
        Self {
            current_frame_number: 0,
            safe_frame_number: 0,
            did_advance_frame_number: false,
            resource_purgatory: VecDeque::new(),
        }
    }
}

struct GpuResourceManagerInner {
    state: Mutex<ManagerState>,
}

impl GpuResourceManagerInner {
    fn lock_state(&self) -> MutexGuard<'_, ManagerState> {
        self.state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

impl Drop for GpuResourceManagerInner {
    fn drop(&mut self) {
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        assert_eq!(
            state.current_frame_number, SHUTDOWN_FRAME_NUMBER,
            "GpuResourceManager must be shut down before destruction"
        );
        assert_eq!(
            state.safe_frame_number, SHUTDOWN_FRAME_NUMBER,
            "GpuResourceManager must be shut down before destruction"
        );
        assert!(
            state.resource_purgatory.is_empty(),
            "GpuResourceManager purgatory must be empty at destruction"
        );
    }
}

/// Owns the deferred-destruction queue for resources referenced by in-flight
/// command buffers.
#[derive(Clone)]
pub struct GpuResourceManager {
    inner: Arc<GpuResourceManagerInner>,
}

impl GpuResourceManager {
    fn new() -> Self {
        Self {
            inner: Arc::new(GpuResourceManagerInner {
                state: Mutex::new(ManagerState::new()),
            }),
        }
    }

    pub fn current_frame_number(&self) -> u64 {
        self.inner.lock_state().current_frame_number
    }

    pub fn safe_frame_number(&self) -> u64 {
        self.inner.lock_state().safe_frame_number
    }

    pub fn advance_frame_number(&self, next_frame_number: u64, safe_frame_number: u64) {
        {
            let mut state = self.inner.lock_state();
            assert!(
                next_frame_number >= state.current_frame_number,
                "current frame number must be monotonic"
            );
            assert!(
                safe_frame_number >= state.safe_frame_number,
                "safe frame number must be monotonic"
            );
            assert!(
                safe_frame_number <= next_frame_number,
                "safe frame number cannot exceed the current frame"
            );

            state.current_frame_number = next_frame_number;
            state.safe_frame_number = safe_frame_number;
            state.did_advance_frame_number = true;
        }

        loop {
            let reclaim = {
                let mut state = self.inner.lock_state();
                if !state
                    .resource_purgatory
                    .front()
                    .is_some_and(|zombie| zombie.last_frame_number <= state.safe_frame_number)
                {
                    break;
                }
                let zombie = match state.resource_purgatory.pop_front() {
                    Some(zombie) => zombie,
                    None => unreachable!("the purgatory front was present"),
                };
                assert_eq!(
                    zombie.allocation.logical_ref_count().load(),
                    0,
                    "purgatory may only own zero-reference resources"
                );
                zombie.allocation
            };
            // Upstream's pop_front destroys this entry inline. Drop exactly
            // one outside the lock, then reacquire and re-evaluate so a
            // reentrant destructor observes the remaining FIFO immediately.
            drop(reclaim);
        }
    }

    fn shutdown(&self) {
        self.advance_frame_number(SHUTDOWN_FRAME_NUMBER, SHUTDOWN_FRAME_NUMBER);
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.inner, &other.inner)
    }

    fn on_rendering_resource_released(&self, allocation: ErasedAllocation) {
        let allocation_manager = match allocation.manager() {
            Some(manager) => manager,
            None => unreachable!("only managed resources enter manager release"),
        };
        assert!(
            self.ptr_eq(allocation_manager),
            "resource released through a different manager"
        );
        assert_eq!(
            allocation.logical_ref_count().load(),
            0,
            "only zero-reference resources enter manager release"
        );

        let mut allocation = Some(allocation);
        {
            let mut state = self.inner.lock_state();
            if state.current_frame_number > state.safe_frame_number
                || !state.did_advance_frame_number
            {
                let last_frame_number = state.current_frame_number;
                if let Some(previous) = state.resource_purgatory.back() {
                    assert!(
                        last_frame_number >= previous.last_frame_number,
                        "purgatory frame numbers must be FIFO-monotonic"
                    );
                }
                let queued = match allocation.take() {
                    Some(allocation) => allocation,
                    None => unreachable!("released allocation has one queue owner"),
                };
                state.resource_purgatory.push_back(ZombieResource {
                    allocation: queued,
                    last_frame_number,
                });
            }
        }
        // A caught-up or shut-down manager deletes immediately outside the
        // manager lock, matching upstream and permitting destructor reentry.
        drop(allocation);
    }
}

/// Non-clone root authority for a [`GpuResourceManager`].
///
/// Resources strongly retain cloneable manager handles, and manager
/// purgatory strongly retains released resource allocations. This unique root
/// breaks that intentional cycle by shutting the manager down on drop. Manager
/// handles cannot be constructed through the public safe API without first
/// creating this owner.
pub struct GpuResourceManagerOwner {
    manager: GpuResourceManager,
}

impl GpuResourceManagerOwner {
    pub fn new() -> Self {
        Self {
            manager: GpuResourceManager::new(),
        }
    }

    pub fn manager(&self) -> GpuResourceManager {
        self.manager.clone()
    }

    /// Idempotently enters shutdown and releases every purgatory allocation.
    pub fn shutdown(&self) {
        self.manager.shutdown();
    }
}

impl Default for GpuResourceManagerOwner {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for GpuResourceManagerOwner {
    fn drop(&mut self) {
        self.manager.shutdown();
    }
}

impl fmt::Debug for GpuResourceManager {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("GpuResourceManager")
            .field("current_frame_number", &self.current_frame_number())
            .field("safe_frame_number", &self.safe_frame_number())
            .finish_non_exhaustive()
    }
}

/// Manual FIFO recycling pool for type-erased GPU resources.
///
/// Pool entries retain a logical count of one while the pool uniquely owns
/// the allocation, exactly like upstream's `unique_ptr` adopted from an
/// `rcp::release()`. Dropping or trimming an entry destroys it directly rather
/// than sending it through manager purgatory.
pub struct GpuResourcePool {
    manager: GpuResourceManager,
    max_pool_count: usize,
    pool: Mutex<VecDeque<ZombieResource>>,
}

impl GpuResourcePool {
    pub fn new(manager: GpuResourceManager, max_pool_count: usize) -> Self {
        Self {
            manager,
            max_pool_count,
            pool: Mutex::new(VecDeque::new()),
        }
    }

    /// Constructs the inheritance-equivalent form of upstream
    /// `GPUResourcePool : GPUResource`. The direct [`new`](Self::new) form is
    /// also retained because upstream permits pools as stack/member values.
    pub fn new_managed(manager: GpuResourceManager, max_pool_count: usize) -> ResourceHandle<Self> {
        ResourceHandle::new(Some(manager.clone()), Self::new(manager, max_pool_count))
    }

    pub fn acquire(&self) -> Option<AnyResourceHandle> {
        let resource = {
            // Pool operations always lock manager state before pool state.
            // Holding the manager snapshot lock through the pool mutation
            // prevents concurrent frame advancement from reordering entries.
            let manager_state = self.manager.inner.lock_state();
            let safe_frame_number = manager_state.safe_frame_number;
            let mut pool = self
                .pool
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !pool
                .front()
                .is_some_and(|zombie| zombie.last_frame_number <= safe_frame_number)
            {
                return None;
            }

            let zombie = match pool.pop_front() {
                Some(zombie) => zombie,
                None => unreachable!("the pool front was present"),
            };
            AnyResourceHandle::from_suspended(zombie.allocation)
        };

        loop {
            let trimmed = {
                let manager_state = self.manager.inner.lock_state();
                let safe_frame_number = manager_state.safe_frame_number;
                let mut pool = self
                    .pool
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if pool.len() <= self.max_pool_count
                    || !pool
                        .front()
                        .is_some_and(|zombie| zombie.last_frame_number <= safe_frame_number)
                {
                    break;
                }
                match pool.pop_front() {
                    Some(zombie) => zombie,
                    None => unreachable!("the safe excess pool front was present"),
                }
            };
            // Match deque::pop_front's inline unique_ptr destruction while
            // still avoiding destructor reentry under either Rust lock.
            drop(trimmed);
        }

        assert_eq!(resource.debugging_ref_count(), 1);
        Some(resource)
    }

    pub fn recycle<T: GpuResourcePayload>(&self, resource: Option<ResourceHandle<T>>) {
        self.recycle_erased(resource.map(ResourceHandle::erase));
    }

    pub fn recycle_erased(&self, resource: Option<AnyResourceHandle>) {
        let Some(mut resource) = resource else {
            return;
        };
        assert_eq!(
            resource.debugging_ref_count(),
            1,
            "only a uniquely referenced resource can be recycled"
        );

        // Match acquire's manager-state-then-pool order and keep the frame
        // snapshot stable until the entry is appended.
        let manager_state = self.manager.inner.lock_state();
        let current_frame_number = manager_state.current_frame_number;
        let mut pool = self
            .pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(previous) = pool.back() {
            assert!(
                current_frame_number >= previous.last_frame_number,
                "pool frame numbers must be FIFO-monotonic"
            );
        }
        pool.push_back(ZombieResource {
            allocation: resource.take_allocation(),
            last_frame_number: current_frame_number,
        });
    }

    #[cfg(test)]
    fn len(&self) -> usize {
        self.pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use std::sync::atomic::AtomicUsize;
    use std::sync::{Barrier, Weak};
    use std::thread;

    struct DropProbe {
        id: usize,
        drops: Arc<Mutex<Vec<usize>>>,
    }

    impl DropProbe {
        fn new(id: usize, drops: &Arc<Mutex<Vec<usize>>>) -> Self {
            Self {
                id,
                drops: Arc::clone(drops),
            }
        }
    }

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
    fn unmanaged_resource_deletes_immediately_at_zero() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let resource = ResourceHandle::new(None, DropProbe::new(1, &drops));
        let clone = resource.clone();
        assert_eq!(resource.debugging_ref_count(), 2);
        drop(clone);
        assert!(dropped(&drops).is_empty());
        drop(resource);
        assert_eq!(dropped(&drops), [1]);
    }

    #[test]
    fn pre_first_frame_release_waits_even_when_frame_zero_is_caught_up() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(1, &drops),
        ));
        assert!(dropped(&drops).is_empty());

        manager.advance_frame_number(0, 0);
        assert_eq!(dropped(&drops), [1]);
        owner.shutdown();
    }

    #[test]
    fn caught_up_manager_releases_immediately() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(3, 3);
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
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(2, 0);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(1, &drops),
        ));
        manager.advance_frame_number(3, 0);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(2, &drops),
        ));

        manager.advance_frame_number(4, 1);
        assert!(dropped(&drops).is_empty());
        manager.advance_frame_number(4, 2);
        assert_eq!(dropped(&drops), [1]);
        manager.advance_frame_number(4, 3);
        assert_eq!(dropped(&drops), [1, 2]);
        owner.shutdown();
    }

    #[test]
    fn pool_recycles_oldest_safe_resource_without_changing_ref_count() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(2, 0);
        let resource = ResourceHandle::new(Some(manager.clone()), DropProbe::new(7, &drops));
        let pool = GpuResourcePool::new(manager.clone(), 4);
        pool.recycle(Some(resource));
        assert!(pool.acquire().is_none());

        manager.advance_frame_number(2, 2);
        let resource = pool.acquire().expect("safe pooled resource");
        assert_eq!(resource.debugging_ref_count(), 1);
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
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(1, 0);
        let pool = GpuResourcePool::new(manager.clone(), 2);
        for id in 0..4 {
            pool.recycle(Some(ResourceHandle::new(
                Some(manager.clone()),
                DropProbe::new(id, &drops),
            )));
        }
        assert_eq!(pool.len(), 4);

        manager.advance_frame_number(1, 1);
        let acquired = pool.acquire().expect("oldest safe resource");
        assert_eq!(
            acquired.downcast_ref::<DropProbe>().map(|probe| probe.id),
            Some(0)
        );
        assert_eq!(pool.len(), 2);
        assert_eq!(dropped(&drops), [1]);

        drop(pool);
        assert_eq!(dropped(&drops), [1, 2, 3]);
        drop(acquired);
        assert_eq!(dropped(&drops), [1, 2, 3, 0]);
        owner.shutdown();
    }

    #[test]
    fn pool_is_type_erased_but_downcast_is_checked_and_destructors_are_concrete() {
        struct OtherProbe(Arc<AtomicUsize>);
        impl Drop for OtherProbe {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        let drops = Arc::new(Mutex::new(Vec::new()));
        let other_drops = Arc::new(AtomicUsize::new(0));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(1, 1);
        let pool = GpuResourcePool::new(manager.clone(), 4);
        pool.recycle(Some(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(4, &drops),
        )));
        pool.recycle(Some(ResourceHandle::new(
            Some(manager.clone()),
            OtherProbe(Arc::clone(&other_drops)),
        )));

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
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(2, 0);
        let pool = GpuResourcePool::new_managed(manager.clone(), 4);
        pool.recycle(Some(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(9, &drops),
        )));

        drop(pool);
        assert!(dropped(&drops).is_empty());
        manager.advance_frame_number(2, 2);
        assert_eq!(dropped(&drops), [9]);
        owner.shutdown();
    }

    #[test]
    fn shutdown_purges_zombies_and_makes_future_releases_immediate() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(5, 2);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(1, &drops),
        ));
        assert!(dropped(&drops).is_empty());

        owner.shutdown();
        owner.shutdown();
        assert_eq!(dropped(&drops), [1]);
        assert_eq!(manager.current_frame_number(), u64::MAX);
        assert_eq!(manager.safe_frame_number(), u64::MAX);

        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(2, &drops),
        ));
        assert_eq!(dropped(&drops), [1, 2]);
    }

    #[test]
    fn reference_count_is_thread_safe_and_releases_once() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
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
        assert_eq!(resource.debugging_ref_count(), 1);
        drop(resource);
        assert!(dropped(&drops).is_empty());
        owner.shutdown();
        assert_eq!(dropped(&drops), [1]);
    }

    #[test]
    fn final_release_on_worker_thread_enters_purgatory_once() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(3, 1);
        let resource = ResourceHandle::new(Some(manager), DropProbe::new(5, &drops));

        assert!(thread::spawn(move || drop(resource)).join().is_ok());
        assert!(dropped(&drops).is_empty());
        owner.shutdown();
        assert_eq!(dropped(&drops), [5]);
    }

    #[test]
    fn manager_purge_drops_reentrant_payload_outside_state_lock() {
        struct ReentrantProbe {
            manager: GpuResourceManager,
            drops: Arc<AtomicUsize>,
        }

        impl Drop for ReentrantProbe {
            fn drop(&mut self) {
                let _ = self.manager.current_frame_number();
                let _ = self.manager.safe_frame_number();
                self.drops.fetch_add(1, Ordering::Relaxed);
            }
        }

        let drops = Arc::new(AtomicUsize::new(0));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(2, 0);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            ReentrantProbe {
                manager,
                drops: Arc::clone(&drops),
            },
        ));

        owner.shutdown();
        assert_eq!(drops.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn manager_reclaims_one_inline_entry_before_reentrant_safe_frame_advance() {
        struct ReentrantAdvanceProbe {
            id: usize,
            manager: GpuResourceManager,
            reentrant_safe_frame: Option<u64>,
            order: Arc<Mutex<Vec<usize>>>,
        }

        impl Drop for ReentrantAdvanceProbe {
            fn drop(&mut self) {
                self.order
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(self.id);
                if let Some(safe_frame) = self.reentrant_safe_frame {
                    self.manager.advance_frame_number(3, safe_frame);
                }
            }
        }

        let order = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        for (frame, id, reentrant_safe_frame) in [(1, 1, Some(3)), (2, 2, None), (3, 3, None)] {
            manager.advance_frame_number(frame, 0);
            drop(ResourceHandle::new(
                Some(manager.clone()),
                ReentrantAdvanceProbe {
                    id,
                    manager: manager.clone(),
                    reentrant_safe_frame,
                    order: Arc::clone(&order),
                },
            ));
        }

        manager.advance_frame_number(3, 2);
        assert_eq!(dropped(&order), [1, 2, 3]);
        owner.shutdown();
    }

    #[test]
    fn pool_trim_and_release_drop_reentrant_payloads_outside_both_locks() {
        struct ReentrantProbe {
            id: usize,
            manager: GpuResourceManager,
            drops: Arc<Mutex<Vec<usize>>>,
        }

        impl Drop for ReentrantProbe {
            fn drop(&mut self) {
                let _ = self.manager.safe_frame_number();
                self.drops
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner)
                    .push(self.id);
            }
        }

        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(1, 0);
        let pool = GpuResourcePool::new(manager.clone(), 1);
        for id in 0..3 {
            pool.recycle(Some(ResourceHandle::new(
                Some(manager.clone()),
                ReentrantProbe {
                    id,
                    manager: manager.clone(),
                    drops: Arc::clone(&drops),
                },
            )));
        }

        manager.advance_frame_number(1, 1);
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
        struct ReentrantAcquireProbe {
            id: usize,
            pool: Option<Weak<GpuResourcePool>>,
            order: Arc<Mutex<Vec<usize>>>,
        }

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
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(1, 0);
        let pool = Arc::new(GpuResourcePool::new(manager.clone(), 1));
        let weak_pool = Arc::downgrade(&pool);
        for (id, reentrant) in [(0, false), (1, true), (2, false), (3, false)] {
            pool.recycle(Some(ResourceHandle::new(
                Some(manager.clone()),
                ReentrantAcquireProbe {
                    id,
                    pool: reentrant.then(|| weak_pool.clone()),
                    order: Arc::clone(&order),
                },
            )));
        }

        manager.advance_frame_number(1, 1);
        let returned = pool.acquire().expect("outer acquire returns entry zero");
        assert_eq!(dropped(&order), [1, 2]);
        assert_eq!(pool.len(), 1);
        drop(returned);
        assert_eq!(dropped(&order), [1, 2, 0]);
        drop(pool);
        assert_eq!(dropped(&order), [1, 2, 0, 3]);
        owner.shutdown();
    }

    #[test]
    fn concurrent_frame_advance_and_recycle_preserve_fifo_invariants() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        let pool = Arc::new(GpuResourcePool::new(manager.clone(), 256));
        let barrier = Arc::new(Barrier::new(5));
        let mut workers = Vec::new();

        {
            let manager = manager.clone();
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                for frame in 1..=200 {
                    manager.advance_frame_number(frame, frame.saturating_sub(1));
                }
            }));
        }
        for lane in 0..4 {
            let manager = manager.clone();
            let pool = Arc::clone(&pool);
            let drops = Arc::clone(&drops);
            let barrier = Arc::clone(&barrier);
            workers.push(thread::spawn(move || {
                barrier.wait();
                for offset in 0..50 {
                    let id = lane * 50 + offset;
                    pool.recycle(Some(ResourceHandle::new(
                        Some(manager.clone()),
                        DropProbe::new(id, &drops),
                    )));
                }
            }));
        }
        for worker in workers {
            assert!(worker.join().is_ok());
        }

        manager.advance_frame_number(200, 200);
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
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(1, 1);
        let resource = ResourceHandle::new(Some(manager.clone()), DropProbe::new(1, &drops));
        let clone = resource.clone();
        let pool = GpuResourcePool::new(manager.clone(), 1);
        let result = catch_unwind(AssertUnwindSafe(|| pool.recycle(Some(resource))));
        assert!(result.is_err());
        assert_eq!(clone.debugging_ref_count(), 1);
        drop(clone);
        assert_eq!(dropped(&drops), [1]);
        drop(pool);
        owner.shutdown();
    }

    #[test]
    fn frame_counters_are_monotonic_and_safe_never_exceeds_current() {
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        manager.advance_frame_number(4, 2);
        assert!(catch_unwind(AssertUnwindSafe(|| manager.advance_frame_number(3, 2))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| manager.advance_frame_number(4, 1))).is_err());
        assert!(catch_unwind(AssertUnwindSafe(|| manager.advance_frame_number(4, 5))).is_err());
        owner.shutdown();
    }

    #[test]
    fn owner_drop_breaks_nonempty_purgatory_cycle_and_drops_payload() {
        let drops = Arc::new(Mutex::new(Vec::new()));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        let weak_manager = Arc::downgrade(&manager.inner);
        manager.advance_frame_number(2, 0);
        drop(ResourceHandle::new(
            Some(manager.clone()),
            DropProbe::new(8, &drops),
        ));
        drop(manager);

        assert!(weak_manager.upgrade().is_some());
        assert!(dropped(&drops).is_empty());
        drop(owner);
        assert_eq!(dropped(&drops), [8]);
        assert!(weak_manager.upgrade().is_none());
    }

    #[test]
    fn pooled_payload_drops_before_its_last_manager_field() {
        struct ManagerOrderProbe {
            manager: Weak<GpuResourceManagerInner>,
            observed_alive: Arc<AtomicUsize>,
        }

        impl Drop for ManagerOrderProbe {
            fn drop(&mut self) {
                if self.manager.upgrade().is_some() {
                    self.observed_alive.store(1, Ordering::Relaxed);
                }
            }
        }

        let observed_alive = Arc::new(AtomicUsize::new(0));
        let owner = GpuResourceManagerOwner::new();
        let manager = owner.manager();
        let weak_manager = Arc::downgrade(&manager.inner);
        let pool = GpuResourcePool::new(manager.clone(), 1);
        pool.recycle(Some(ResourceHandle::new(
            Some(manager),
            ManagerOrderProbe {
                manager: weak_manager.clone(),
                observed_alive: Arc::clone(&observed_alive),
            },
        )));

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
        assert_send_sync::<GpuResourceManager>();
        assert_send_sync::<GpuResourceManagerOwner>();
        assert_send_sync::<GpuResourcePool>();
    }
}
