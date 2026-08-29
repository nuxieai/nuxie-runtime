#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum GpuTextureFormat {
    Rgba32,
    Bc1,
    Bc2,
    Bc3,
    Bc7,
    Astc,
    Etc2,
}
