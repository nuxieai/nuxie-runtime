#[cfg(any(
    feature = "testing",
    all(target_arch = "wasm32", target_os = "unknown")
))]
use std::sync::{LazyLock, Mutex};

#[cfg(feature = "testing")]
use std::collections::VecDeque;

pub struct RandomProvider;

#[cfg(feature = "testing")]
#[derive(Default)]
struct TestingRandomState {
    calls: i32,
    results: VecDeque<f32>,
}

#[cfg(feature = "testing")]
static TESTING_RANDOM_STATE: LazyLock<Mutex<TestingRandomState>> =
    LazyLock::new(|| Mutex::new(TestingRandomState::default()));

#[cfg(all(
    not(feature = "testing"),
    target_arch = "wasm32",
    target_os = "unknown"
))]
static WASM_RANDOM_STATE: LazyLock<Mutex<u64>> = LazyLock::new(|| Mutex::new(0));

impl RandomProvider {
    #[cfg(feature = "testing")]
    pub fn add_random_value(value: f32) {
        testing_state().results.push_back(value);
    }

    #[cfg(feature = "testing")]
    pub fn clear_randoms() {
        *testing_state() = TestingRandomState::default();
    }

    /// Seed the same process-global generator used by `generate_random_float`.
    pub fn seed(seed: u32) {
        #[cfg(feature = "testing")]
        let _ = seed;

        #[cfg(all(
            not(feature = "testing"),
            target_arch = "wasm32",
            target_os = "unknown"
        ))]
        {
            // Emscripten's musl `srand` stores the wrapping 32-bit `seed - 1`.
            *wasm_random_state() = u64::from(seed.wrapping_sub(1));
        }

        #[cfg(all(
            not(feature = "testing"),
            not(all(target_arch = "wasm32", target_os = "unknown"))
        ))]
        native_seed(seed);
    }

    pub fn layer_seed(deterministic: bool) -> u32 {
        if deterministic {
            1
        } else {
            nondeterministic_seed()
        }
    }

    #[cfg(feature = "testing")]
    pub fn generate_random_float() -> f32 {
        let mut state = testing_state();
        state.calls += 1;
        state.results.pop_front().unwrap_or(0.0)
    }

    #[cfg(all(
        not(feature = "testing"),
        target_arch = "wasm32",
        target_os = "unknown"
    ))]
    pub fn generate_random_float() -> f32 {
        let mut state = wasm_random_state();
        *state = 6_364_136_223_846_793_005_u64
            .wrapping_mul(*state)
            .wrapping_add(1);
        (*state >> 33) as u32 as f32 / 2_147_483_647.0
    }

    #[cfg(all(
        not(feature = "testing"),
        not(all(target_arch = "wasm32", target_os = "unknown"))
    ))]
    pub fn generate_random_float() -> f32 {
        native_draw()
    }

    #[cfg(feature = "testing")]
    pub fn total_calls() -> i32 {
        testing_state().calls
    }
}

#[cfg(feature = "testing")]
fn testing_state() -> std::sync::MutexGuard<'static, TestingRandomState> {
    TESTING_RANDOM_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(all(
    not(feature = "testing"),
    target_arch = "wasm32",
    target_os = "unknown"
))]
fn wasm_random_state() -> std::sync::MutexGuard<'static, u64> {
    WASM_RANDOM_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(all(
    not(feature = "testing"),
    not(all(target_arch = "wasm32", target_os = "unknown")),
    target_os = "android"
))]
unsafe extern "C" {
    fn srand(seed: libc::c_uint);
    fn rand() -> libc::c_int;
}

#[cfg(all(
    not(feature = "testing"),
    not(all(target_arch = "wasm32", target_os = "unknown"))
))]
fn native_seed(seed: u32) {
    #[cfg(not(target_os = "android"))]
    // SAFETY: this calls the same process-global C runtime function as pinned
    // C++; runtime use serializes layer initialization and random draws.
    unsafe {
        libc::srand(seed)
    };
    #[cfg(target_os = "android")]
    // SAFETY: bionic exports `srand` with this standard C signature.
    unsafe {
        srand(seed)
    };
}

