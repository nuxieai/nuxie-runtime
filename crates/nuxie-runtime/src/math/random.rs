use std::collections::VecDeque;
#[cfg(any(
    all(
        target_arch = "wasm32",
        target_os = "unknown",
        not(feature = "clock-seed")
    ),
    test
))]
use std::sync::atomic::AtomicU64;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, OnceLock};
use std::thread::ThreadId;

#[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
#[path = "random/native.rs"]
mod platform;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[path = "random/wasm.rs"]
mod platform;

#[cfg(any(all(target_arch = "wasm32", target_os = "unknown"), test))]
const EMSCRIPTEN_MUSL_MULTIPLIER: u64 = 6_364_136_223_846_793_005;
#[cfg(any(all(target_arch = "wasm32", target_os = "unknown"), test))]
const EMSCRIPTEN_MUSL_RAND_MAX: f32 = 2_147_483_647.0;

#[cfg(any(all(target_arch = "wasm32", target_os = "unknown"), test))]
fn emscripten_musl_seed(platform_seed: &mut u64, seed: u32) {
    // Emscripten's `s` is 32-bit unsigned, so `s - 1` wraps before the
    // result is widened into the 64-bit static state.
    *platform_seed = u64::from(seed.wrapping_sub(1));
}

#[cfg(any(all(target_arch = "wasm32", target_os = "unknown"), test))]
fn emscripten_musl_next(platform_seed: &mut u64) -> u32 {
    *platform_seed = EMSCRIPTEN_MUSL_MULTIPLIER
        .wrapping_mul(*platform_seed)
        .wrapping_add(1);
    (*platform_seed >> 33) as u32
}

#[cfg(any(all(target_arch = "wasm32", target_os = "unknown"), test))]
fn emscripten_musl_draw(platform_seed: &mut u64) -> f32 {
    emscripten_musl_next(platform_seed) as f32 / EMSCRIPTEN_MUSL_RAND_MAX
}

#[cfg(any(all(target_arch = "wasm32", target_os = "unknown"), test))]
fn emscripten_monotonic_millis_to_seed(milliseconds: f64) -> u32 {
    // Preserve Emscripten 3.1.61's left-associated JS expression exactly:
    // `Math.round(now * 1000 * 1000)`, followed by C++'s unsigned narrowing.
    // Reassociating this as `now * 1_000_000` can change the low bit.
    (milliseconds * 1_000.0 * 1_000.0).round() as u64 as u32
}

#[cfg(any(
    all(
        target_arch = "wasm32",
        target_os = "unknown",
        not(feature = "clock-seed")
    ),
    test
))]
fn deterministic_counter_seed(counter: &AtomicU64) -> u32 {
    // The counter is the fallback steady-clock duration count; narrow it the
    // same way pinned C++ narrows that count to unsigned int.
    counter.fetch_add(1, Ordering::Relaxed) as u32
}

/// Safe process-global translation of pinned C++ `RandomProvider`.
///
/// C++ owns one static provider backed by the platform C `rand()` and, under
/// `TESTING`, one counted FIFO whose exhausted fallback is zero
/// (`include/rive/math/random.hpp:13-51`; `src/math/random.cpp:8-15`).
/// Rust preserves that process-global call/order boundary. A mutex replaces
/// C++'s data race without changing serialized behavior. Native targets call
/// their C runtime; the libc-less browser target reproduces the exact pinned
/// Emscripten C-runtime algorithm in its target adapter.
pub(crate) struct RuntimeRandomProvider;

static DETERMINISTIC_MODE: AtomicBool = AtomicBool::new(false);

#[derive(Debug, Default)]
struct RuntimeRandomProviderState {
    test_values: Option<RuntimeRandomTestValues>,
    total_calls: usize,
    platform_seed: u64,
}

#[derive(Debug)]
struct RuntimeRandomTestValues {
    owner: ThreadId,
    values: VecDeque<f32>,
    calls: usize,
}

/// Scoped, thread-owned installation of the pinned-C++ test FIFO.
///
/// C++ differentials run their probe in an isolated process. Rust's test
/// binary runs cases concurrently, so the harness associates the temporary
/// FIFO with the installing thread and clears it on drop. Production random
/// ownership remains process-global; this scope only prevents one oracle test
/// from poisoning another.
#[doc(hidden)]
pub struct RuntimeRandomTestValuesGuard {
    owner: ThreadId,
    _scope: std::sync::MutexGuard<'static, ()>,
}

