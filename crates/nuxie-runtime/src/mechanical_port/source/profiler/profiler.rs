#[cfg(feature = "rive_microprofile")]
pub const MICROPROFILE_IMPLEMENTATION_ENABLED: bool = true;
#[cfg(not(feature = "rive_microprofile"))]
pub const MICROPROFILE_IMPLEMENTATION_ENABLED: bool = false;
