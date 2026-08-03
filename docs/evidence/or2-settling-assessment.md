# Can an embedder stop-when-settled loop trust the settled bool? (UNIV-1353)

Date: 2026-08-02. Evidence commit: the [OR-2] landing this file ships with.
Question (Levi): "validate the side channel behavior and be confident that we
can implement a loop that stops." Downstream consumer: UNIV-1343 (gate editor
rAF scheduling on the advance bool).

## Verdict

**Yes, with one stated boundary.** After the [OR-1]/[OR-2] gate, the Rust
runtime's advance bool is differentially pinned against pinned C++
`Scene::advanceAndApply` across the whole corpus, every sample, every
advance — 324/324 entries exact, 669 side-channel segments, with exactly one
carve-out (V11) that does not involve the settled bool. An embedder loop of
the form *advance → if settled, stop scheduling; wake on input/VM mutation*
matches C++ frame-scheduling behavior on every covered scene. The full
stop-the-loop endpoint UNIV-1343 wants is implementable now on the runtime
surface; the remaining caveats below are about *what the C++ contract itself
means*, not about Rust drifting from it.

## What was proven, and how

The settled bool in the channel is `!advanceAndApply(elapsed)` — the pinned
C++ facade return, including its subtleties:

- **Zero-second forcing** (`state_machine_instance.cpp:2612`): a 0-second
  advance always reports unsettled. Verified corpus-wide (every t=0 sample).
- **Pending-report terms** (`:2663-2665`): reported events / listener
  view-model reports force another frame. Verified on every event corpus
  file, including listener-fired and timeline-fired events.
- **Quantized nested artboards** (`nested_artboard.cpp:983-986`): a held
  frame with unflushed accumulated time keeps demanding redraws.
  `settle_quantized_nested_keeps_going` pins it at samples straddling the
  hold boundary.
- **Static scenes** (`static_scene.cpp:22-28`): C++ returns keep-going
  unconditionally — a scene with no state machine NEVER reports settled.

Dedicated settling differentials (post-first-frame samples, past the V2
t=0-only blind spot), all byte-exact between the two runtimes:

| entry | shape | result |
|---|---|---|
| `settle_one_shot_reports_settled` | one-shot SM animation, samples 0..3s | unsettled at 0, settled from t=1, STAYS settled |
| `settle_loop_never_settles` | looping timeline events, samples 0..3s | never settled, including across loop wraps |
| `settle_quantized_nested_keeps_going` | quantize+speed nested SM | never settled while quantization holds frames |

Also load-bearing: `event_on_listener` (settles at t=1 after a pointer
click sequence; hit results and the fired-event report identical), and the
full 14-file pointer-script corpus with tri-state `HitResult` per verb.

## Bugs the gate already caught (why this gate earns its keep)

1. **The settled bool was being dropped on the floor** (fixed in [OR-2]):
   the runner-facing `advance_frame_components_with_state_machine` discarded
   the components' keep-going result (`.map(|_| ())`), so quantized nesteds
   and solo-hidden nested machines reported settled while C++ kept going —
   the dangerous direction (a stop-loop would freeze mid-animation). Two
   corpus files diverged; both exact after the fix
   (`advance_frame_components_with_state_machine_report`).
2. **V11**: a real missed root-layer transition at t=0 on
   `global_variables_test`, invisible to the t=0-only draw floor, caught by
   the `statesChanged` channel; draw diverges up to 32px at t>0. Filed with
   repro; does not involve the settled bool.

## Boundaries an embedder loop must respect (C++ contract, not Rust drift)

1. **Sticky-unsettled classes exist by design.** Static scenes (C++ returns
   keep-going forever) and quantized nesteds (until flushed) never settle.
   A loop must treat settled as "may stop", never invert it into "must have
   settled by now". For static scenes the editor should not drive a loop at
   all (nothing animates); the constant unsettled signal is C++'s own
   contract, mirrored deliberately (`static_scene.rs:63`).
2. **Wake conditions are the embedder's job.** Settled means "no further
   frame needed *absent new input*". Pointer input, direct SM inputs, and
   VM mutations can re-arm; the runtime's `needs_advance` flag covers this
   internally, but the loop must advance once after any host-driven
   mutation rather than trusting a stale settled.
3. **The gate covers the corpus.** 324 files, all their samples and input
   scripts. Post-first-frame coverage of settled is corpus-wide for every
   multi-sample entry plus the three dedicated `settle_*` entries; the 237
   t=0-only entries observe only the forced-unsettled value until #OR-4
   densifies sampling. The differential fuzzer (#OR-7) will extend this to
   randomized times/inputs.

## What's still missing on the surface (documented remainder, register V4)

- Per-layer changed-state **identity** (C++ `stateChangedByIndex`): Rust
  records only the count. Not needed for the stop loop.
- View-model value dumps: no pinned enumeration order exists on both sides;
  comparing without one would only measure enumeration noise.
- Hover cursor; key/text verbs (#OR-3/#FT-TEXT); audio channel (#FT-AUDIO).
