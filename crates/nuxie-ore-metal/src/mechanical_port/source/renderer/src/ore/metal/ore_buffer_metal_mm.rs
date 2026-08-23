/*
 * Copyright 2025 Rive
 */

// #include "ore_buffer_metal.hpp"
// #include "rive/renderer/ore/ore_context_metal.hpp"
// #include "rive/rive_types.hpp"

// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source implementation
// renderer/src/ore/metal/ore_buffer_metal.mm.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![cfg(target_vendor = "apple")]
use super::ore_buffer_metal_hpp::{Backing, BufferMetalState, NativeMetalBuffer};
use super::*;

#[cfg(target_vendor = "apple")]
use objc2_foundation::NSString;
#[cfg(target_vendor = "apple")]
use objc2_metal::{MTLBuffer, MTLDevice, MTLResource, MTLResourceOptions};

// namespace rive::ore

impl BufferMetal {
    // void BufferMetal::markBound()
    pub fn markBound(&self) {
        let mut state = self.lockState();
        let current = state.m_currentIndex;
        state.m_pool[current].serial = self.m_contextState.currentSerial();
        state.m_boundSinceUpdate = true;
    }

    /// Source-ordered `markBound(); current()` as one recording-domain
    /// operation. This prevents the safe Rust update seam from orphaning
    /// between the two source expressions while preserving the selected
    /// backing and serial written by the pinned implementation.
    pub fn currentAndMarkBound(&self) -> NativeMetalBuffer {
        let mut state = self.lockState();
        let current = state.m_currentIndex;
        state.m_pool[current].serial = self.m_contextState.currentSerial();
        state.m_boundSinceUpdate = true;
        state.m_pool[current].mtl.clone()
    }

    // bool BufferMetal::acquireFreshBacking(uint32_t writeOffset,
    //                                        uint32_t writeSize)
    fn acquireFreshBacking(
        &self,
        state: &mut BufferMetalState,
        writeOffset: u32,
        writeSize: u32,
    ) -> bool {
        let old = state.m_pool[state.m_currentIndex]
            .mtl
            .clone()
            .expect("BufferMetal backing must have a native MTLBuffer");
        // Reuse a backing the GPU has finished with, else allocate one.
        let mut fresh: usize = state.m_pool.len();
        for i in 0..state.m_pool.len() {
            if i != state.m_currentIndex
                && self.m_contextState.isSerialComplete(state.m_pool[i].serial)
            {
                fresh = i;
                break;
            }
        }
        if fresh == state.m_pool.len() {
            let mtl = self.m_device.newBufferWithLength_options(
                self.base.size() as usize,
                MTLResourceOptions::StorageModeShared,
            );
            let Some(mtl) = mtl else {
                // Keep the current backing. Report it like the D3D12 path so the
                // degraded (racy) fallback is diagnosable.
                self.m_contextState.setLastError(
                    "ore: Metal buffer backing allocation failed; reusing in flight backing for this update",
                );
                return false;
            };
            if !state.m_label.is_empty() {
                // Objective-C++: `mtl.label = @(m_label.c_str());`
                mtl.setLabel(Some(&NSString::from_str(&state.m_label)));
            }
            state.m_pool.push(Backing {
                mtl: Some(mtl),
                serial: 0,
            });
        }

        state.m_currentIndex = fresh;
        // Carry contents forward so a partial update keeps untouched bytes. Skip
        // when this update covers the whole buffer.
        if !(writeOffset == 0 && writeSize == self.base.size()) {
            // SAFETY: both source and destination are MTL buffers allocated for
            // `m_size` bytes, and the source and destination pool entries are
            // distinct because the reuse scan excludes `m_currentIndex`.
            unsafe {
                let current = state.m_pool[state.m_currentIndex]
                    .mtl
                    .as_ref()
                    .expect("BufferMetal backing must have a native MTLBuffer");
                std::ptr::copy_nonoverlapping(
                    old.contents().as_ptr().cast::<u8>(),
                    current.contents().as_ptr().cast::<u8>(),
                    self.base.size() as usize,
                );
            }
        }
        true
    }

    // void BufferMetal::update(const void* data,
    //                           uint32_t size,
    //                           uint32_t offset)
    pub fn update(&self, data: &[u8], size: u32, offset: u32) -> Result<(), BufferUpdateError> {
        let end = offset
            .checked_add(size)
            .ok_or(BufferUpdateError::RangeOverflow)?;
        debug_assert!(end <= self.base.size());
        if end > self.base.size() {
            return Err(BufferUpdateError::RangeOutOfBounds);
        }
        let source = data
            .get(..size as usize)
            .ok_or(BufferUpdateError::SourceTooShort)?;
        let mut state = self.lockState();
        if state.m_boundSinceUpdate {
            // On allocation failure keep the current backing and retry next update.
            if self.acquireFreshBacking(&mut state, offset, size) {
                state.m_boundSinceUpdate = false;
            }
        }
        // `size` remains explicit, matching the source const-void-pointer API;
        // the borrowed slice supplies that pointer's provenance.
        unsafe {
            let current = state.m_pool[state.m_currentIndex]
                .mtl
                .as_ref()
                .expect("BufferMetal backing must have a native MTLBuffer");
            std::ptr::copy_nonoverlapping(
                source.as_ptr(),
                current
                    .contents()
                    .as_ptr()
                    .cast::<u8>()
                    .add(offset as usize),
                size as usize,
            );
        }
        Ok(())
    }
}

impl BufferApi for BufferMetal {
    fn size(&self) -> u32 {
        self.base.size()
    }

    fn usage(&self) -> BufferUsage {
        self.base.usage()
    }

    fn update(&self, data: &[u8], size: u32, offset: u32) -> Result<(), BufferUpdateError> {
        BufferMetal::update(self, data, size, offset)
    }
}

// } // namespace rive::ore
