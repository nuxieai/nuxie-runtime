use std::collections::VecDeque;
use std::sync::{LazyLock, Mutex, MutexGuard};

pub struct RandomProvider;

#[derive(Default)]
struct TestingRandomState {
    calls: i32,
    results: VecDeque<f32>,
}

#[derive(Default)]
struct RandomState {
    testing: Option<TestingRandomState>,
    wasm_seed: u64,
}

static RANDOM_STATE: LazyLock<Mutex<RandomState>> =
    LazyLock::new(|| Mutex::new(RandomState::default()));

impl RandomProvider {
    /// Enter the pinned `TESTING` FIFO mode and append a deterministic draw.
    pub fn add_random_value(value: f32) {
        random_state()
            .testing
            .get_or_insert_with(TestingRandomState::default)
            .results
            .push_back(value);
    }

    /// Match pinned `clearRandoms`: remain in test mode with an empty FIFO.
    pub fn clear_randoms() {
        random_state().testing = Some(TestingRandomState::default());
    }

    /// Leave the downstream runtime test adaptation and resume platform draws.
    pub fn clear_testing_mode() {
        random_state().testing = None;
    }

    /// Seed the same process-global generator used by `generate_random_float`.
    pub fn seed(seed: u32) {
        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            // Emscripten's musl `srand` stores the wrapping 32-bit `seed - 1`.
            random_state().wasm_seed = u64::from(seed.wrapping_sub(1));
        }

        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        native_seed(seed);
    }

    pub fn layer_seed(deterministic: bool) -> u32 {
        if deterministic {
            1
        } else {
            nondeterministic_seed()
        }
    }

    pub fn generate_random_float() -> f32 {
        let mut state = random_state();
        if let Some(testing) = &mut state.testing {
            testing.calls += 1;
            return testing.results.pop_front().unwrap_or(0.0);
        }

        #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
        {
            state.wasm_seed = 6_364_136_223_846_793_005_u64
                .wrapping_mul(state.wasm_seed)
                .wrapping_add(1);
            return (state.wasm_seed >> 33) as u32 as f32 / 2_147_483_647.0;
        }

        #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
        {
            drop(state);
            native_draw()
        }
    }

    pub fn total_calls() -> i32 {
        random_state()
            .testing
            .as_ref()
            .map_or(0, |testing| testing.calls)
    }
}

fn random_state() -> MutexGuard<'static, RandomState> {
    RANDOM_STATE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

#[cfg(all(
    not(all(target_arch = "wasm32", target_os = "unknown")),
    target_os = "android"
))]
unsafe extern "C" {
    fn srand(seed: libc::c_uint);
    fn rand() -> libc::c_int;
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
fn native_seed(seed: u32) {
    #[cfg(not(target_os = "android"))]
    // SAFETY: this calls the same process-global C runtime function as pinned C++.
    unsafe {
        libc::srand(seed)
    };
    #[cfg(target_os = "android")]
    // SAFETY: bionic exports `srand` with this standard C signature.
    unsafe {
        srand(seed)
    };
}

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
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
    not(all(target_arch = "wasm32", target_os = "unknown")),
    not(any(target_os = "android", target_os = "wasi"))
))]
fn platform_rand_max() -> f32 {
    libc::RAND_MAX as f32
}

#[cfg(all(
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
