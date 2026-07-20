#[cfg(all(target_arch = "x86_64", target_feature = "sse4.1"))]
pub const LUAU_TARGET_SSE41: bool = true;

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse4.1")))]
pub const LUAU_TARGET_SSE41: bool = false;
