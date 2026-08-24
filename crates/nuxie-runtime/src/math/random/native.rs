// The Rust libc crate does not expose bionic's `srand`/`rand`/`RAND_MAX`
// bindings for Android even though bionic provides the functions; bind them
// directly so Android matches the same process-global C generator every
// other native target uses.
#[cfg(target_os = "android")]
unsafe extern "C" {
    fn srand(seed: libc::c_uint);
    fn rand() -> libc::c_int;
}

pub(super) fn seed(_platform_seed: &mut u64, seed: u32) {
    // SAFETY: C++ calls the same process-global C function. The caller holds
    // the runtime provider mutex across every `srand` and `rand`.
    #[cfg(not(target_os = "android"))]
    unsafe {
        libc::srand(seed)
    };
    #[cfg(target_os = "android")]
    unsafe {
        srand(seed)
    };
}

pub(super) fn draw(_platform_seed: &mut u64) -> f32 {
    // SAFETY: C++ delegates to the same platform C function. The caller holds
    // the runtime provider mutex across every `srand` and `rand`.
    #[cfg(not(target_os = "android"))]
    unsafe {
        libc::rand() as f32 / platform_rand_max()
    }
    #[cfg(target_os = "android")]
    unsafe {
        rand() as f32 / platform_rand_max()
    }
}

#[cfg(not(any(target_os = "wasi", target_os = "android")))]
fn platform_rand_max() -> f32 {
    libc::RAND_MAX as f32
}

#[cfg(target_os = "android")]
fn platform_rand_max() -> f32 {
    // Bionic's stdlib.h defines RAND_MAX as 0x7fffffff; the Rust libc crate
    // does not expose the macro for Android.
    2_147_483_647.0
}

#[cfg(target_os = "wasi")]
fn platform_rand_max() -> f32 {
    // wasi-libc's stdlib.h defines RAND_MAX as 0x7fffffff, but the Rust libc
    // crate does not currently expose that macro for WASI.
    2_147_483_647.0
}

pub(super) fn nondeterministic_seed() -> u32 {
    // A valid target high_resolution_clock source is an operating-system
    // invariant on every supported native target. If that invariant fails, do
    // not unwind through an embedder/FFI boundary; fall back to C++'s
    // deterministic seed.
    seed_from_high_resolution_nanoseconds(high_resolution_nanoseconds())
}

fn seed_from_high_resolution_nanoseconds(nanoseconds: Option<i128>) -> u32 {
    nanoseconds.map_or(1, |value| value as u32)
}

#[cfg(any(unix, target_os = "wasi"))]
fn high_resolution_nanoseconds() -> Option<i128> {
    let mut value = std::mem::MaybeUninit::<libc::timespec>::uninit();
    #[cfg(target_vendor = "apple")]
    let clock_id = APPLE_HIGH_RESOLUTION_CLOCK_ID;
    // Pinned Rive's ordinary Linux build selects clang without `-stdlib`;
    // clang therefore uses libstdc++, whose high_resolution_clock aliases
    // system_clock. CLOCK_REALTIME is that platform's direct clock source.
    #[cfg(all(not(target_vendor = "apple"), target_os = "linux"))]
    let clock_id = LINUX_HIGH_RESOLUTION_CLOCK_ID;
    // Android NDK, WASI libc++, and the remaining supported Unix libc++
    // toolchains alias high_resolution_clock to steady_clock.
    #[cfg(all(not(target_vendor = "apple"), not(target_os = "linux")))]
    let clock_id = OTHER_UNIX_HIGH_RESOLUTION_CLOCK_ID;

    // SAFETY: `value` points to writable `timespec` storage. A successful
    // `clock_gettime` initializes both fields before `assume_init`.
    let status = unsafe { libc::clock_gettime(clock_id, value.as_mut_ptr()) };
    if status != 0 {
        return None;
    }
    // SAFETY: guarded by the successful `clock_gettime` result above.
    let value = unsafe { value.assume_init() };
    Some(i128::from(value.tv_sec) * 1_000_000_000 + i128::from(value.tv_nsec))
}

#[cfg(all(any(unix, target_os = "wasi"), target_vendor = "apple"))]
const APPLE_HIGH_RESOLUTION_CLOCK_ID: libc::clockid_t = libc::CLOCK_MONOTONIC_RAW;

#[cfg(all(unix, not(target_vendor = "apple"), target_os = "linux"))]
const LINUX_HIGH_RESOLUTION_CLOCK_ID: libc::clockid_t = libc::CLOCK_REALTIME;

#[cfg(all(
    any(unix, target_os = "wasi"),
    not(target_vendor = "apple"),
    not(target_os = "linux")
))]
const OTHER_UNIX_HIGH_RESOLUTION_CLOCK_ID: libc::clockid_t = libc::CLOCK_MONOTONIC;

#[cfg(windows)]
fn high_resolution_nanoseconds() -> Option<i128> {
    use std::sync::OnceLock;
    use windows_sys::Win32::System::Performance::{
        QueryPerformanceCounter, QueryPerformanceFrequency,
    };

    static FREQUENCY: OnceLock<Option<i64>> = OnceLock::new();
    let frequency = (*FREQUENCY.get_or_init(|| {
        let mut value = 0;
        // SAFETY: `value` is valid writable storage for the Windows API.
        let status = unsafe { QueryPerformanceFrequency(&mut value) };
        (status != 0 && value > 0).then_some(value)
    }))?;
    let mut counter = 0;
    // SAFETY: `counter` is valid writable storage for the Windows API.
    let status = unsafe { QueryPerformanceCounter(&mut counter) };
    if status == 0 {
        return None;
    }

    // Match libc++ chrono.cpp's integer evaluation order exactly.
    let seconds = counter / frequency;
    let fractions = counter % frequency;
    Some(
        i128::from(seconds) * 1_000_000_000
            + i128::from(fractions) * 1_000_000_000 / i128::from(frequency),
    )
}

#[cfg(not(any(unix, windows, target_os = "wasi")))]
compile_error!("RandomProvider needs a pinned high_resolution_clock adapter for this target");

#[cfg(test)]
mod tests {
    use super::seed_from_high_resolution_nanoseconds;

    #[test]
    fn clock_failure_uses_non_panicking_deterministic_seed() {
        assert_eq!(seed_from_high_resolution_nanoseconds(None), 1);
        assert_eq!(
            seed_from_high_resolution_nanoseconds(Some(i128::from(u32::MAX) + 2)),
            1
        );
    }
}
