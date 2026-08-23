//! Complete mechanical implementation translation of
//! `renderer/src/ore/vulkan/ore_shader_module_vulkan.cpp`.

use ash::vk;

use super::ore_shader_module_vulkan_decl::ShaderModuleVulkan;

impl Drop for ShaderModuleVulkan {
    fn drop(&mut self) {
        if self.m_vkShaderModule != vk::ShaderModule::null() {
            if let Some(destroy_shader_module) = self.m_vkDestroyShaderModule {
                unsafe {
                    destroy_shader_module(
                        self.m_vkDevice,
                        self.m_vkShaderModule,
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
        _shader_module: vk::ShaderModule,
        _allocator: *const vk::AllocationCallbacks<'_>,
    ) {
        DESTROY_COUNT.fetch_add(1, Ordering::Relaxed);
    }

    #[test]
    fn destroys_only_a_published_module_with_a_destroy_function() {
        DESTROY_COUNT.store(0, Ordering::Relaxed);
        drop(ShaderModuleVulkan::new());
        assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 0);

        let mut module = ShaderModuleVulkan::new();
        unsafe {
            module.setNativeShaderModule(
                vk::Device::from_raw(1),
                vk::ShaderModule::from_raw(2),
                count_destroy,
            );
        }
        drop(module);
        assert_eq!(DESTROY_COUNT.load(Ordering::Relaxed), 1);
    }
}
