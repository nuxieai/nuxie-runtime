# FL-E8 static-text, ListPath, and RawText evidence

Authority: `.fle8/directives.md`, `.fle8/impl-spec.md`, and the binding
orchestrator rulings in `.fle8/rulings.md`. The pinned C++ oracle is
`d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

The seven directive-owned C++ rows are:

- `src/shapes/list_path.cpp`
- `src/text/raw_text.cpp`
- `src/text/text_modifier.cpp`
- `src/text/text_style.cpp`
- `src/text/text_style_feature.cpp`
- `src/text/text_target_modifier.cpp`
- `src/text/text_variation_modifier.cpp`

## Work-package status

- [x] WP0 ledger revert and family map: rejected D13/D14/D15 ceilings removed,
  FLR-20 reduced to D3/layout-engine, exactly seven family rows pending in
  wave FL-E8, and counts `334/1/7`.
- [x] WP1 static-text extensions: ordered generic/subtype modifier
  registration, feature-aware shaping, occurrence-local target resolution,
  localized variable-font splitting, live dirt/callback behavior,
  `modifyOrigin`, deterministic fixtures, D-ST differentials, and G-ST
  goldens. Exactly five rows promoted; counts `339/1/2`.
- [x] WP2 dynamic ListPath: occurrence-local synthetic vertices and number
  subscriptions, positional remap/teardown, safe invalid-input boundaries,
  exact eight-phase/60-frame renderer parity, and D-LP-EDGE lifecycle coverage.
- [ ] WP3 standalone RawText and shared color-glyph engine: pending.

## Ledger transitions

| Point | Faithful | Divergent by decision | Pending | Promotion |
|---|---:|---:|---:|---|
| WP0 | 334 | 1 | 7 | none |
| WP1 | 339 | 1 | 2 | five static-text rows |
| WP2 | 340 | 1 | 1 | `src/shapes/list_path.cpp` |
| WP3 | 341 | 1 | 0 | `src/text/raw_text.cpp` |

The sole divergent row is D3/layout-engine. WP1 promotion records
implementation acceptance while `verification = "pending-verification"`
remains an independent orchestrator action.

## WP0 evidence

- Checker negative controls reject D13, D14, D15, any ceiling other than D3,
  membership/dependency drift, and count drift.
- Production WP0 ledger result: faithful 334, divergent-by-decision 1,
  pending 7.
- The additive WP1 gate floor reran runtime, ordinary/forced-scripted corpus,
  binary, and checker coverage without weakening the WP0 baseline.

## WP1 differentials and owner ratchets

- `D-ST-STRUCT`:
  `d_st_struct_live_cpp_modifier_registration_matches_rust`; preserves the
  upstream one-Text/one-group/one-range/zero-abstract-modifier/interpolator
  assertion structure and registered vector counts.
- `D-ST-FONT`:
  `d_st_font_live_cpp_embedded_font_fixture_matches_rust`; ports all four
  live phases of `data_binding_fonts_test.cpp:18` and compares flattened
  shaped-glyph results after default shaping, decoded Kablammo replacement,
  and both listener swaps.
- `D-ST-FEATURE`:
  `d_st_feature_live_cpp_authored_feature_chain_matches_rust`; drives the same
  generated fixture through C++ and Rust and compares ordered feature values
  plus shaped glyphs at baseline, no-dirt live edit, clone rebuild, and
  independent legitimate shape invalidation.
- `D-ST-VARIATION`:
  `d_st_variation_live_cpp_axis_value_mutation_matches_rust_update`; drives
  the same generated fixture through C++ and Rust and compares glyphs,
  per-glyph axis coordinates, coverage splits, raw strength extrapolation,
  live value dirt, clone locality, tag inaction, and later refresh.
- `D-ST-TARGET`:
  `d_st_target_live_cpp_missing_target_is_ok_like_rust`; compares live target
  reports for valid, missing, wrong-parent, and inherited follow-path cases.
  Its synthetic `Bone` target proves schema-wide `TransformComponent`
  acceptance beyond the former kind whitelist.
- Focused R-ST-OWNER tests (6/6) additionally cover feature callback inaction,
  default/duplicate/raw-strength variation math, coverage splitting,
  occurrence/clone locality, axis dirt, font invalidation, target safety,
  generic vectors, and `modifyOrigin`.

## Fixtures and corpus

- `fixtures/fl-e8/text_style_feature.riv` SHA-256:
  `86d0ff4c56ea1fbc1db12dd89e8a8a123ba94b3c0da79f4b13914f4e709d9ec7`.
- `fixtures/fl-e8/text_variation_modifier.riv` SHA-256:
  `d52215dac9580188dfb0ffec552c79a4e667d8df637b79afb1e34fb5777f0494`.
- The codegen integration test regenerates each fixture twice, proves byte
  determinism and Rust import/type keys, and the generation command was also
  independently rerun with byte-for-byte comparison.
- Corpus delta: +2 exact entries and +2 exact segments, from 317/647 to
  319/649. Ordinary and forced-scripted runs both report 319 exact entries,
  649 exact segments, and zero divergence/unsupported/not-yet rows.
- Workspace-relative `fixtures/` resolution is tested; all other relative
  corpus paths continue to resolve under the pinned runtime checkout.

## WP1 gate evidence

- Fresh C++ probe build: pass.
- Live D-ST suite: 5/5 pass.
- Focused R-ST-OWNER suite: 6/6 pass.
- `cargo test -p nuxie-runtime`: pass; final count is in `W122-report.md`.
- `cargo test -p nuxie-codegen`: 1 command + 1 fixture integration + 7 schema
  tests pass.
- `cargo test -p nuxie`, unfiltered: pass. The default-renderer test executes
  its draw/pixel assertions when an adapter exists and validates the public
  adapter-unavailable error contract on this GPU-less host; it is not skipped.
- Ordinary golden: 319/319 entries, 649/649 segments, zero failures.
- `make scripted-golden-compare`: 319/319 entries, 649/649 segments, zero
  failures. The real target reuses a complete pinned decoder archive set when
  the external checkout is read-only, freshly links the runners, and performs
  the full comparison.
- Binary differential: 70/70 pass.
- Checker: 94/94 unit tests pass; production result 339/1/2 and
  `path_epoch_compensation=64/0..67`.

## WP2 ListPath evidence

- `D-LP-INIT/XY/RD/DETACHED/POINT/INVALID/PARTIAL/LIVE`: the generated
  `list_to_path` action stream ports `data_binding_test.cpp:1585-1819` and is
  byte-exact against the pinned `.sriv`, including all 60 live frames.
- `D-LP-EDGE`: focused live-fixture coverage proves invalid-index no-dirt,
  identical reconciliation dirt, positional reorder, duplicate rows,
  same-count replacement, old-source unsubscribe, live replacement writes,
  cold clone ownership, tail shrink, and zero/one-vertex empty rendering.
- `R-LP-OWNER`: the direct owner retains one listener, synthetic detached
  vertex, strong instance reference, and exact number-cell subscriptions per
  filtered row. Drop/remap clears subscriptions before vertex state is lost.
- `G-LP` and `S-LP`: the existing `list_to_path` golden row remains singular
  and exact; its silver row is now `status = "exact"` only after an independent
  byte-exact validation.
- Safe divergence is confined to C++ null/UB preconditions: null list/item and
  wrong converter output clear stale Rust state, unsubscribe, dirty, and return
  failure. No crash is manufactured.
- WP2 gates and final counts are recorded in `W123-report.md`.

## Corrective additive TextModifierGroup work

The `src/text/text_modifier_group.cpp` row remains faithful and was not
reopened. WP1 additively restores behavior omitted by the older mapping:
generic modifier retention, shape/follow-path subtype indexing, shape-aware
range dirt, localized run splitting, and `modifyOrigin`. Existing
modifier-range, follow-path, TextInput, FL-E7, and FL-E2 ratchets remain green.

## RawText GM follow-up lane

The upstream `gm-rawtext`, `gm-feather`, `gm-atlas`, `gm-dither`, and
`gm-subpass` fixtures are outside this family's directed corpus reach. They
remain a nonblocking follow-up and are not claimed as standalone RawText or
U-color acceptance evidence.

## Ambiguity and safety log

- Wrong-parent target objects are removed by C++ addition; Rust rejects the
  invalid occurrence before registration. Missing IDs succeed with a null
  target. Valid kinds use schema `TransformComponent` ancestry, matching the
  domain of C++'s unchecked cast.
- Feature tag/value and variation axis-tag writes remain non-dirty and retain
  their old option snapshot. Clone construction or a legitimate independent
  shape invalidation rebuilds the snapshot from live values. Whole-style axis
  edits retain C++ shape dirt, and all shaping paths apply axes and features
  together.
