# B-6 structural fidelity audit summary

Audit source: pinned C++ `/Users/levi/dev/oss/rive-runtime` at
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`. All 448 manifest rows have an
on-disk record. The initial sweep records live under `results/`; the
post-RB-1/RD-1 closure decisions are in [SECOND_PASS.md](SECOND_PASS.md).
Row B6-0448 (`src/core/field_types/core_uint64_type.cpp`) was added on
2026-07-31 to repair the B-2 inventory omission at the pin; its record is a
post-audit amendment in [binary-core](results/binary-core.md).

## Post-audit additions (2026-08-04)

Rows **B6-0449..B6-0458** cover the ten upstream files that first appear
*after* the frozen audit ref. They do not exist at `d788e8ec`, so they were
audited against the live pin `4ac7b327` with the same five axes; C++ anchors
cite that pin and Rust anchors cite the current tree. Each cluster file carries
a "Post-audit additions (2026-08-04)" preamble stating that scope before its
records.

| row | upstream file | cluster | verdict |
|---|---|---|---|
| B6-0449 | `src/animation/keyframe_int.cpp` | animation | ADAPTED |
| B6-0450 | `src/component_origin.cpp` | misc-core | ADAPTED |
| B6-0451 | `src/core/field_types/core_int_type.cpp` | binary-core | ADAPTED |
| B6-0452 | `src/data_bind/context/context_value_asset_blob.cpp` | data-bind-view-model | ADAPTED |
| B6-0453 | `src/layout/grid_item_placement.cpp` | layout-shapes-paint | DIVERGENT |
| B6-0454 | `src/layout/grid_track.cpp` | layout-shapes-paint | DIVERGENT |
| B6-0455 | `src/layout/layout_participant.cpp` | layout-shapes-paint | DIVERGENT |
| B6-0456 | `src/layout/layout_sizing_style.cpp` | layout-shapes-paint | ADAPTED |
| B6-0457 | `src/viewmodel/runtime/viewmodel_instance_asset_blob_runtime.cpp` | data-bind-view-model | ADAPTED |
| B6-0458 | `src/viewmodel/viewmodel_instance_asset_blob.cpp` | data-bind-view-model | ADAPTED |

These closed register row #H5 and emptied `POST_AUDIT_UNAUDITED` in
`tools/b6-audit/check.py`; the gate census is now 456 rows at
22/211/156/30/37 (ISOMORPHIC/ADAPTED/DIVERGENT/TRACKED-GAP/N-A). The
per-verdict and per-cluster tables below are the frozen `d788e8ec` sweep and
are deliberately not restated.

Two findings surfaced by this sweep have owners: B6-0455's absent
`ParticipantAnimation` lifecycle (`cascadeLayoutStyle`, `advanceComponent`,
`applyInterpolation` — layout animation state is built only for types that
are-a `LayoutComponent`, so participants snap instead of interpolating) is
register row F15 / UNIV-1603; B6-0453/0454's missing layout-dirty pushes were
fixed same-day (UNIV-1604) and those records are restated post-fix. B6-0455
stays DIVERGENT on its own mutation-gated mechanisms rather than TRACKED-GAP.

## Final verdict totals

| Verdict | Rows |
|---|---:|
| ISOMORPHIC | 19 |
| ADAPTED | 193 |
| DIVERGENT | 157 |
| TRACKED-GAP | 30 |
| UNKNOWN | 0 |
| N/A | 49 |
| **Total** | **448** |

`TRACKED-GAP` means the comparable C++ lifecycle is absent or incomplete in
Rust and an existing F/A/C/RB register item owns it. It closes the audit
decision, not the implementation gap. No row may use that verdict without an
owner in `docs/parity-gap-register.md`.

## Per-cluster totals

| Cluster | Total | ISOMORPHIC | ADAPTED | DIVERGENT | TRACKED-GAP | N/A |
|---|---:|---:|---:|---:|---:|---:|
| [data-bind-view-model](results/data-bind-view-model.md) | 81 | 0 | 31 | 50 | 0 | 0 |
| [animation](results/animation.md) | 86 | 7 | 58 | 13 | 8 | 0 |
| [layout-shapes-paint](results/layout-shapes-paint.md) | 54 | 4 | 10 | 39 | 1 | 0 |
| [unavailable](results/unavailable.md) | 48 | 0 | 0 | 0 | 0 | 48 |
| [misc-core](results/misc-core.md) | 40 | 1 | 27 | 12 | 0 | 0 |
| [assets-importers](results/assets-importers.md) | 36 | 0 | 34 | 0 | 2 | 0 |
| [text](results/text.md) | 30 | 0 | 6 | 15 | 9 | 0 |
| [bones-math-components](results/bones-math-components.md) | 21 | 6 | 6 | 7 | 1 | 1 |
| [constraints](results/constraints.md) | 18 | 0 | 3 | 11 | 4 | 0 |
| [lua-scripting](results/lua-scripting.md) | 14 | 0 | 9 | 2 | 3 | 0 |
| [binary-core](results/binary-core.md) | 8 | 1 | 7 | 0 | 0 | 0 |
| [scripted](results/scripted.md) | 6 | 0 | 1 | 3 | 2 | 0 |
| [focus-input](results/focus-input.md) | 3 | 0 | 0 | 3 | 0 | 0 |
| [artboard](results/artboard.md) | 2 | 0 | 0 | 2 | 0 | 0 |
| [state-machine](results/state-machine.md) | 1 | 0 | 1 | 0 | 0 | 0 |
| **Total** | **448** | **19** | **193** | **157** | **30** | **49** |

## Disposition closure

- Family A, the retained data-bind/view-model core, was rebuilt and closed by
  RB-1.
- Family B, the runtime drawing ownership and traversal boundary, was rebuilt
  and closed by RD-1. The five mesh/slice rows re-audited in the second pass
  are ADAPTED under RF-27/RF-28.
- Focus projection remains a confirmed divergence owned by RB-2.
- Three bounded residuals are now explicit: RB-3 deferred script advance,
  RB-4 scalar ScriptInput rehydration, and RB-5 solid-color paint mutation.
- Every formerly UNKNOWN row is either an idiom-backed ADAPTED/N/A decision or
  a register-owned TRACKED-GAP.

The audit itself is complete. Open F/A/C/RB items remain implementation work
and do not turn the audit back into an UNKNOWN inventory.

## Ratchet

`make b6-audit-check` verifies the pin, row count, unique IDs, exact verdict
census, zero UNKNOWN rows, every exact second-pass disposition,
TRACKED-GAP ownership, and the second-pass evidence links.
