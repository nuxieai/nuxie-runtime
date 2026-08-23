//! Complete mechanical implementation translation of
//! `renderer/src/ore/vulkan/ore_buffer_vulkan.cpp`.

#![allow(non_snake_case)]

use super::ore_buffer_vulkan_decl::{AllocationMirror, Backing, BufferVulkan};
use ash::vk;
use ash::vk::Handle;
use nuxie_ore_metal::buffer::{BufferApi, BufferUpdateError};
use nuxie_ore_metal::types::BufferUsage;
use vk_mem::{Alloc, AllocationCreateFlags, AllocationCreateInfo, MemoryUsage};

impl Drop for BufferVulkan {
    // BufferVulkan::~BufferVulkan()
    fn drop(&mut self) {
        unsafe {
            let pool = &mut *self.m_pool.get();
            if !pool.is_empty() {
                // `m_vkBuffer` and `m_vmaAllocation` mirror the current pool
                // entry. Destroy the pool entries, never the mirror.
                for backing in pool {
                    if let Some(mut allocation) = backing.vmaAllocation.take() {
                        self.vk()
                            .allocator()
                            .destroy_buffer(backing.vkBuffer, &mut allocation);
                    }
                }
                *self.m_vmaAllocation.get() = AllocationMirror::Null;
            } else if self.m_vkBuffer.get() != vk::Buffer::null() {
                // TextureVulkan staging buffer: the mirror is the owner.
                let allocation = core::mem::replace(
                    &mut *self.m_vmaAllocation.get(),
                    AllocationMirror::Null,
                );
                let AllocationMirror::Owned(mut allocation) = allocation else {
                    panic!("staging BufferVulkan requires its owned VMA allocation");
                };
                self.vk()
                    .allocator()
                    .destroy_buffer(self.m_vkBuffer.get(), &mut allocation);
            }

            // C++ destroys derived members in reverse declaration order, then
            // the Buffer base. The explicit sequence keeps the VulkanContext
            // and allocator alive through every native destruction above.
            core::mem::ManuallyDrop::drop(&mut self.m_pool);
            core::mem::ManuallyDrop::drop(&mut self.m_vk);
            core::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl BufferVulkan {
    // BufferVulkan::Backing BufferVulkan::allocateBacking()
    fn allocateBacking(&self) -> Backing {
        let buffer_info = vk::BufferCreateInfo::default()
            .size(self.size() as vk::DeviceSize)
            .usage(self.m_vkUsage);
        let allocation_info = AllocationCreateInfo {
            flags: AllocationCreateFlags::MAPPED,
            #[allow(deprecated)]
            usage: MemoryUsage::CpuToGpu,
            ..Default::default()
        };
        let (buffer, allocation) = match unsafe {
            self.vk()
                .allocator()
                .create_buffer(&buffer_info, &allocation_info)
        } {
            Ok(backing) => backing,
            Err(_) => return Backing::default(),
        };
        let mapped = self
            .vk()
            .allocator()
            .get_allocation_info(&allocation)
            .mapped_data
            .cast::<u8>();
        Backing {
            vkBuffer: buffer,
            vmaAllocation: Some(Box::new(allocation)),
            mappedPtr: mapped,
            frameStamp: 0,
        }
    }

    // void BufferVulkan::markBound()
    pub(crate) fn markBound(&self) {
        unsafe {
            let pool = &mut *self.m_pool.get();
            if !pool.is_empty() {
                pool[self.m_currentIndex.get()].frameStamp = self.manager().currentFrameNumber();
            }
        }
        self.m_boundSinceUpdate.set(true);
    }

    // bool BufferVulkan::acquireFreshBacking(uint32_t writeOffset,
    //                                         uint32_t writeSize)
    fn acquireFreshBacking(&self, writeOffset: u32, writeSize: u32) -> bool {
        let safe = self.manager().safeFrameNumber();
        let current = self.manager().currentFrameNumber();
        unsafe {
            let pool = &mut *self.m_pool.get();
            let oldMapped = pool[self.m_currentIndex.get()].mappedPtr;

            let mut fresh = pool.len();
            for (index, backing) in pool.iter().enumerate() {
                if index != self.m_currentIndex.get()
                    && backing.frameStamp <= safe
                    && backing.frameStamp < current
                {
                    fresh = index;
                    break;
                }
            }
            if fresh == pool.len() {
                let backing = self.allocateBacking();
                if backing.vkBuffer == vk::Buffer::null() {
                    return false;
                }
                pool.push(backing);
            }

            self.m_currentIndex.set(fresh);
            let backing = &mut pool[fresh];
            if !(writeOffset == 0 && writeSize == self.size()) {
                core::ptr::copy_nonoverlapping(oldMapped, backing.mappedPtr, self.size() as usize);
            }

            self.m_vkBuffer.set(backing.vkBuffer);
            let allocation = backing
                .vmaAllocation
                .as_deref_mut()
                .expect("pooled BufferVulkan backing requires a VMA allocation");
            *self.m_vmaAllocation.get() = AllocationMirror::Alias(allocation as *mut _);
            self.m_vkMappedPtr.set(backing.mappedPtr);
            true
        }
    }

    // void BufferVulkan::update(const void* data, uint32_t size,
    //                           uint32_t offset)
    pub(crate) fn update(
        &self,
        data: &[u8],
        size: u32,
        offset: u32,
    ) -> Result<(), BufferUpdateError> {
        let end = offset
            .checked_add(size)
            .ok_or(BufferUpdateError::RangeOverflow)?;
        debug_assert!(end <= self.size());
        if end > self.size() {
            return Err(BufferUpdateError::RangeOutOfBounds);
        }
        let source = data
            .get(..size as usize)
            .ok_or(BufferUpdateError::SourceTooShort)?;
        if self.m_boundSinceUpdate.get() && unsafe { !(&*self.m_pool.get()).is_empty() } {
            if self.acquireFreshBacking(offset, size) {
                self.m_boundSinceUpdate.set(false);
            }
        }
        let mapped = self.m_vkMappedPtr.get();
        assert!(!mapped.is_null());
        unsafe {
            core::ptr::copy_nonoverlapping(
                source.as_ptr(),
                mapped.add(offset as usize),
                size as usize,
            );
        }
        Ok(())
    }
}

impl BufferApi for BufferVulkan {
    fn size(&self) -> u32 {
        self.base.size()
    }

    fn usage(&self) -> BufferUsage {
        self.base.usage()
    }

    fn update(
        &self,
        data: &[u8],
        size: u32,
        offset: u32,
    ) -> Result<(), BufferUpdateError> {
        BufferVulkan::update(self, data, size, offset)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_ore_metal::gpu_resource::GPUResourceManagerOwner;

    #[test]
    fn checked_update_rejects_source_and_destination_range_errors() {
        let owner = GPUResourceManagerOwner::new();
        let buffer = BufferVulkan::new(owner.manager(), 8, BufferUsage::uniform);
        assert_eq!(
            buffer.update(&[1, 2], 3, 0),
            Err(BufferUpdateError::SourceTooShort)
        );
        assert_eq!(
            buffer.update(&[1], 1, u32::MAX),
            Err(BufferUpdateError::RangeOverflow)
        );
        let source_assert = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = buffer.update(&[1, 2], 2, 7);
        }));
        assert!(source_assert.is_err());
        drop(buffer);
        owner.shutdown();
    }
}
