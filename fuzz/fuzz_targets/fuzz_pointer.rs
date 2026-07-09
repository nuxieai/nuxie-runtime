#![no_main]

//! Like `fuzz_runtime`, but also replays a few derived pointer events against
//! the default state machine between advances, exercising hit-testing and
//! listener actions.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    rive_fuzz::run_pointer(data);
});
