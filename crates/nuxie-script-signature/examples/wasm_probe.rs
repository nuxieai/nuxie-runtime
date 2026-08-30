//! Raw test-only ABI for invoking the Rust-selected WASM verifier from Node.

#[unsafe(no_mangle)]
pub extern "C" fn signature_probe_alloc(length: usize) -> *mut u8 {
    Box::into_raw(vec![0u8; length.max(1)].into_boxed_slice()) as *mut u8
}

/// # Safety
/// Pointer and length must come from `signature_probe_alloc`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn signature_probe_free(pointer: *mut u8, length: usize) {
    let slice = std::ptr::slice_from_raw_parts_mut(pointer, length.max(1));
    // SAFETY: guaranteed by the test ABI caller.
    unsafe {
        drop(Box::from_raw(slice));
    }
}

/// # Safety
/// All inputs must point into allocated WASM memory for their declared sizes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn signature_probe_verify(
    signature: *const u8,
    message: *const u8,
    message_length: usize,
    context: *const u8,
    public_key: *const u8,
) -> u32 {
    // SAFETY: guaranteed by the test ABI caller; inputs are immutable borrows.
    let (signature, message, context, public_key) = unsafe {
        (
            &*signature.cast::<[u8; nuxie_script_signature::SIGNATURE_BYTES]>(),
            std::slice::from_raw_parts(message, message_length),
            &*context.cast::<[u8; nuxie_script_signature::CONTEXT_BYTES]>(),
            &*public_key.cast::<[u8; nuxie_script_signature::PUBLIC_KEY_BYTES]>(),
        )
    };
    nuxie_script_signature::verify(signature, message, context, public_key).into()
}
