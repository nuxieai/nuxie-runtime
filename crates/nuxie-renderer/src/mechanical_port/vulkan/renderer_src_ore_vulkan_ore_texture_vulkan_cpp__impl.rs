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
    textureFormatBytesPerTexel, BufferUsage, TextureDataDesc, TextureFormat,
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

impl TextureVulkan {
    /// Marks the subresource written and reports whether it already was.
    pub(crate) fn vkMarkWritten(&self, mip: u32, layer: u32) -> bool {
        let index = layer as usize * self.numMipmaps() as usize + mip as usize;
        let mut written = self.m_vkWritten.borrow_mut();
        if index >= written.len() {
            written.resize(index + 1, false);
        }
        let was_written = written[index];
        written[index] = true;
        was_written
    }
}

pub(crate) fn uploadImpl(
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
    let bytes = data.data.expect("shared upload validation requires data");
    let Some(owner) = owner else {
        return fail(
            texture,
            "upload: source texture retain is unavailable".into(),
            TextureUploadError::WrongResourceKind,
        );
    };

    let bytes_per_texel = textureFormatBytesPerTexel(texture.format());
    if bytes_per_texel == 0 {
        texture
            .oreContextMut()
            .setLastError("upload: block-compressed formats not yet supported");
        // Upstream uploadImpl is void: backend diagnostics do not turn the
        // shared Texture::upload validation result into a failure.
        return Ok(());
    }
    let upload_size = u64::from(data.bytesPerRow)
        .checked_mul(u64::from(data.rowsPerImage))
        .and_then(|value| value.checked_mul(u64::from(data.depth)))
        .unwrap_or(u64::MAX);
    if upload_size > u64::from(u32::MAX) {
        texture.oreContextMut().setLastError(format!(
            "upload: size ({upload_size}) exceeds uint32_t staging buffer max"
        ));
        return Ok(());
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
            texture.oreContextMut().setLastError(format!(
                "upload: staging buffer allocation failed (size={}, vk={})",
                upload_size,
                error.as_raw()
            ));
            return Ok(());
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
        texture.oreContextMut().setLastError(format!(
            "upload: staging buffer allocation failed (size={}, vk={})",
            upload_size,
            vk::Result::SUCCESS.as_raw()
        ));
        return Ok(());
    }
    unsafe {
        // The native buffer, allocation, and mapped range are the tuple just
        // returned by `vk_context.allocator()`.
        staging.installStagingBacking(buffer, allocation, mapped);
    }
    staging
        .update(bytes, required as u32, 0)
        .map_err(|_| TextureUploadError::SizeOverflow)?;
    let domain = nuxie_ore_metal::context_backend_domain(&texture.oreContextMut().base);
    let staging =
        ResourceHandle::new_buffer_with_installed_manager_in_domain(domain, staging).erase();
    let region = vk::BufferImageCopy {
        buffer_offset: 0,
        buffer_row_length: data.bytesPerRow / bytes_per_texel,
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
            width: data.width,
            height: data.height,
            depth: data.depth,
        },
    };
    texture.vkMarkWritten(data.mipLevel, data.layer);
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
        if !self.m_vkRiveTexture.get().is_null() && !self.m_vkOreContext.get().is_null() {
            if let Some(registry) = self.m_riveWrappedRegistry.upgrade() {
                let this = std::ptr::NonNull::from(&mut *self);
                registry.borrow_mut().retain(|texture| *texture != this);
            }
        }
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
            ManuallyDrop::drop(&mut self.m_vkRiveTexture);
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
