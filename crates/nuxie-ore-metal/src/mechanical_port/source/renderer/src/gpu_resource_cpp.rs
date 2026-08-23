/*
 * Copyright 2025 Rive
 */

// #include "rive/renderer/gpu_resource.hpp"

// Mechanical translation of the complete pinned source implementation
// renderer/src/gpu_resource.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

use std::fmt;
use std::marker::PhantomData;

// namespace rive::gpu

fn clone_pointer(pointer: &ResourcePointer) -> ResourcePointer {
    pointer.base().retain();
    pointer.clone()
}

fn release_pointer(pointer: ResourcePointer) {
    if !pointer.base().release() {
        return;
    }

    let manager = pointer.base().manager().cloned();
    let owner = ResourceOwner::new(pointer);
    if let Some(manager) = manager {
        manager.onRenderingResourceReleased(owner);
    }
    // With no manager, `owner` drops here and invokes the concrete destructor.
}

impl<T: GpuResourcePayload> Clone for ResourceHandle<T> {
    fn clone(&self) -> Self {
        Self {
            pointer: Some(clone_pointer(self.pointer())),
            marker: PhantomData,
        }
    }
}

impl<T: GpuResourcePayload> Drop for ResourceHandle<T> {
    fn drop(&mut self) {
        if let Some(pointer) = self.pointer.take() {
            release_pointer(pointer);
        }
    }
}

impl<T: GpuResourcePayload> fmt::Debug for ResourceHandle<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("rcp")
            .field("payload_type", &std::any::type_name::<T>())
            .field("logical_refcnt", &self.debugging_refcnt())
            .finish_non_exhaustive()
    }
}

impl Clone for AnyResourceHandle {
    fn clone(&self) -> Self {
        Self {
            pointer: Some(clone_pointer(self.pointer())),
        }
    }
}

impl Drop for AnyResourceHandle {
    fn drop(&mut self) {
        if let Some(pointer) = self.pointer.take() {
            release_pointer(pointer);
        }
    }
}

impl fmt::Debug for AnyResourceHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("rcp<GPUResource>")
            .field("logical_refcnt", &self.debugging_refcnt())
            .finish_non_exhaustive()
    }
}

/// Source `ref_rcp(T*)`, expressed without a lifetime-free raw pointer.
pub fn ref_rcp(resource: &AnyResourceHandle) -> AnyResourceHandle {
    resource.clone()
}

impl Drop for GPUResourceManagerInner {
    fn drop(&mut self) {
        let state = self
            .state
            .get_mut()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        debug_assert_eq!(
            state.currentFrameNumber, SHUTDOWN_FRAME_NUMBER,
            "GPUResourceManager must be shut down before destruction"
        );
        debug_assert_eq!(
            state.safeFrameNumber, SHUTDOWN_FRAME_NUMBER,
            "GPUResourceManager must be shut down before destruction"
        );
        debug_assert!(
            state.resourcePurgatory.is_empty(),
            "GPUResourceManager purgatory must be empty at destruction"
        );
    }
}

impl GPUResourceManager {
    // void GPUResourceManager::advanceFrameNumber(uint64_t nextFrameNumber,
    //                                              uint64_t safeFrameNumber)
    pub fn advanceFrameNumber(&self, nextFrameNumber: u64, safeFrameNumber: u64) {
        {
            let mut state = self.inner.lock_state();
            debug_assert!(nextFrameNumber >= state.currentFrameNumber);
            debug_assert!(safeFrameNumber >= state.safeFrameNumber);
            debug_assert!(safeFrameNumber <= nextFrameNumber);
            state.currentFrameNumber = nextFrameNumber;
            state.safeFrameNumber = safeFrameNumber;
            state.didAdvanceFrameNumber = true;
        }

        loop {
            let reclaim = {
                let mut state = self.inner.lock_state();
                if !state
                    .resourcePurgatory
                    .front()
                    .is_some_and(|resource| resource.lastFrameNumber <= state.safeFrameNumber)
                {
                    break;
                }
                let zombie = state
                    .resourcePurgatory
                    .pop_front()
                    .expect("purgatory front was present");
                debug_assert_eq!(
                    zombie.resource.pointer().base().debugging_refcnt(),
                    0,
                    "purgatory may only own zero-reference resources"
                );
                zombie.resource
            };
            // Permit source destructor re-entry without holding manager state.
            drop(reclaim);
        }
    }

