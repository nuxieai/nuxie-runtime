use crate::mechanical_port::source::gpu_texture_format::GpuTextureFormat;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Ktx2HwSupport {
    pub bc: bool,
    pub astc: bool,
    pub etc2: bool,
}
impl Default for Ktx2HwSupport {
    fn default() -> Self {
        Self {
            bc: true,
            astc: true,
            etc2: true,
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ktx2DecodeResult {
    pub format: GpuTextureFormat,
    pub pixel_width: u32,
    pub pixel_height: u32,
    pub level_count: u32,
    pub block_width: u8,
    pub block_height: u8,
    pub srgb: bool,
    pub blocks: Vec<u8>,
    pub software_decoded: bool,
}

pub trait Ktx2Decoder {
    fn decode_ktx2(bytes: &[u8], out: &mut Ktx2DecodeResult, hw_support: Ktx2HwSupport) -> bool;
}
