#[cfg(feature = "testing")]
use std::collections::VecDeque;
#[cfg(feature = "testing")]
use std::sync::{LazyLock, Mutex};

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

impl RandomProvider {
    #[cfg(feature = "testing")]
    pub fn add_random_value(value: f32) {
        TESTING_RANDOM_STATE
            .lock()
            .unwrap()
            .results
            .push_back(value);
    }

    #[cfg(feature = "testing")]
    pub fn clear_randoms() {
        *TESTING_RANDOM_STATE.lock().unwrap() = TestingRandomState::default();
    }

    #[cfg(feature = "testing")]
    pub fn generate_random_float() -> f32 {
        let mut state = TESTING_RANDOM_STATE.lock().unwrap();
        state.calls += 1;
        state.results.pop_front().unwrap_or(0.0)
    }

    #[cfg(feature = "testing")]
    pub fn total_calls() -> i32 {
        TESTING_RANDOM_STATE.lock().unwrap().calls
    }

    #[cfg(not(feature = "testing"))]
    pub fn generate_random_float() -> f32 {
        unsafe extern "C" {
            fn rand() -> core::ffi::c_int;
        }
        // C RAND_MAX is required to preserve the upstream provider's range.
        const RAND_MAX: f32 = 2_147_483_647.0;
        unsafe { rand() as f32 / RAND_MAX }
    }
}
