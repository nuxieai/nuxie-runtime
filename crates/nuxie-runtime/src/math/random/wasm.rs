use wasm_bindgen::{JsCast, JsValue};

use super::{emscripten_monotonic_millis_to_seed, emscripten_musl_draw, emscripten_musl_seed};

pub(super) fn seed(platform_seed: &mut u64, seed: u32) {
    emscripten_musl_seed(platform_seed, seed);
}

pub(super) fn draw(platform_seed: &mut u64) -> f32 {
    emscripten_musl_draw(platform_seed)
}

pub(super) fn nondeterministic_seed() -> u32 {
    // Pinned C++'s browser build uses Emscripten 3.1.61. libc++ aliases
    // high_resolution_clock to steady_clock, whose CLOCK_MONOTONIC shim
    // rounds performance.now() milliseconds to nanoseconds before C++ narrows
    // the count to unsigned int.
    let global = js_sys::global();
    let performance = js_sys::Reflect::get(&global, &JsValue::from_str("performance"))
        .unwrap_or_else(|error| {
            wasm_bindgen::throw_val(error);
        });
    let now = js_sys::Reflect::get(&performance, &JsValue::from_str("now"))
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
        .dyn_into::<js_sys::Function>()
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
    let milliseconds = now
        .call0(&performance)
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
        .as_f64()
        .unwrap_or_else(|| wasm_bindgen::throw_str("performance.now() did not return a number"));

    emscripten_monotonic_millis_to_seed(milliseconds)
}