impl Drop for RuntimeRandomTestValuesGuard {
    fn drop(&mut self) {
        let mut state = lock_state();
        if state
            .test_values
            .as_ref()
            .is_some_and(|values| values.owner == self.owner)
        {
            state.test_values = None;
        }
    }
}

fn state() -> &'static Mutex<RuntimeRandomProviderState> {
    static STATE: OnceLock<Mutex<RuntimeRandomProviderState>> = OnceLock::new();
    STATE.get_or_init(|| Mutex::new(RuntimeRandomProviderState::default()))
}

fn test_scope() -> &'static Mutex<()> {
    static TEST_SCOPE: OnceLock<Mutex<()>> = OnceLock::new();
    TEST_SCOPE.get_or_init(|| Mutex::new(()))
}

fn lock_state() -> std::sync::MutexGuard<'static, RuntimeRandomProviderState> {
    state()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

impl RuntimeRandomProvider {
    /// Mirror the per-layer `srand` in
    /// `StateMachineLayerInstance::init`
    /// (`state_machine_instance.cpp:150-167`).
    pub(crate) fn initialize_layer() {
        let seed = Self::layer_seed();
        let mut state = lock_state();
        platform::seed(&mut state.platform_seed, seed);
    }

    fn layer_seed() -> u32 {
        Self::layer_seed_for(DETERMINISTIC_MODE.load(Ordering::SeqCst))
    }

    fn layer_seed_for(deterministic: bool) -> u32 {
        if deterministic {
            1
        } else {
            platform::nondeterministic_seed()
        }
    }

    pub(crate) fn generate_random_float() -> f32 {
        let mut state = lock_state();
        let current = std::thread::current().id();
        if let Some(values) = state
            .test_values
            .as_mut()
            .filter(|values| values.owner == current)
        {
            values.calls = values.calls.saturating_add(1);
            return values.values.pop_front().unwrap_or(0.0);
        }

        state.total_calls = state.total_calls.saturating_add(1);

        platform::draw(&mut state.platform_seed)
    }

    pub(crate) fn set_test_values(values: &[f32]) -> RuntimeRandomTestValuesGuard {
        let scope = test_scope()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let owner = std::thread::current().id();
        let mut state = lock_state();
        state.test_values = Some(RuntimeRandomTestValues {
            owner,
            values: values.iter().copied().collect(),
            calls: 0,
        });
        state.total_calls = 0;
        RuntimeRandomTestValuesGuard {
            owner,
            _scope: scope,
        }
    }

    pub(crate) fn total_calls() -> usize {
        let state = lock_state();
        let current = std::thread::current().id();
        state
            .test_values
            .as_ref()
            .filter(|values| values.owner == current)
            .map_or(state.total_calls, |values| values.calls)
    }
}

/// Set the process-global deterministic mode used by runtime facilities.
///
/// Pinned C++ exposes `File::deterministicMode`; every state-machine layer
/// initialization seeds the global C random provider with `1` when enabled
/// (`include/rive/file.hpp:87-89`;
/// `src/animation/state_machine_instance.cpp:150-167`). The ordinary Rust
/// runtime defaults to `false`; oracle runners enable it before constructing
/// occurrences, just like the C++ golden runner.
#[doc(hidden)]
pub fn set_runtime_deterministic_mode(enabled: bool) {
    DETERMINISTIC_MODE.store(enabled, Ordering::SeqCst);
}

/// Install the process-global counted FIFO used by pinned-C++ random
/// differentials. Values are consumed in order and exhaustion returns zero.
#[doc(hidden)]
pub fn set_runtime_random_test_values(values: &[f32]) -> RuntimeRandomTestValuesGuard {
    RuntimeRandomProvider::set_test_values(values)
}

