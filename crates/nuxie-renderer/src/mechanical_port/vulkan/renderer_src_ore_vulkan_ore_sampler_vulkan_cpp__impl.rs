//! Complete mechanical implementation translation of
//! `renderer/src/ore/vulkan/ore_sampler_vulkan.cpp`.

use ash::vk;

use super::ore_sampler_vulkan_decl::SamplerVulkan;

impl Drop for SamplerVulkan {
    fn drop(&mut self) {
        if self.m_vkSampler != vk::Sampler::null() {
            if let Some(destroy_sampler) = self.m_vkDestroySampler {
                unsafe {
                    destroy_sampler(
                        self.m_vkDevice,
                        self.m_vkSampler,
                        core::ptr::null(),
                    )
                };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ash::vk::Handle;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static DESTROY_COUNT: AtomicUsize = AtomicUsize::new(0);

    unsafe extern "system" fn count_destroy(
        _device: vk::Device,
        _sampler: vk::Sampler,
        _allocator: *const vk::AllocationCallbacks<'_>,
    ) {
        DESTROY_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    #[test]
    fn destroys_only_a_published_sampler_with_a_destroy_function() {
        DESTROY_COUNT.store(0, Ordering::Relaxed);
        drop(SamplerVulkan::new());
        assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 0);

        let mut sampler = SamplerVulkan::new();
        unsafe {
            sampler.setNativeSampler(
                vk::Device::from_raw(1),
                vk::Sampler::from_raw(2),
                count_destroy,
            );
        }
        drop(sampler);
        assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 1);
    }
}