    // void GPUResourceManager::onRenderingResourceReleased(GPUResource*)
    fn onRenderingResourceReleased(&self, resource: ResourceOwner) {
        debug_assert!(
            resource
                .pointer()
                .base()
                .manager()
                .is_some_and(|resource_manager| self.ptr_eq(resource_manager)),
            "only resources from this manager enter manager release"
        );
        debug_assert_eq!(resource.pointer().base().debugging_refcnt(), 0);

        let mut resource = Some(resource);
        {
            let mut state = self.inner.lock_state();
            if state.currentFrameNumber > state.safeFrameNumber || !state.didAdvanceFrameNumber {
                let lastFrameNumber = state.currentFrameNumber;
                if let Some(previous) = state.resourcePurgatory.back() {
                    debug_assert!(lastFrameNumber >= previous.lastFrameNumber);
                }
                state.resourcePurgatory.push_back(ZombieResource {
                    resource: resource.take().expect("single purgatory transfer"),
                    lastFrameNumber,
                });
            }
        }
        // Caught-up and shutdown releases destroy outside the manager lock.
        drop(resource);
    }

    // void GPUResourceManager::shutdown()
    pub fn shutdown(&self) {
        self.advanceFrameNumber(SHUTDOWN_FRAME_NUMBER, SHUTDOWN_FRAME_NUMBER);
    }
}

impl Drop for GPUResourceManagerOwner {
    fn drop(&mut self) {
        let state = self.manager.inner.lock_state();
        debug_assert_eq!(
            state.currentFrameNumber, SHUTDOWN_FRAME_NUMBER,
            "GPUResourceManager::shutdown() must be called after completion and before owner destruction"
        );
        debug_assert_eq!(
            state.safeFrameNumber, SHUTDOWN_FRAME_NUMBER,
            "GPUResourceManager::shutdown() must be called after completion and before owner destruction"
        );
        debug_assert!(
            state.resourcePurgatory.is_empty(),
            "GPUResourceManager purgatory must be empty at owner destruction"
        );
    }
}

impl GPUResourcePool {
    // rcp<GPUResource> GPUResourcePool::acquire()
    pub fn acquire(&self) -> Option<AnyResourceHandle> {
        let manager = self.inheritedManager();
        let resource = {
            // Source operations serialize manager state before pool state.
            let manager_state = manager.inner.lock_state();
            let safeFrameNumber = manager_state.safeFrameNumber;
            let mut pool = self
                .m_pool
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            if !pool
                .front()
                .is_some_and(|resource| resource.lastFrameNumber <= safeFrameNumber)
            {
                return None;
            }
            let zombie = pool.pop_front().expect("pool front was present");
            AnyResourceHandle::from_suspended(zombie.resource)
        };

        loop {
            let trimmed = {
                let manager_state = manager.inner.lock_state();
                let safeFrameNumber = manager_state.safeFrameNumber;
                let mut pool = self
                    .m_pool
                    .lock()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                if pool.len() <= self.m_maxPoolCount
                    || !pool
                        .front()
                        .is_some_and(|resource| resource.lastFrameNumber <= safeFrameNumber)
                {
                    break;
                }
                pool.pop_front()
                    .expect("safe excess pool front was present")
            };
            drop(trimmed);
        }

        debug_assert_eq!(resource.debugging_refcnt(), 1);
        Some(resource)
    }

    // void GPUResourcePool::recycle(rcp<GPUResource>)
    pub fn recycle(&self, mut resource: Option<AnyResourceHandle>) {
        let Some(mut resource) = resource.take() else {
            return;
        };
        debug_assert_eq!(resource.debugging_refcnt(), 1);

        let manager = self.inheritedManager();
        let manager_state = manager.inner.lock_state();
        let currentFrameNumber = manager_state.currentFrameNumber;
        let mut pool = self
            .m_pool
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(previous) = pool.back() {
            debug_assert!(currentFrameNumber >= previous.lastFrameNumber);
        }
        pool.push_back(ZombieResource {
            resource: resource.take_owner(),
            lastFrameNumber: currentFrameNumber,
        });
    }
}

// } // namespace rive::gpu