/// Return the process-global random-provider call count.
#[doc(hidden)]
pub fn runtime_random_call_count() -> usize {
    RuntimeRandomProvider::total_calls()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::AtomicU64;

    use super::{
        RuntimeRandomProvider, deterministic_counter_seed, emscripten_monotonic_millis_to_seed,
        emscripten_musl_draw, emscripten_musl_next, emscripten_musl_seed,
    };

    #[test]
    fn self_contained_browser_seed_counter_produces_distinct_seeds() {
        let counter = AtomicU64::new(0);

        assert_eq!(deterministic_counter_seed(&counter), 0);
        assert_eq!(deterministic_counter_seed(&counter), 1);
    }

    #[test]
    fn self_contained_browser_seed_counter_uses_cpp_unsigned_narrowing() {
        let counter = AtomicU64::new(u64::from(u32::MAX));

        assert_eq!(deterministic_counter_seed(&counter), u32::MAX);
        assert_eq!(deterministic_counter_seed(&counter), 0);
    }

    #[test]
    fn injected_values_are_counted_and_scoped() {
        let values = RuntimeRandomProvider::set_test_values(&[0.75, 0.25]);

        assert_eq!(RuntimeRandomProvider::generate_random_float(), 0.75);
        assert_eq!(RuntimeRandomProvider::generate_random_float(), 0.25);
        assert_eq!(RuntimeRandomProvider::generate_random_float(), 0.0);
        assert_eq!(RuntimeRandomProvider::total_calls(), 3);

        drop(values);
    }

    #[test]
    fn deterministic_layer_seed_matches_cpp_contract() {
        assert_eq!(RuntimeRandomProvider::layer_seed_for(true), 1);

        // Hold the provider lock across both layer-initialization sequences so
        // a parallel test cannot consume the process-global C stream between
        // `srand` and `rand`. These are the exact seed/draw helpers used by
        // `initialize_layer` and `generate_random_float`.
        let mut state = super::lock_state();
        super::platform::seed(
            &mut state.platform_seed,
            RuntimeRandomProvider::layer_seed_for(true),
        );
        let first = super::platform::draw(&mut state.platform_seed);
        super::platform::seed(
            &mut state.platform_seed,
            RuntimeRandomProvider::layer_seed_for(true),
        );
        let second = super::platform::draw(&mut state.platform_seed);
        assert_eq!(first, second, "each deterministic layer reseeds C rand");
    }

    #[test]
    fn browser_provider_matches_pinned_emscripten_musl_sequence() {
        // Pinned C++'s `build_rive.sh` selects Emscripten 3.1.61. Its
        // `system/lib/libc/musl/src/prng/rand.c` stores `seed = s - 1`, then
        // applies this wrapping 64-bit LCG and returns `seed >> 33`.
        let mut seed = 0;
        emscripten_musl_seed(&mut seed, 1);
        assert_eq!(emscripten_musl_draw(&mut seed), 0.0);
        assert_eq!(
            [
                emscripten_musl_next(&mut seed),
                emscripten_musl_next(&mut seed),
                emscripten_musl_next(&mut seed),
            ],
            [740_882_966, 1_616_430_695, 1_708_849_955]
        );

        emscripten_musl_seed(&mut seed, u32::MAX);
        assert_eq!(
            [
                emscripten_musl_next(&mut seed),
                emscripten_musl_next(&mut seed),
            ],
            [1_308_150_633, 1_150_367_849]
        );

        emscripten_musl_seed(&mut seed, 0);
        assert_eq!(seed, u64::from(u32::MAX));
        assert_ne!(
            seed,
            u64::from(0_u32).wrapping_sub(1),
            "subtracting after widening is the rejected translation"
        );
        assert_eq!(
            [
                emscripten_musl_next(&mut seed),
                emscripten_musl_next(&mut seed),
            ],
            [2_049_033_599, 2_025_915_578],
            "`unsigned s - 1` must wrap before widening to the u64 state"
        );

        let discriminator_millis = 955_096.428_917_5;
        let exact_seed = emscripten_monotonic_millis_to_seed(discriminator_millis);
        assert_eq!(
            exact_seed, 1_613_689_205,
            "Emscripten multiplies milliseconds by 1000 twice before rounding"
        );
        assert_ne!(
            exact_seed,
            (discriminator_millis * 1_000_000.0).round() as u64 as u32,
            "reassociating the two Emscripten multiplications is observable"
        );
    }
}
