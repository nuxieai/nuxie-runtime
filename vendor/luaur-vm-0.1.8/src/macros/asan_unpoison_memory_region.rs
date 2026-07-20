#[allow(non_snake_case)]
macro_rules! ASAN_UNPOISON_MEMORY_REGION {
    ($addr:expr, $size:expr) => {
        // AddressSanitizer is not supported in standard Rust without unstable features or external C links.
        // In a portable context, this macro is a no-op unless the build environment provides __asan_unpoison_memory_region.
        ()
    };
}

pub(crate) use ASAN_UNPOISON_MEMORY_REGION;
