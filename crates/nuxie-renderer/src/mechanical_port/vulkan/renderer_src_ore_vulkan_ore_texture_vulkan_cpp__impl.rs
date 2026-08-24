//! Complete mechanical implementation translation of
//! `renderer/src/ore/vulkan/ore_texture_vulkan.cpp`.

#![allow(non_snake_case)]

use super::ore_buffer_vulkan_decl::BufferVulkan;
use super::ore_context_vulkan_decl::VkPendingTextureUpload;
use super::ore_texture_vulkan_decl::{TextureViewVulkan, TextureVulkan};
use ash::vk;
use nuxie_ore_metal::buffer::BufferApi;
use nuxie_ore_metal::gpu_resource::{AnyResourceHandle, GpuResourcePayload, ResourceHandle};
use nuxie_ore_metal::texture::TextureUploadError;
use nuxie_ore_metal::types::{
    textureFormatBytesPerTexel, BufferUsage, TextureDataDesc, TextureFormat, TextureType,
};
use std::mem::ManuallyDrop;
use vk_mem::{Alloc, AllocationCreateFlags, AllocationCreateInfo, MemoryUsage};

fn isDepthStencilFormat(format: TextureFormat) -> bool {
    matches!(
        format,
        TextureFormat::depth16unorm
            | TextureFormat::depth24plusStencil8
            | TextureFormat::depth32float
            | TextureFormat::depth32floatStencil8
    )
}

fn hasStencil(format: TextureFormat) -> bool {
    matches!(
        format,
        TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
    )
}

pub(crate) fn aspectMask(format: TextureFormat) -> vk::ImageAspectFlags {
    if isDepthStencilFormat(format) {
        let mut flags = vk::ImageAspectFlags::DEPTH;
        if hasStencil(format) {
            flags |= vk::ImageAspectFlags::STENCIL;
        }
        flags
    } else {
        vk::ImageAspectFlags::COLOR
    }
}

fn fail(
    texture: &TextureVulkan,
    message: String,
    error: TextureUploadError,
) -> Result<(), TextureUploadError> {
    texture.oreContextMut().setLastError(message);
    Err(error)
}

