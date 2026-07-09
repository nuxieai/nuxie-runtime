#![no_main]

//! The important target: bytes -> import; if import ACCEPTS, build the graph,
//! instantiate the default artboard, advance the default scene twice, and draw
//! through the null renderer. Any panic is a finding.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    rive_fuzz::run_runtime(data);
});
