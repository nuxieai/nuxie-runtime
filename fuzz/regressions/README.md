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

## FIXED: unbounded chain-walk HANGs on malformed input (not panics)

One finding class, several reachable sites. In every case `read_runtime_file`
**accepted** the file and the pipeline then entered an infinite loop (a HANG,
not a panic) while walking a parent/reference chain whose links a malformed file
had made cyclic. No cycle guard bounded the walk.

Reproducers (now replayed by `make fuzz-regressions` under `fuzz_runtime/`):

- `fuzz_runtime-hang-layout-parent-cycle-min.riv` (39 bytes, minimized) and
  `fuzz_runtime-hang-layout-parent-cycle-orig.riv` (581 bytes, original) —
  hung in `crates/rive-runtime/src/components.rs`
  `runtime_layout_chain_has_layout_component` (and its sibling
  `runtime_constrained_layout_ancestor`), reached from
  `ArtboardInstance::from_graph_with_artboards`. The
  `while let Some(...) { local_id = parent_local; }` layout-parent walk never
  terminated on a `parentId` cycle.
- `fuzz_runtime-hang-drawrules-refchain.riv` (297 bytes) — hung in
  `crates/rive-graph/src/lib.rs` `flattened_draw_rules_local` /
  `runtime_object_for_local` / `object_parent_id`, reached from
  `GraphFile::from_runtime_file` during draw-order computation. A cyclic
  draw-rule / object reference chain looped forever.

C++ parity: the reference `rive-runtime` **also hangs** on the original input
(confirmed with the C++ golden runner — it spins in `Artboard::initialize` ->
`Path::onAddedClean`'s unbounded shape-parent walk). `Component::validate` only
checks that a parent resolves to a `ContainerComponent`; it does not reject
parent cycles.

FIX (coordinator decision 2026-07-09, v2-status item 27): each parent/reference
walk now carries a visited-id set, mirroring C++'s own cycle-guard idiom
(`DependencySorter::visit`'s `m_Perm`/`m_Temp` visited sets,
`src/dependency_sorter.cpp`; cf. `Artboard::validateObjects`'s bounded
`for (int cycle = 0; cycle < 100; cycle++)`, `src/artboard.cpp`). This is a
DELIBERATE divergence: where C++ hangs, we terminate the walk gracefully
(treated as no-ancestor / no-rule). It is unreachable on any valid file, so
golden-compare is unchanged (263/584). See the code comments at each guard site.

Reproduce (any file above), from repo root:

    cd fuzz && rustup run nightly cargo fuzz run fuzz_runtime \
        regressions/fuzz_runtime/fuzz_runtime-hang-layout-parent-cycle-min.riv \
        -- -runs=0 -timeout=10

Post-fix this exits cleanly instead of timing out.
