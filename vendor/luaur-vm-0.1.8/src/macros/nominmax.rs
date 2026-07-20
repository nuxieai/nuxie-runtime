#[cfg(not(target_arch = "wasm32"))]
pub const NOMINMAX: () = ();
