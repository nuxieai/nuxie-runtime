# B-6 structural fidelity audit summary

Audit source: pinned C++ `/Users/levi/dev/oss/rive-runtime` at
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`. All 447 manifest rows have an
on-disk record. The initial sweep records live under `results/`; the
post-RB-1/RD-1 closure decisions are in [SECOND_PASS.md](SECOND_PASS.md).

## Final verdict totals

| Verdict | Rows |
|---|---:|
| ISOMORPHIC | 19 |
| ADAPTED | 192 |
| DIVERGENT | 157 |
| TRACKED-GAP | 30 |
| UNKNOWN | 0 |
| N/A | 49 |
| **Total** | **447** |

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
| [binary-core](results/binary-core.md) | 7 | 1 | 6 | 0 | 0 | 0 |
| [scripted](results/scripted.md) | 6 | 0 | 1 | 3 | 2 | 0 |
| [focus-input](results/focus-input.md) | 3 | 0 | 0 | 3 | 0 | 0 |
| [artboard](results/artboard.md) | 2 | 0 | 0 | 2 | 0 | 0 |
| [state-machine](results/state-machine.md) | 1 | 0 | 1 | 0 | 0 | 0 |
| **Total** | **447** | **19** | **192** | **157** | **30** | **49** |

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
