//! Texture-backed polyfill for vertex-stage storage buffers.
//!
//! This mirrors Rive's `StorageTextureBufferWebGPU`: shaders address a fixed
//! 128-texel-wide texture and each logical storage-buffer binding is copied to
//! the texture starting at texel zero.

use crate::work_metrics::CountedDeviceExt;
use std::num::NonZeroU64;

pub(crate) const STORAGE_TEXTURE_WIDTH: u32 = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StorageBufferStructure {
    Uint32x4,
    Uint32x2,
    Float32x4,
}

impl StorageBufferStructure {
    const fn element_size(self) -> u64 {
        match self {
            Self::Uint32x4 | Self::Float32x4 => 16,
            Self::Uint32x2 => 8,
        }
    }

    const fn format(self) -> wgpu::TextureFormat {
        match self {
            Self::Uint32x4 => wgpu::TextureFormat::Rgba32Uint,
            Self::Uint32x2 => wgpu::TextureFormat::Rg32Uint,
            Self::Float32x4 => wgpu::TextureFormat::Rgba32Float,
        }
    }

    const fn sample_type(self) -> wgpu::TextureSampleType {
        match self {
            Self::Uint32x4 | Self::Uint32x2 => wgpu::TextureSampleType::Uint,
            Self::Float32x4 => wgpu::TextureSampleType::Float { filterable: false },
        }
    }
}

pub(crate) enum StorageResource {
    Buffer {
        buffer: wgpu::Buffer,
        offset: u64,
        size: NonZeroU64,
    },
    Texture {
        _texture: wgpu::Texture,
        view: wgpu::TextureView,
    },
}

impl StorageResource {
    pub(crate) fn upload<T: bytemuck::Pod>(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        label: &'static str,
        values: &[T],
        structure: StorageBufferStructure,
        polyfill: bool,
    ) -> Self {
        let bytes = bytemuck::cast_slice(values);
        assert!(!bytes.is_empty(), "storage upload must not be empty");
        let usage = if polyfill {
            wgpu::BufferUsages::COPY_SRC
        } else {
            wgpu::BufferUsages::STORAGE
        };
        let buffer = device.create_counted_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytes,
            usage,
        });
        Self::from_buffer(
            device,
            encoder,
            label,
            &buffer,
            0,
            NonZeroU64::new(bytes.len() as u64).expect("nonempty storage upload"),
            structure,
            polyfill,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn from_buffer(
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        label: &'static str,
        buffer: &wgpu::Buffer,
        offset: u64,
        size: NonZeroU64,
        structure: StorageBufferStructure,
        polyfill: bool,
    ) -> Self {
        if !polyfill {
            return Self::Buffer {
                buffer: buffer.clone(),
                offset,
                size,
            };
        }

        let element_size = structure.element_size();
        assert_eq!(size.get() % element_size, 0);
        assert_eq!(offset % wgpu::COPY_BUFFER_ALIGNMENT, 0);
        let element_count = size.get() / element_size;
        let (width, height) = storage_texture_size(element_count);
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: structure.format(),
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        // A buffer-to-texture copy has one width for every copied row. Split
        // off a partial final row so the source upload does not need the extra
        // worst-case row padding that Rive's persistent BufferRing reserves.
        let full_rows = element_count / u64::from(STORAGE_TEXTURE_WIDTH);
        let final_row_width = element_count % u64::from(STORAGE_TEXTURE_WIDTH);
        let bytes_per_full_row = u64::from(STORAGE_TEXTURE_WIDTH) * element_size;
        if full_rows != 0 {
            copy_rows(
                encoder,
                buffer,
                offset,
                &texture,
                0,
                STORAGE_TEXTURE_WIDTH,
                u32::try_from(full_rows).expect("storage texture height fits u32"),
                Some(
                    u32::try_from(bytes_per_full_row).expect("storage texture row pitch fits u32"),
                ),
            );
        }
        if final_row_width != 0 {
            copy_rows(
                encoder,
                buffer,
                offset + full_rows * bytes_per_full_row,
                &texture,
                u32::try_from(full_rows).expect("storage texture row fits u32"),
                u32::try_from(final_row_width).expect("storage texture width fits u32"),
                1,
                None,
            );
        }

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        Self::Texture {
            _texture: texture,
            view,
        }
    }

    pub(crate) fn binding(&self) -> wgpu::BindingResource<'_> {
        match self {
            Self::Buffer {
                buffer,
                offset,
                size,
            } => wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                buffer,
                offset: *offset,
                size: Some(*size),
            }),
            Self::Texture { view, .. } => wgpu::BindingResource::TextureView(view),
        }
    }
}

pub(crate) fn layout_entry(
    binding: u32,
    structure: StorageBufferStructure,
    visibility: wgpu::ShaderStages,
    polyfill: bool,
) -> wgpu::BindGroupLayoutEntry {
    let ty = if polyfill {
        wgpu::BindingType::Texture {
            sample_type: structure.sample_type(),
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    } else {
        wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: true },
            has_dynamic_offset: false,
            min_binding_size: None,
        }
    };
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty,
        count: None,
    }
}

fn storage_texture_size(element_count: u64) -> (u32, u32) {
    assert!(element_count != 0);
    let width = element_count.min(u64::from(STORAGE_TEXTURE_WIDTH));
    let height = element_count.div_ceil(u64::from(STORAGE_TEXTURE_WIDTH));
    (
        u32::try_from(width).expect("storage texture width fits u32"),
        u32::try_from(height).expect("storage texture height fits u32"),
    )
}

#[allow(clippy::too_many_arguments)]
fn copy_rows(
    encoder: &mut wgpu::CommandEncoder,
    buffer: &wgpu::Buffer,
    buffer_offset: u64,
    texture: &wgpu::Texture,
    texture_y: u32,
    width: u32,
    height: u32,
    bytes_per_row: Option<u32>,
) {
    encoder.copy_buffer_to_texture(
        wgpu::TexelCopyBufferInfo {
            buffer,
            layout: wgpu::TexelCopyBufferLayout {
                offset: buffer_offset,
                bytes_per_row,
                rows_per_image: None,
            },
        },
        wgpu::TexelCopyTextureInfo {
            texture,
            mip_level: 0,
            origin: wgpu::Origin3d {
                x: 0,
                y: texture_y,
                z: 0,
            },
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_geometry_matches_cpp_storage_texture_addressing() {
        assert_eq!(storage_texture_size(1), (1, 1));
        assert_eq!(storage_texture_size(128), (128, 1));
        assert_eq!(storage_texture_size(129), (128, 2));
        assert_eq!(storage_texture_size(256), (128, 2));
    }

    #[test]
    fn texture_formats_match_cpp_storage_buffer_structures() {
        assert_eq!(
            StorageBufferStructure::Uint32x4.format(),
            wgpu::TextureFormat::Rgba32Uint
        );
        assert_eq!(
            StorageBufferStructure::Uint32x2.format(),
            wgpu::TextureFormat::Rg32Uint
        );
        assert_eq!(
            StorageBufferStructure::Float32x4.format(),
            wgpu::TextureFormat::Rgba32Float
        );
    }
}
