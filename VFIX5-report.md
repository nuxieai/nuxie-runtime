# VFIX lane 5 — V12 closure report

Date: 2026-08-03

Branch: `levi/vfix-convergence`

Pinned C++ runtime: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Outcome

V12 (`db_health_tracker`) is closed and restored to `exact`. The three enrolled
samples complete in the Rust runner and match C++ for all three draw segments
and all three side-channel segments. The corpus-wide scripted comparison is
green at 362 entries, 327 exact rows, 1,069 exact segments, and 1,069
side-channel segments, with only the already-registered 27 divergences and
eight `not-yet` rows.

## Required bootstrap

The lane began with the requested command sequence:

```text
rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/
make fixtures
make cpp-probe
```

The fixture and probe steps completed against the pinned C++ commit.

## Diagnosis and port

Release profiling preceded source inspection. Host sampling tools were tried
first, but macOS task inspection was unavailable in the managed sandbox
(`xctrace` service/cache access, `sample`, LLDB debugserver, and `samply` task
inspection were denied). The release runner's built-in phase benchmark and
temporary environment-gated timing probes localized the CPU time to the first
`Artboard::updatePass`. All probes were removed before verification.

The pinned C++ implementation already establishes both relevant bounds:

- `StateMachineInstance::advanceAndApply` performs at most five outer passes,
  resets the DataContext and Artboard after every pass, and continues solely
  while Component dirt remains (`state_machine_instance.cpp:2649-2707`).
- `Artboard::updateComponents` retains its independent 100-step dependency
  bound (`artboard.cpp:1204-1238`). The V12 trace never reached that bound.

Rust already had the five-pass ceiling and per-pass resets. Its discrepancy was
an additional continuation term: any pending DataBind work—including permanent
polling members—kept the outer loop alive after Component dirt was clean. The
fix removes that Rust-only condition. A targeted regression proves that a
persisting bind cannot extend a clean component pass, while the existing test
continues to prove that real Component dirt receives exactly five passes.

The stricter runtime behavior exposed one facade regression: a later
`FlowSession` StateBatch could overwrite the prior frame's reset trigger before
its zero reached the retained ScriptInput target. The facade now flushes that
already-validated prior-frame binding state before the transaction candidate
adopts live ScriptInput targets and rehomes onto the mutated ViewModel graph.
This preserves repeated `0 -> 1 -> 0 -> 1` trigger edges without adding a
non-C++ settlement continuation condition.

Source commits:

- `252e5dad` — `runtime: stop settlement on clean component dirt`
- `6499b3ca` — `nuxie: flush reset binds before state batch`

Both commit messages cite the corresponding pinned upstream file and line
ranges.

## Timing evidence

The pre-fix release runner completed the full no-side V12 sample set in roughly
4 seconds and full side-channel runs in 9–10 seconds. After the fix, three
clean warmed release benchmark runs measured:

| run | elapsed | advance |
|---:|---:|---:|
| 1 | 3,295.57 ms | 2,937.84 ms |
| 2 | 3,251.89 ms | 2,890.41 ms |
| 3 | 3,244.98 ms | 2,879.75 ms |

Post-fix release side-channel runs remained 9–10 seconds with the same stream
size. The change removes clean passes and adds no work to the golden-runner
path, so the measured V12 release timing does not regress.

For the debug diagnosis, the historical capture timed out at 180.05 seconds
after consuming 173.40 seconds of user CPU. The fixed three-sample no-side
benchmark completes in 100.763 seconds (`advance=85.922s`, `prepare=14.833s`).
The provenance-verified debug side-channel runner also completes in both the
one-row exact comparison and the full corpus gate.

## Verification

- `cargo test -p nuxie-runtime` — pass (947 unit tests plus integration tests)
- `cargo test -p nuxie --features scripting` — pass (248 passed, one ignored,
  plus all integration suites)
- one-row scripted golden comparison — pass (3/3 exact draw segments, 3/3
  exact side-channel segments)
- `make scripted-golden-compare` — pass (362 entries; 327 exact; 1,069 exact
  segments; 1,069 side-channel segments; zero failures)
- `make runtime-frame-loop-port-check` — pass (112 checker tests,
  correspondence check, and live structural check)
- `make rust-attribution-check` — pass (10 tests and complete source
  attribution)
- `git diff --check` — pass

The first full scripted sweep encountered the repository's documented
nondeterministic pinned-C++ semantic-tree crash in
`data_binding_artboards_test`. Three isolated identical C++ runs returned
`0, 139, 139`; the required full rerun cleared that oracle flake and passed.
No corpus status or tolerance was changed to hide it.

## Repository annotations and cleanup

- `corpus.toml`: V12 changed from `not-yet` to `exact`; only the evidenced
  `post-zero-runtime-hang:V12` marker was removed.
- `docs/parity-gap-register.md`: V12 is marked closed with the exact comparison
  and bounded-loop evidence.
- `docs/v-row-triage.md` remains untracked and is intentionally excluded from
  every commit.
- Temporary profiling instrumentation and the one-row scratch manifest were
  removed. No `/tmp` path was used.
