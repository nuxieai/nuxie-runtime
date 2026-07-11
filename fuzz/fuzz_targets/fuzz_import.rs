#![no_main]

//! Parser-hardening target: raw bytes -> `nuxie_binary::read_runtime_file`.
//! The importer must never panic on arbitrary input.

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    nuxie_fuzz::run_import(data);
});
