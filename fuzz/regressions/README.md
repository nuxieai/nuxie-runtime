# Fuzz regressions

Committed reproducers for findings from the fuzz targets.

## Layout

- `fuzz/regressions/<target>/` — reproducers for **fixed** bugs. `make
  fuzz-regressions` replays these with `-runs=0` (just execute the inputs) and
  must exit cleanly. Add a reproducer here in the same commit that lands the
  fix.
- `fuzz/regressions/open/` — reproducers for **known-open** findings that still
  crash or hang. These are archived here so the exact input is not lost, but
  they are **not** replayed by `make fuzz-smoke` or `make fuzz-regressions`
  (that would wedge the gate). Move a file up into
  `fuzz/regressions/<target>/` when its finding is fixed.

## Known-open finding: unbounded chain-walk HANGs on malformed input (not panics)

One finding class, several reachable sites. In every case `read_runtime_file`
**accepts** the file and the pipeline then enters an infinite loop (a HANG, not
a panic) while walking a parent/reference chain whose links a malformed file has
made cyclic. No cycle guard bounds the walk.

Reproducers under `open/`:

- `fuzz_runtime-hang-layout-parent-cycle-min.riv` (39 bytes, minimized) and
  `fuzz_runtime-hang-layout-parent-cycle-orig.riv` (581 bytes, original) —
  hang in `crates/rive-runtime/src/components.rs`
  `runtime_layout_chain_has_layout_component` (and its sibling
  `runtime_constrained_layout_ancestor`), reached from
  `ArtboardInstance::from_graph_with_artboards`. The
  `while let Some(...) { local_id = parent_local; }` layout-parent walk never
  terminates on a `parentId` cycle.
- `fuzz_runtime-hang-drawrules-refchain.riv` (297 bytes) — hang in
  `crates/rive-graph/src/lib.rs` `flattened_draw_rules_local` /
  `runtime_object_for_local` / `object_parent_id`, reached from
  `GraphFile::from_runtime_file` during draw-order computation. A cyclic
  draw-rule / object reference chain loops forever.

C++ parity: the reference `rive-runtime` **also hangs** on the original input
(confirmed with the C++ golden runner — it spins in `Artboard::initialize` ->
`Path::onAddedClean`'s unbounded shape-parent walk). `Component::validate` only
checks that a parent resolves to a `ContainerComponent`; it does not reject
parent cycles. So this is shared behavior with upstream C++, not a Rust-only
regression — which is why it is left UNFIXED here pending a maintainer decision
(see the session report).

Reproduce (any file above), from repo root:

    cd fuzz && rustup run nightly cargo fuzz run fuzz_runtime \
        regressions/open/fuzz_runtime-hang-layout-parent-cycle-min.riv -- -timeout=10

libFuzzer reports a timeout at ~10s.

Proposed fix (needs sign-off — it makes Rust diverge from C++ by terminating
where C++ hangs; it cannot change any valid file's output, so golden-compare
stays 263/584): give each parent/reference-chain walk a visited-id set or an
iteration cap, mirroring the cycle-guard idiom C++ already uses elsewhere
(`DependencySorter` cycle detection, `src/dependency_sorter.cpp`;
`Artboard::validateObjects` `for (int cycle = 0; cycle < 100; cycle++)`,
`src/artboard.cpp`). This is a small cross-crate hardening pass (rive-graph +
rive-runtime), not a one-line guard, which is why it is reported rather than
applied in this lane.
