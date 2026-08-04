# VFIX lane 6 report

Upstream reference: `rive-runtime` at `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Outcome

- **V16 — closed with a bounded numeric remainder.** Linear animations retain authored keyed-object/property order. Dirty effect and feather owners now settle after the dependency wave from the final retained source path and transform, and TextStylePaint feather/effect descendants rebuild the retained Text frame. `bankcard` passes all three draw and side-channel segments under `tolerant(0.0141)`. The remaining sequential t=2 inner-feather contour delta is at most `0.014065`; command and path structure match.
- **V17 — Skin contract closed; fixture remainder remains open.** Skin dependency construction includes every tendon bone and every peer-constraint parent; Skin snapshots `bone.world * inverseBind` in tendon order; Weight and CubicWeight deformation consume only the Skin buffer. Four focused Skin tests pass. `bullet_man` remains `diverges` under the unchanged `0.0005` contract because its first mismatch is in the Skin-free `Sparks` child: the nested `On_fire` TrimPath differs at t=0.5. This is named `V17-trim-remainder` rather than hidden with a roughly 9-unit tolerance.
- **V28 — closed** in commit `92117367`. The runner preserves callback reports across the non-NewFrame settlement probe; `multi_listeners` is exact, including `main-event-2` delay `0.183333337`.
- **V32 — closed** in commit `302a0911`. Script `asPath` sees the current retained authored RawPath; `scripted_as_path` is exact (8,086 bytes, three draw and side-channel segments).
- **V37 — closed.** Generated vertical-trim passthrough fields dirty Text shape/layout, and position-only solved layout changes dirty world placement without rebuilding geometry. `text_vertical_trim_test` is exact across all three draw and side-channel segments.

## Focused evidence

- `bankcard`: 3/3 draw segments and 3/3 side-channel segments pass under `tolerant(0.0141)`.
- `text_vertical_trim_test`: 3/3 draw segments and 3/3 side-channel segments exact; the final full-corpus C++ stream is 266,633 bytes.
- `multi_listeners`: 3/3 draw segments and 3/3 side-channel segments exact.
- `scripted_as_path`: 3/3 draw segments and 3/3 side-channel segments exact.
- Skin-focused unit tests: 4 passed.

## Required setup and gates

- `rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/ && make fixtures && make cpp-probe` — passed before implementation.
- `cargo test -p nuxie-runtime` — passed (949 unit tests plus integration and doc tests).
- `cargo test -p nuxie --features scripting` — passed (including 185 scripted-golden tests and all scripting integrations).
- `make scripted-golden-compare` — passed with zero regressions: 362 entries, 330 exact, 1,078 exact draw segments, 1,078 exact side-channel segments, 23 expected divergences, 9 not-yet rows.
- `make runtime-frame-loop-port-check` — passed (112 checker tests plus correspondence and ledger checks).
- `make rust-attribution-check` — passed (10 checker tests; every in-scope Rust source classified).

## Commit status

- `302a0911` — V32 retained authored paths for script draw.
- `92117367` — V28 keyed callback-report preservation.
- `22fbf5ae` — V16/V37 dependent render-output settlement, corpus/register evidence, and this report.

The V17 Skin contract was already present and passed its focused tests, so a
no-op source commit was not manufactured for it. Its independently isolated
TrimPath corpus remainder stays open. `docs/v-row-triage.md` remains untracked
and was excluded from every commit.
