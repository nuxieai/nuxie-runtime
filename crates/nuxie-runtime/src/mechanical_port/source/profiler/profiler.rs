#[cfg(feature = "rive_microprofile")]
pub const MICROPROFILE_IMPLEMENTATION_ENABLED: bool = true;
#[cfg(not(feature = "rive_microprofile"))]
pub const MICROPROFILE_IMPLEMENTATION_ENABLED: bool = false;

#[cfg(all(feature = "rive_microprofile", target_os = "windows"))]
pub const MICROPROFILE_GPU_TIMERS_D3D11: u32 = 1;

#[cfg(all(feature = "rive_microprofile", target_os = "windows"))]
pub const MICROPROFILE_GPU_TIMERS_D3D12: u32 = 1;

// The pinned Emscripten branch only surrounds the MicroProfile include with a
// Clang format-warning diagnostic push/pop; it has no Rust runtime behavior.
