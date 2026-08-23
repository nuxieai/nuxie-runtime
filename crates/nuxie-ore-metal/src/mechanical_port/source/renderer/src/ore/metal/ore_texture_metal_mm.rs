/*
 * Copyright 2025 Rive
 */

// #include "ore_texture_metal.hpp"
// #include "rive/rive_types.hpp"

// #import <Metal/Metal.h>

// Mechanical translation of the complete pinned source implementation
// renderer/src/ore/metal/ore_texture_metal.mm.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![cfg(target_vendor = "apple")]
use super::*;

#[cfg(target_vendor = "apple")]
use core::ffi::c_void;
#[cfg(target_vendor = "apple")]
use std::ptr::NonNull;

#[cfg(target_vendor = "apple")]
use objc2_metal::{MTLOrigin, MTLRegion, MTLSize, MTLTexture};

// namespace rive::ore

impl TextureMetal {
    // void TextureMetal::upload(const TextureDataDesc& data)
    pub fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError> {
        debug_assert!(self.m_mtlTexture.is_some());
        debug_assert!(data.data.is_some());

        let texture = self
            .m_mtlTexture
            .as_ref()
            .ok_or(TextureUploadError::MissingNativeTexture)?;
        let bytes = data.data.ok_or(TextureUploadError::NullData)?;
        if bytes.is_empty() {
            return Err(TextureUploadError::EmptyData);
        }

        let region = MTLRegion {
            origin: MTLOrigin {
                x: data.x as usize,
                y: data.y as usize,
                z: data.z as usize,
            },
            size: MTLSize {
                width: data.width as usize,
                height: data.height as usize,
                depth: data.depth as usize,
            },
        };

        // Apple's `replaceRegion:` docs require `bytesPerImage = 0` for
        // non-array 2D textures (Metal API Validation aborts on any other
        // value). For texture3D / array2D the value is the per-slice stride.
        let row_bytes = data.bytesPerRow as usize;
        let mtlBytesPerImage: usize = if self.base.r#type() == TextureType::texture3D
            || self.base.r#type() == TextureType::array2D
        {
            row_bytes
                .checked_mul(data.rowsPerImage as usize)
                .ok_or(TextureUploadError::SizeOverflow)?
        } else {
            0
        };

        // C++ receives only a raw pointer, but Metal reads the complete region
        // synchronously. Validate the borrowed Rust span before crossing that
        // unsafe native boundary so it cannot authorize an out-of-provenance
        // read.
        let required = match self.base.r#type() {
            TextureType::texture3D => mtlBytesPerImage
                .checked_mul(data.depth as usize)
                .ok_or(TextureUploadError::SizeOverflow)?,
            TextureType::array2D => mtlBytesPerImage,
            TextureType::texture2D | TextureType::cube => row_bytes
                .checked_mul(data.height as usize)
                .ok_or(TextureUploadError::SizeOverflow)?,
        };
        if bytes.len() < required {
            return Err(TextureUploadError::DataTooShort {
                required,
                actual: bytes.len(),
            });
        }

        let bytes = NonNull::new(bytes.as_ptr().cast_mut().cast::<c_void>())
            .expect("TextureDataDesc data must be non-null");
        unsafe {
            texture.replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage(
                region,
                data.mipLevel as usize,
                data.layer as usize,
                bytes,
                data.bytesPerRow as usize,
                mtlBytesPerImage,
            );
        }
        Ok(())
    }
}

impl TextureApi for TextureMetal {
    fn width(&self) -> u32 {
        self.base.width()
    }

    fn height(&self) -> u32 {
        self.base.height()
    }

    fn depthOrArrayLayers(&self) -> u32 {
        self.base.depthOrArrayLayers()
    }

    fn format(&self) -> TextureFormat {
        self.base.format()
    }

    fn r#type(&self) -> TextureType {
        self.base.r#type()
    }

    fn numMipmaps(&self) -> u32 {
        self.base.numMipmaps()
    }

    fn sampleCount(&self) -> u32 {
        self.base.sampleCount()
    }

    fn isRenderTarget(&self) -> bool {
        self.base.isRenderTarget()
    }

    fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError> {
        TextureMetal::upload(self, data)
    }
}

// } // namespace rive::ore
