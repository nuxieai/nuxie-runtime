# Luaur integration probes

This harness uses the workspace's patched Luaur crates.

## Protected errors without unwinding

Run the same error/recovery probe in both profiles:

```sh
cargo run -p luaur-rt-test-harness --example protected_error_recovery
CARGO_PROFILE_DEV_PANIC=abort cargo run -p luaur-rt-test-harness --example protected_error_recovery
```

Both commands must exit successfully. The probe checks error-object identity,
nested protected calls, host callback errors, error handlers (including failing
handlers), arithmetic/indexing failures, coroutine yield/error, malformed source,
and successful execution after repeated failures in the same VM.

This executable is intentional: stable Rust's ordinary test harness requires
unwinding, so a passing unit test cannot demonstrate containment in an aborting
build. This probe is not a replacement for browser/Worker qualification.

Initial evidence on runtime `df893fde9b4f0094021deda873e85a42129a34cb`:
the default profile passes all three repetitions; the aborting profile prints
`initial execution: passed` and exits 134 at the first guest error. This is the
regression for [UNIV-1645](https://universe.basis.dev/issue/UNIV-1645).
