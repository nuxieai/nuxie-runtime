# Reproduce the runner PGO experiment

`runner_pgo.py` builds an ordinary control and a profile-guided candidate from one frozen Git revision. It reuses the campaign's four-fixture training flow and validates both binaries against all 25 recording fixtures. It does not change production build defaults or measure a speedup.

Requirements: Python 3.12+, a native Rust toolchain, Cargo dependencies already cached, explicit C/C++ compilers, and `llvm-profdata` from the same LLVM major version as `rustc -vV`. Matching the full LLVM version is preferable; both versions are recorded. This is a native-host experiment because the instrumented runner must execute on the build machine.

The committed manifest pins the 25 asset hashes from C++ revision `160085c654874d35ad654a750e782d7c78e050d9`. Supply that checkout's asset directory explicitly. The assets are copied into the experiment after their hashes are verified; they are not distributed with this tool.

For the audited Apple Silicon build:

```sh
cargo fetch --locked
python3 tools/perf-gate/runner_pgo.py \
  --repo . --revision HEAD \
  --assets "$RIVE_RUNTIME_DIR/tests/unit_tests/assets" \
  --manifest tools/perf-gate/pgo-fixtures.json \
  --output target/runner-pgo-experiment \
  --target aarch64-apple-darwin \
  --cc /usr/bin/clang --cxx /usr/bin/clang++ \
  --llvm-profdata llvm-profdata --jobs 2
```

`--revision` selects committed source, excluding working edits and untracked files. Commit the intended candidate before using it. All three build lanes use that same archive. The output directory must be new, so profiles from an earlier run cannot contaminate training. `--check-only` freezes and verifies inputs without building; use a separate new output directory for the subsequent full experiment.

Apple Clang is named explicitly above because adding another LLVM installation to `PATH` can otherwise silently change the compiler used for native dependencies. Other native hosts need their own target and explicit native compilers; only the Apple Silicon configuration has been exercised in the original experiment.

The script retains the repository's release profile: fat LTO, one codegen unit, unwind. It pins the actual Rust compiler and Cargo executables from the selected sysroot before entering the archive, so a rustup proxy cannot switch compilers because of an archived toolchain file. It records and checks compiler/LLVM executable hashes. It clears inherited Rust flags, wrappers, profile and incremental-build overrides, native compiler flags, target overrides, and profiling output variables. It passes each lane's intended Rust flags through `CARGO_ENCODED_RUSTFLAGS`, preserving paths containing spaces. Build-related Cargo configuration (`build`, `target`, `profile`, or `env`) is rejected so it cannot silently override this experiment; other discovered Cargo configurations are recorded and checked for changes.

The sequence is fixed:

1. Build the ordinary control with no extra Rust flags.
2. Build with `-Cprofile-generate`, then train **only** `zombie_skins`, `spotify_kids_demo`, `car_widgets_v01`, and `data_viz_demo`. Each process advances continuously through 10,000 samples at 60 Hz, with `--benchmark --execute-scripts`. State is not reset between samples.
3. Merge those raw profiles with `llvm-profdata merge --failure-mode=any`, then build with `-Cprofile-use`, missing-profile diagnostics, and indirect-call-promotion remarks.
4. Compare control and PGO on 100 progressive samples for each of the 25 fixtures, using ordinary recording plus `--execute-scripts --side-channel`. Success requires zero exits and exact stdout **and** stderr. The 21 held-out fixtures never train the profile.

Inspect `provenance.json`, `commands.json`, `source-manifest.json`, `fixture-manifest.json`, `validation.json`, and the build logs. `profile-diagnostics.json` separates missing-function warnings from profile hash mismatches, which fail the experiment. It records the total promotion remarks and their positions before or after the final runner invocation line. Parallel Cargo jobs can interleave compiler output: the preceding invocation is context, not proof of the remark's originating crate or compilation phase. These counts must not be described as final fat-LTO runner promotions without independent attribution. If no final invocation is observed, the positional counts are null. Missing-function warnings can occur for untrained code; they are retained for review, not treated as proof that the profile is stale. Source and Cargo configuration hashes are checked before and after each build.

The resulting `binaries/control` and `binaries/pgo` are candidates for an independent timing comparison on the same quiet machine. Keep the held-out results separate from the four trained fixtures. Training and recording timings are not valid performance measurements. A passing recording comparison does not establish general parity, and gains can change with compiler, CPU, source, or content. Retrain for each source/toolchain change; this tool intentionally has no profile-reuse or deployment mode.

The profile trains the runner's NullRenderer advance workload. It does not train a product's GPU renderer, native embedding, or every script/audio path. Shipping a product build with PGO requires a representative workload for that product, another correctness comparison, and a separate measured benefit.

Run the orchestration tests without compiling Rust:

```sh
python3 tools/perf-gate/test_runner_pgo.py
```
