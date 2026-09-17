//! Platform media players. Decoding and audio clocks belong to the platform;
//! scene composition belongs to the runtime's persistent renderer factory.
#[cfg(target_os = "android")]
pub mod android;
#[cfg(any(
    target_os = "macos",
    target_os = "ios",
    target_os = "tvos",
    target_os = "visionos"
))]
pub mod apple;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod browser;

#[cfg(not(target_arch = "wasm32"))]
pub mod source;

pub mod scene;

pub mod pool;

pub mod sync;

#[cfg(any(target_os = "android", test))]
mod audio;
