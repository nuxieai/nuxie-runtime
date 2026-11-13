//! The browser audio host boundary, and the only `unsafe` in the audio code.
//!
//! Two directions cross here. The JS host's imports (wasm module
//! `nuxie_audio`) are called through safe wrappers that take slices, so a
//! caller cannot hand the host a pointer and length that disagree. The
//! kernel's exports (`tools/luau-audio-web/src/lib.rs`) hand JS a buffer and
//! read a request from it; those two functions take raw parts from JS and
//! are `unsafe fn`, with the contract JS must keep stated on each.
//!
//! Without the `browser-audio-host` feature (server-side wasm) the imports do
//! not exist and the wrappers are ordinary functions: every request fails
//! with a fixed response and the clock stays at zero.

#[cfg(feature = "browser-audio-host")]
#[link(wasm_import_module = "nuxie_audio")]
// SAFETY: these are the JS host's functions (`createImports` in
// `tools/luau-audio-web/shared.mjs`). Their declared types match the host's
// wasm signatures; each is called only through the wrapper below it, which
// states what the host may do with the memory it is given.
unsafe extern "C" {
    fn host_call(op: u32, input: *const u8, length: usize, output: *mut u8, capacity: usize)
    -> i32;
    fn now_ms() -> f64;
    fn shared_raw(
        op: u32,
        owner: u32,
        id: u32,
        bytes: *mut u8,
        length: usize,
        offset: u32,
        value: u32,
    ) -> f64;
}

/// Runs host operation `op` on `input` and writes the response into
/// `output`. Returns the response length, or minus the length the response
/// needs when `output` is shorter than that.
#[cfg(feature = "browser-audio-host")]
pub(super) fn call(op: u32, input: &[u8], output: &mut [u8]) -> i32 {
    // SAFETY: the host reads at most `input.len()` bytes from `input` and
    // writes at most `output.len()` bytes to `output`, during this call only,
    // and keeps neither address after it returns. Both slices are live and
    // `output` is not aliased for the call's duration.
    unsafe {
        host_call(
            op,
            input.as_ptr(),
            input.len(),
            output.as_mut_ptr(),
            output.len(),
        )
    }
}
/// Without the browser audio host, every request fails with this response.
#[cfg(not(feature = "browser-audio-host"))]
const NO_HOST_RESPONSE: &[u8] = br#"{"ok":false,"error":"audio needs the browser audio host"}"#;
#[cfg(not(feature = "browser-audio-host"))]
pub(super) fn call(_op: u32, _input: &[u8], output: &mut [u8]) -> i32 {
    // The response is a short constant, so its length fits in an i32.
    let length = NO_HOST_RESPONSE.len() as i32;
    let Some(output) = output.get_mut(..NO_HOST_RESPONSE.len()) else {
        return length.wrapping_neg();
    };
    output.copy_from_slice(NO_HOST_RESPONSE);
    length
}

/// Runs binary shared-transport operation `op` on `bytes`; negative on failure.
#[cfg(feature = "browser-audio-host")]
pub(super) fn shared(op: u32, owner: u32, id: u32, bytes: &mut [u8], offset: u32, value: u32) -> f64 {
    // SAFETY: the host reads and writes at most `bytes.len()` bytes at
    // `bytes`, during this call only, and keeps no address after it returns.
    unsafe {
        shared_raw(
            op,
            owner,
            id,
            bytes.as_mut_ptr(),
            bytes.len(),
            offset,
            value,
        )
    }
}
#[cfg(not(feature = "browser-audio-host"))]
pub(super) fn shared(op: u32, owner: u32, id: u32, bytes: &mut [u8], offset: u32, value: u32) -> f64 {
    let _ = (op, owner, id, bytes, offset, value);
    -1.0
}

/// The host clock in milliseconds (performance.now or Date.now).
#[cfg(feature = "browser-audio-host")]
pub(super) fn now() -> f64 {
    // SAFETY: takes and returns plain values; no memory is shared.
    unsafe { now_ms() }
}
#[cfg(not(feature = "browser-audio-host"))]
pub(super) fn now() -> f64 {
    0.0
}

/// Hands JS an owned, zeroed buffer of `length` bytes for one request, or
/// null when it cannot be allocated. JS owns it until it passes the same
/// address and length to `free`.
pub fn alloc(length: usize) -> *mut u8 {
    let mut buffer = Vec::new();
    if buffer.try_reserve_exact(length).is_err() {
        return std::ptr::null_mut();
    }
    buffer.resize(length, 0u8);
    Box::into_raw(buffer.into_boxed_slice()).cast::<u8>()
}
/// Takes back a buffer handed out by `alloc`.
///
/// # Safety
/// `pointer` and `length` must be exactly an address and length returned by
/// `alloc` (the length passed to it), not freed before, and JS must not use
/// the buffer after this call.
pub unsafe fn free(pointer: *mut u8, length: usize) {
    // SAFETY: by the contract above, these are the raw parts of a live
    // `Box<[u8]>` that `alloc` leaked, so rebuilding and dropping it frees
    // it once.
    drop(unsafe { Box::from_raw(std::ptr::slice_from_raw_parts_mut(pointer, length)) });
}
/// Runs one JSON kernel request read from `length` bytes at `pointer`; the
/// response is `response()`'s bytes, valid until the next request.
///
/// # Safety
/// `pointer` must address `length` initialized bytes in this instance's
/// memory (normally a buffer from `alloc`), and JS must not write them during
/// the call.
pub unsafe fn kernel_request(pointer: *const u8, length: usize) -> *const u8 {
    // SAFETY: by the contract above, the bytes are readable, initialized
    // and unchanged for this call, which copies what it keeps.
    let input = unsafe { std::slice::from_raw_parts(pointer, length) };
    super::run_kernel_request(input)
}