pub(crate) fn upload(
    texture: &TextureVulkan,
    data: &TextureDataDesc<'_>,
    owner: Option<AnyResourceHandle>,
) -> Result<(), TextureUploadError> {
    assert!(!texture.m_vkOreContext.get().is_null());
    if texture.m_vkImage == vk::Image::null() {
        return fail(
            texture,
            "upload: native image is null".into(),
            TextureUploadError::MissingNativeTexture,
        );
    }
    let Some(bytes) = data.data else {
        return fail(
            texture,
            "upload: data is null".into(),
            TextureUploadError::NullData,
        );
    };
    let Some(owner) = owner else {
        return fail(
            texture,
            "upload: source texture retain is unavailable".into(),
            TextureUploadError::WrongResourceKind,
        );
    };

    let bytes_per_texel = textureFormatBytesPerTexel(texture.format());
    if data.mipLevel >= texture.numMipmaps() {
        return fail(
            texture,
            format!(
                "upload: mipLevel ({}) >= numMipmaps ({})",
                data.mipLevel,
                texture.numMipmaps()
            ),
            TextureUploadError::SizeOverflow,
        );
    }
    if data.layer >= texture.depthOrArrayLayers() {
        return fail(
            texture,
            format!(
                "upload: layer ({}) >= depthOrArrayLayers ({})",
                data.layer,
                texture.depthOrArrayLayers()
            ),
            TextureUploadError::SizeOverflow,
        );
    }
    let mip_width = (texture.width() >> data.mipLevel).max(1);
    let mip_height = (texture.height() >> data.mipLevel).max(1);
    let width = if data.width > 0 {
        data.width
    } else {
        mip_width
    };
    let height = if data.height > 0 {
        data.height
    } else {
        mip_height
    };
    let max_depth = if texture.r#type() == TextureType::texture3D {
        texture.depthOrArrayLayers()
    } else {
        1
    };
    let depth = if data.depth > 0 {
        data.depth
    } else {
        max_depth
    };
    if u64::from(data.x) + u64::from(width) > u64::from(mip_width)
        || u64::from(data.y) + u64::from(height) > u64::from(mip_height)
    {
        return fail(
            texture,
            format!(
                "upload: region (x={} y={} w={} h={}) out of bounds for mip {} ({}x{})",
                data.x, data.y, width, height, data.mipLevel, mip_width, mip_height
            ),
            TextureUploadError::SizeOverflow,
        );
    }
    if u64::from(data.z) + u64::from(depth) > u64::from(max_depth) {
        return fail(
            texture,
            format!(
                "upload: z-region (z={} depth={}) out of bounds (maxDepth={})",
                data.z, depth, max_depth
            ),
            TextureUploadError::SizeOverflow,
        );
    }
    if bytes_per_texel == 0 {
        return fail(
            texture,
            "upload: block-compressed formats not yet supported".into(),
            TextureUploadError::SizeOverflow,
        );
    }
    if data.bytesPerRow != 0 && data.bytesPerRow % bytes_per_texel != 0 {
        return fail(
            texture,
            format!(
                "upload: bytesPerRow ({}) must be a whole number of texels (bytesPerTexel={})",
                data.bytesPerRow, bytes_per_texel
            ),
            TextureUploadError::SizeOverflow,
        );
    }
    if data.bytesPerRow != 0
        && u64::from(data.bytesPerRow) < u64::from(width) * u64::from(bytes_per_texel)
    {
        return fail(
            texture,
            format!(
                "upload: bytesPerRow ({}) < width * bytesPerTexel ({})",
                data.bytesPerRow,
                u64::from(width) * u64::from(bytes_per_texel)
            ),
            TextureUploadError::SizeOverflow,
        );
    }
    if data.rowsPerImage > 0 && data.rowsPerImage < height {
        return fail(
            texture,
            format!(
                "upload: rowsPerImage ({}) < height ({})",
                data.rowsPerImage, height
            ),
            TextureUploadError::SizeOverflow,
        );
    }
    let bytes_per_row = if data.bytesPerRow != 0 {
        u64::from(data.bytesPerRow)
    } else {
        u64::from(width) * u64::from(bytes_per_texel)
    };
    let rows_per_image = if data.rowsPerImage > 0 {
        data.rowsPerImage
    } else {
        height
    };
    let upload_size = bytes_per_row
        .checked_mul(u64::from(rows_per_image))
        .and_then(|value| value.checked_mul(u64::from(depth)))
        .unwrap_or(u64::MAX);
    if upload_size > u64::from(u32::MAX) {
        return fail(
            texture,
            format!("upload: size ({upload_size}) exceeds uint32_t staging buffer max"),
            TextureUploadError::SizeOverflow,
        );
    }
    let required = upload_size as usize;
    if bytes.len() < required {
        return fail(
            texture,
            format!(
                "upload: data too short (required={} actual={})",
                required,
                bytes.len()
            ),
            TextureUploadError::DataTooShort {
                required,
                actual: bytes.len(),
            },
        );
    }

    let manager = texture
        .base
        .gpu_resource()
        .manager()
        .expect("TextureVulkan requires its source manager")
        .clone();
    let vk_context = texture
        .m_vk
        .as_ref()
        .expect("TextureVulkan requires its retained VulkanContext")
        .clone();
    let mut staging = BufferVulkan::new(manager.clone(), required as u32, BufferUsage::upload);
    unsafe {
        // The staging buffer retains this exact context and its device; its
        // backing below is allocated from the same context's VMA owner.
        staging.setVulkanContext(vk_context.clone());
        staging.setDeviceAndUsage(vk_context.device, vk::BufferUsageFlags::TRANSFER_SRC);
    }
    let buffer_info = vk::BufferCreateInfo::default()
        .size(upload_size)
        .usage(vk::BufferUsageFlags::TRANSFER_SRC);
    let allocation_info = AllocationCreateInfo {
        flags: AllocationCreateFlags::MAPPED,
        #[allow(deprecated)]
        usage: MemoryUsage::CpuOnly,
        ..Default::default()
    };
    let (buffer, allocation) = match unsafe {
        vk_context
            .allocator()
            .create_buffer(&buffer_info, &allocation_info)
    } {
        Ok(value) => value,
        Err(error) => {
            return fail(
                texture,
                format!(
                    "upload: staging buffer allocation failed (size={}, vk={})",
                    upload_size,
                    error.as_raw()
                ),
                TextureUploadError::SizeOverflow,
            );
        }
    };
    let mapped = vk_context
        .allocator()
        .get_allocation_info(&allocation)
        .mapped_data
        .cast::<u8>();
    if mapped.is_null() {
        let mut allocation = allocation;
        unsafe {
            vk_context
                .allocator()
                .destroy_buffer(buffer, &mut allocation)
        };
        return fail(
            texture,
            format!(
                "upload: staging buffer allocation failed (size={}, vk={})",
                upload_size,
                vk::Result::ERROR_MEMORY_MAP_FAILED.as_raw()
            ),
            TextureUploadError::SizeOverflow,
        );
    }
    unsafe {
        // The native buffer, allocation, and mapped range are the tuple just
        // returned by `vk_context.allocator()`.
        staging.installStagingBacking(buffer, allocation, mapped);
    }
    staging
        .update(bytes, required as u32, 0)
        .map_err(|_| TextureUploadError::SizeOverflow)?;
    let staging = ResourceHandle::new_buffer(Some(manager), staging).erase();
    let region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: if data.bytesPerRow != 0 {
            data.bytesPerRow / bytes_per_texel
        } else {
            0
        },
        buffer_image_height: data.rowsPerImage,
        image_subresource: vk::ImageSubresourceLayers {
            aspect_mask: aspectMask(texture.format()),
            mip_level: data.mipLevel,
            base_array_layer: data.layer,
            layer_count: 1,
        },
        image_offset: vk::Offset3D {
            x: data.x as i32,
            y: data.y as i32,
            z: data.z as i32,
        },
        image_extent: vk::Extent3D {
            width,
            height,
            depth,
        },
    };
    texture
        .oreContextMut()
        .vkQueuePendingTextureUpload(VkPendingTextureUpload {
            texture: owner,
            stagingBuffer: staging,
            region,
            aspectMask: aspectMask(texture.format()),
        });
    Ok(())
}

impl Drop for TextureVulkan {
    fn drop(&mut self) {
        if self.m_vkImage != vk::Image::null() {
            if let Some(mut allocation) = self.m_vmaAllocation.take() {
                let vk_context = self
                    .m_vk
                    .as_ref()
                    .expect("owned TextureVulkan image requires VulkanContext");
                unsafe {
                    vk_context
                        .allocator()
                        .destroy_image(self.m_vkImage, &mut allocation)
                };
            }
        }
        unsafe {
            ManuallyDrop::drop(&mut self.m_vk);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Drop for TextureViewVulkan {
    fn drop(&mut self) {
        if self.m_vkImageView != vk::ImageView::null() {
            if let Some(destroy) = self.m_vkDestroyImageView {
                unsafe { destroy(self.m_vkDevice, self.m_vkImageView, core::ptr::null()) };
            }
        }
        unsafe { ManuallyDrop::drop(&mut self.base) };
    }
}
