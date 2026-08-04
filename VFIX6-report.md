# VFIX lane 6 report

Upstream reference: `rive-runtime` at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Outcome

- **V16 — partial; feather remainder remains open.** Linear animations retain authored keyed-object/property order, but `bankcard` still has a Rive-owned sequential t=2 inner-feather contour delta of at most `0.014065`. A global post-wave suffix-settlement prototype was removed after closeout review because it violated RF-29's explicit retained dependency-node requirement. The row remains `diverges` as `V16-feather-remainder`; no tolerance is used to call it exact.
- **V17 — Skin contract closed; fixture remainder remains open.** Skin dependency construction includes every tendon bone and every peer-constraint parent; Skin snapshots `bone.world * inverseBind` in tendon order; Weight and CubicWeight deformation consume only the Skin buffer. Four focused Skin tests pass. `bullet_man` remains `diverges` under the unchanged `0.0005` contract because its first mismatch is in the Skin-free `Sparks` child: the nested `On_fire` TrimPath differs at t=0.5. This is named `V17-trim-remainder` rather than hidden with a roughly 9-unit tolerance.
- **V28 — closed** in commit `92117367`. The runner preserves callback reports across the non-NewFrame settlement probe; `multi_listeners` is exact, including `main-event-2` delay `0.183333337`.
- **V32 — closed** in commit `302a0911`. Script `asPath` sees the current retained authored RawPath; `scripted_as_path` is exact (8,086 bytes, three draw and side-channel segments).
- **V37 — closed.** Generated vertical-trim passthrough fields dirty Text shape/layout, and position-only solved layout changes dirty world placement without rebuilding geometry. `text_vertical_trim_test` is exact across all three draw and side-channel segments.

## Focused evidence

- `bankcard`: command/path structure and side channels localize the remaining t=2 inner-feather numeric mismatch; the row remains divergent.
- `text_vertical_trim_test`: 3/3 draw segments and 3/3 side-channel segments exact; the final full-corpus C++ stream is 266,633 bytes.
- `multi_listeners`: 3/3 draw segments and 3/3 side-channel segments exact.
- `scripted_as_path`: 3/3 draw segments and 3/3 side-channel segments exact.
- Skin-focused unit tests: 4 passed.

## Required setup and gates

- `rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/ && make fixtures && make cpp-probe` — passed before implementation.
- `cargo test -p nuxie-runtime` — passed (949 unit tests plus integration and doc tests).
- `cargo test -p nuxie --features scripting` — passed (including 185 scripted-golden tests and all scripting integrations).
- `make scripted-golden-compare` — **blocked in the pinned C++ oracle after closeout reclassification.** The full run reaches the expected 362-entry census (329 exact, 1,075 exact draw segments, 1,075 exact side-channel segments, 24 divergences, 9 not-yet rows), but the scripted C++ runner intermittently exits with SIGSEGV on an otherwise passing corpus entry (`stateful_source_switch` in the final attempt; earlier attempts named `data_binding_artboards_test` and `replace_view_model`). A clean release rebuild did not eliminate the full-run crash. Each named entry passes when rerun directly, and the six-entry lane/repro set (`bankcard`, `bullet_man`, `multi_listeners`, `scripted_as_path`, `text_vertical_trim_test`, `stateful_source_switch`) passes with all nine exact draw and side-channel segments. No Rust/C++ comparison regression was reported before the oracle process crashed.
- `make runtime-frame-loop-port-check` — passed (112 checker tests plus correspondence and ledger checks).
- `make rust-attribution-check` — passed (10 checker tests; every in-scope Rust source classified).

## Commit status

- `302a0911` — V32 retained authored paths for script draw.
- `92117367` — V28 keyed callback-report preservation.
- `22fbf5ae` — V16 settlement prototype, V37 render-output settlement, corpus/register evidence, and this report.
- `67cac18d` — removes the RF-29-incompatible V16 prototype and restores an honest divergent classification.
- `0e462993` — confines the V28 runner-only state-probe seam to the existing `tools` feature.

The V28 non-NewFrame probe seam is public only under the runtime's existing
`tools` feature, which the golden runner enables; it is absent from ordinary
runtime builds.

The V17 Skin contract was already present and passed its focused tests, so a
no-op source commit was not manufactured for it. Its independently isolated
TrimPath corpus remainder stays open. `docs/v-row-triage.md` remains untracked
and was excluded from every commit.
