//! Worklet processor kernel using the shipping Rust/Luau implementation.
#[cfg(target_arch = "wasm32")]
use nuxie_scripting::vm::luau_audio_web as kernel;
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn nuxie_audio_alloc(length: usize) -> *mut u8 {
    kernel::alloc(length)
}
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nuxie_audio_free(pointer: *mut u8, length: usize) {
    unsafe { kernel::free(pointer, length) }
}
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nuxie_audio_request(pointer: *const u8, length: usize) -> *const u8 {
    unsafe { kernel::kernel_request(pointer, length) }
}
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn nuxie_audio_response_len() -> usize {
    kernel::response_len()
}
#[cfg(target_arch = "wasm32")]
#[unsafe(no_mangle)]
pub extern "C" fn nuxie_audio_render(frames: u32, low: u32, high: u32) -> *const f32 {
    kernel::render(frames, low, high)
}
