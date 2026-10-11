//! Monotonic engine time, independent of playback speed and frame advances.

pub(crate) fn now_micros() -> i64 {
    #[cfg(test)]
    if let Some(clock) = TEST_CLOCK.with(std::cell::Cell::get) {
        return clock();
    }
    platform_now_micros()
}

#[cfg(test)]
thread_local! {
    static TEST_CLOCK: std::cell::Cell<Option<fn() -> i64>> = const { std::cell::Cell::new(None) };
}

#[cfg(test)]
pub(crate) fn with_test_clock<T>(clock: fn() -> i64, run: impl FnOnce() -> T) -> T {
    struct Restore(Option<fn() -> i64>);
    impl Drop for Restore {
        fn drop(&mut self) {
            TEST_CLOCK.with(|slot| slot.set(self.0));
        }
    }
    let _restore = Restore(TEST_CLOCK.with(|slot| slot.replace(Some(clock))));
    run()
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn platform_now_micros() -> i64 {
    // Only differences are consumed; an arbitrary steady epoch is equivalent
    // to C++ steady_clock::time_since_epoch().
    static EPOCH: std::sync::OnceLock<std::time::Instant> = std::sync::OnceLock::new();
    EPOCH
        .get_or_init(std::time::Instant::now)
        .elapsed()
        .as_micros() as i64
}

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    feature = "js-host-seed"
))]
fn platform_now_micros() -> i64 {
    use wasm_bindgen::{JsCast, JsValue};

    // Same browser/worker host boundary used by the existing random seed,
    // but retain integer microseconds rather than reducing to a seed.
    let global = js_sys::global();
    let performance = js_sys::Reflect::get(&global, &JsValue::from_str("performance"))
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
    let now = js_sys::Reflect::get(&performance, &JsValue::from_str("now"))
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
        .dyn_into::<js_sys::Function>()
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error));
    let milliseconds = now
        .call0(&performance)
        .unwrap_or_else(|error| wasm_bindgen::throw_val(error))
        .as_f64()
        .unwrap_or_else(|| wasm_bindgen::throw_str("performance.now() did not return a number"));
    (milliseconds * 1_000.0) as i64
}

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    not(feature = "js-host-seed")
))]
thread_local! {
    static HOST_CLOCK: std::cell::Cell<Option<fn() -> i64>> = const { std::cell::Cell::new(None) };
}

/// Install the monotonic microsecond clock for an import-free Wasm embedder.
///
/// Required before clock-driven runtime operations, including constructing a
/// watermarked artboard in deterministic mode. The
/// callback must use a consistent monotonic epoch for the entire lifetime of
/// every runtime instance. This explicit boundary keeps cold publisher modules free
/// of JS imports without silently disabling the playback clock budget.
#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    not(feature = "js-host-seed")
))]
pub fn set_clock(clock: fn() -> i64) {
    HOST_CLOCK.with(|host_clock| host_clock.set(Some(clock)));
}

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    not(feature = "js-host-seed")
))]
fn platform_now_micros() -> i64 {
    let clock = HOST_CLOCK.with(std::cell::Cell::get).expect(
        "import-free Wasm runtime requires engine_clock::set_clock before clock-driven operations",
    );
    clock()
}
