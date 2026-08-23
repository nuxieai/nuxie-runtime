//! Complete mechanical implementation translation of
//! `renderer/src/ore/vulkan/ore_bind_group_vulkan.cpp`.

#![allow(non_snake_case)]

use super::ore_bind_group_vulkan_decl::{BindGroupVulkan, CachedSet};
use ash::vk;

impl BindGroupVulkan {
    pub(crate) fn markUBOsBound(&self) {
        for write in self.m_uboWrites.iter() {
            assert!(!write.buffer.is_null());
            // SAFETY: the backend-independent retained-buffer graph owns every
            // concrete BufferVulkan referenced by this replay recipe.
            unsafe { (&*write.buffer).markBound() };
        }
    }

    pub(crate) fn resolveDescriptorSet(&self) -> vk::DescriptorSet {
        // C++ rcp aliases permit recording-thread mutation through a
        // BindGroup*. Rust confines that mutation to this exact cache cell.
        let setCache = unsafe { &mut *self.m_setCache.get() };
        for cached in setCache.iter() {
            let mut matches = true;
            for (index, write) in self.m_uboWrites.iter().enumerate() {
                assert!(!write.buffer.is_null());
                let current = unsafe { (&*write.buffer).current() };
                if cached.key[index] != current {
                    matches = false;
                    break;
                }
            }
            if matches {
                return cached.set;
            }
        }

        let descriptor_set_layout = self.m_vkDSL;
        let alloc = self
            .context_mut()
            .vkAllocateDescriptorSet(descriptor_set_layout);
        if alloc.set == vk::DescriptorSet::null() {
            return vk::DescriptorSet::null();
        }

        const MAX_WRITES: usize = 32;
        assert!(self.m_uboWrites.len() + self.m_imageWrites.len() <= MAX_WRITES);
        let mut writes = [vk::WriteDescriptorSet::default(); MAX_WRITES];
        let mut buffer_infos = [vk::DescriptorBufferInfo::default(); MAX_WRITES];
        let mut image_infos = [vk::DescriptorImageInfo::default(); MAX_WRITES];
        let mut write_index = 0usize;
        let mut cached = CachedSet {
            key: Vec::with_capacity(self.m_uboWrites.len()),
            set: vk::DescriptorSet::null(),
            pool: None,
        };

        for uniform in self.m_uboWrites.iter() {
            assert!(!uniform.buffer.is_null());
            let vk_buffer = unsafe { (&*uniform.buffer).current() };
            cached.key.push(vk_buffer);
            buffer_infos[write_index] = vk::DescriptorBufferInfo {
                buffer: vk_buffer,
                offset: u64::from(uniform.offset),
                range: u64::from(uniform.range),
            };
            writes[write_index] = vk::WriteDescriptorSet::default()
                .dst_set(alloc.set)
                .dst_binding(uniform.dstBinding)
                .dst_array_element(0)
                .descriptor_count(1)
                .descriptor_type(uniform.r#type);
            writes[write_index].p_buffer_info = &buffer_infos[write_index];
            write_index += 1;
        }
        for image in self.m_imageWrites.iter() {
            image_infos[write_index] = vk::DescriptorImageInfo {
                sampler: image.sampler,
                image_view: image.imageView,
                image_layout: image.imageLayout,
            };
            writes[write_index] = vk::WriteDescriptorSet::default()
                .dst_set(alloc.set)
                .dst_binding(image.dstBinding)
                .dst_array_element(0)
                .descriptor_count(1)
                .descriptor_type(image.r#type);
            writes[write_index].p_image_info = &image_infos[write_index];
            write_index += 1;
        }

        if write_index > 0 {
            unsafe {
                self.context()
                    .m_vk
                    .m_ashDevice
                    .update_descriptor_sets(&writes[..write_index], &[])
            };
        }
        cached.set = alloc.set;
        cached.pool = alloc.pool;
        setCache.push(cached);
        setCache.last().unwrap().set
    }
}
