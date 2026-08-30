//! The pinned libhydrogen verifier used by Rive's `RiveCode` signatures.
//!
//! Native consumers retain the original libhydrogen backend. Host-free wasm
//! links the same C verification functions, without the unrelated platform RNG
//! and key-generation translation units.

pub const SIGNATURE_BYTES: usize = 64;
pub const PUBLIC_KEY_BYTES: usize = 32;
pub const CONTEXT_BYTES: usize = 8;

/// Verify exactly the supplied libhydrogen signature and context.
/// Length validation and signed-content admission remain the caller's job.
pub fn verify(
    signature: &[u8; SIGNATURE_BYTES],
    message: &[u8],
    context: &[u8; CONTEXT_BYTES],
    public_key: &[u8; PUBLIC_KEY_BYTES],
) -> bool {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    {
        libhydrogen::sign::verify(
            &libhydrogen::sign::Signature::from(*signature),
            message,
            &libhydrogen::sign::Context::from(*context),
            &libhydrogen::sign::PublicKey::from(*public_key),
        )
        .is_ok()
    }
    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    {
        freestanding_verify(signature, message, context, public_key)
    }
}

#[cfg(any(
    all(target_arch = "wasm32", target_os = "unknown"),
    all(test, feature = "freestanding-tests")
))]
fn freestanding_verify(
    signature: &[u8; SIGNATURE_BYTES],
    message: &[u8],
    context: &[u8; CONTEXT_BYTES],
    public_key: &[u8; PUBLIC_KEY_BYTES],
) -> bool {
    unsafe extern "C" {
        fn nuxie_script_hydro_sign_verify(
            signature: *const u8,
            message: *const u8,
            message_len: usize,
            context: *const u8,
            public_key: *const u8,
        ) -> i32;
    }
    // SAFETY: all fixed-size inputs have the sizes required by hydrogen.h;
    // message is borrowed for the call, including a valid pointer at len zero.
    unsafe {
        nuxie_script_hydro_sign_verify(
            signature.as_ptr(),
            message.as_ptr(),
            message.len(),
            context.as_ptr(),
            public_key.as_ptr(),
        ) == 0
    }
}

#[cfg(all(test, feature = "freestanding-tests", not(target_arch = "wasm32")))]
mod tests;
