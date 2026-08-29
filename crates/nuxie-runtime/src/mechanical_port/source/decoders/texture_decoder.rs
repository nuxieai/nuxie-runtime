//! Pinned `texture_decoder.cpp` configuration with no software decoder families
//! compiled in. Hardware-supported KTX2 blocks remain usable. Software-enabled
//! configurations require the real bc7decomp/rgbcx, astcenc, and ETCPACK adapters;
//! this owner does not advertise those configurations or synthesize pixels.

use super::bitmap_decoder::Bitmap;
use crate::mechanical_port::source::gpu_texture_format::GpuTextureFormat;

pub fn decode_texture(
    _blocks: &[u8],
    _width: u32,
    _height: u32,
    format: GpuTextureFormat,
    _block_width: u32,
    _block_height: u32,
) -> Option<Bitmap> {
    match format {
        GpuTextureFormat::Astc => {
            eprintln!("ASTC texture not supported (build with --with_rive_astc_decoder)");
        }
        GpuTextureFormat::Bc1
        | GpuTextureFormat::Bc2
        | GpuTextureFormat::Bc3
        | GpuTextureFormat::Bc7 => {
            eprintln!("BC texture not supported (build with --with_rive_bc_decoder)");
        }
        GpuTextureFormat::Etc2 => {
            eprintln!("ETC texture not supported (build with --with_rive_etc_decoder)");
        }
        _ => {
            eprintln!("decode_texture - unsupported format {}", format as u8);
        }
    }
    None
}