#[cfg(all(
    not(feature = "testing"),
    not(all(target_arch = "wasm32", target_os = "unknown"))
))]
fn native_draw() -> f32 {
    #[cfg(not(target_os = "android"))]
    // SAFETY: `rand` takes no arguments and returns a C `int`.
    unsafe {
        libc::rand() as f32 / platform_rand_max()
    }
    #[cfg(target_os = "android")]
    // SAFETY: bionic exports `rand` with this standard C signature.
    unsafe {
        rand() as f32 / platform_rand_max()
    }
}

#[cfg(all(
    not(feature = "testing"),
    not(all(target_arch = "wasm32", target_os = "unknown")),
    not(any(target_os = "android", target_os = "wasi"))
))]
fn platform_rand_max() -> f32 {
    libc::RAND_MAX as f32
}

#[cfg(all(
    not(feature = "testing"),
    not(all(target_arch = "wasm32", target_os = "unknown")),
    any(target_os = "android", target_os = "wasi")
))]
fn platform_rand_max() -> f32 {
    2_147_483_647.0
}

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    feature = "js-host-seed"
))]
fn nondeterministic_seed() -> u32 {
    use wasm_bindgen::{JsCast, JsValue};

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
    (milliseconds * 1_000.0 * 1_000.0).round() as u64 as u32
}

#[cfg(all(
    target_arch = "wasm32",
    target_os = "unknown",
    not(feature = "js-host-seed")
))]
fn nondeterministic_seed() -> u32 {
    1
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn nondeterministic_seed() -> u32 {
    high_resolution_nanoseconds().map_or(1, |value| value as u32)
}

#[cfg(any(unix, target_os = "wasi"))]
fn high_resolution_nanoseconds() -> Option<i128> {
    let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
    #[cfg(target_vendor = "apple")]
    let clock_id = libc::CLOCK_MONOTONIC_RAW;
    #[cfg(all(not(target_vendor = "apple"), target_os = "linux"))]
    let clock_id = libc::CLOCK_REALTIME;
    #[cfg(all(not(target_vendor = "apple"), not(target_os = "linux")))]
    let clock_id = libc::CLOCK_MONOTONIC;

    // SAFETY: `value` points to writable `timespec` storage.
    let status = unsafe { libc::clock_gettime(clock_id, value.as_mut_ptr()) };
    if status != 0 {
        return None;
    }
    // SAFETY: successful `clock_gettime` initialized both fields.
    let value = unsafe { value.assume_init() };
    Some(i128::from(value.tv_sec) * 1_000_000_000 + i128::from(value.tv_nsec))
}

#[cfg(windows)]
fn high_resolution_nanoseconds() -> Option<i128> {
    use windows_sys::Win32::System::Performance::{
        QueryPerformanceCounter, QueryPerformanceFrequency,
    };

    let mut frequency = 0;
    // SAFETY: `frequency` is writable storage for the Windows API.
    let status = unsafe { QueryPerformanceFrequency(&mut frequency) };
    if status == 0 || frequency <= 0 {
        return None;
    }
    let mut counter = 0;
    // SAFETY: `counter` is writable storage for the Windows API.
    let status = unsafe { QueryPerformanceCounter(&mut counter) };
    if status == 0 {
        return None;
    }
    let seconds = counter / frequency;
    let fractions = counter % frequency;
    Some(
        i128::from(seconds) * 1_000_000_000
            + i128::from(fractions) * 1_000_000_000 / i128::from(frequency),
    )
}

#[cfg(not(any(unix, windows, target_os = "wasi")))]
fn high_resolution_nanoseconds() -> Option<i128> {
    None
}
