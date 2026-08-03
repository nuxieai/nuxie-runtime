# Script verbs lane report

Date: 2026-08-03

Branch: `levi/script-verbs`

Base: `origin/main` (`a3dc487d`)

## Outcome

V5 / #OR-3 is closed for its declared scope. The C++ and Rust golden runners
now share deterministic grammar, execution, and stream records for:

- typed state-machine `setInput` mutations: bool, number, and trigger;
- slash-path view-model bool, number, string, enum, color, and trigger
  mutations through `--view-model-script`;
- logical artboard resize with DPR-derived physical pixel dimensions; and
- chronological merging of input and view-model scripts, with input events
  first at an exactly equal timestamp.

The comparator parses and forwards `view_model_script`, and corpus regeneration
preserves that field. The C++ change is confined to `tools/golden-runner`; no
tracked file in the pinned upstream Rive checkout was edited.

## Commits

- `b50aab80` — `docs: specify scripted mutation verbs`
- `43b859d7` — `golden-runner: emit scripted mutation verbs`
- `8ac7e4e2` — `rust-golden-runner: execute scripted mutation verbs`
- `b8d35f68` — `golden-compare: cover scripted mutation verbs`
- `f4d00fe6` — `[V5] Capture lane finalization`
- `7d76057f` — `[V5] Test script verb parsers in both runners`
- parity ratchets and this report — final V5 closeout commit

## Corpus evidence

Six new exact entries use existing pinned fixtures:

| entry | fixture | evidence |
|---|---|---|
| `script_verbs_set_input` | `smi_test.riv`, `artboard to nest` / `State Machine 1` | `trig`, `bool`, and `num` typed inputs |
| `script_verbs_view_model` | `data_converter_to_number.riv`, `main` | bool, number, string, enum, and color mutations |
| `script_verbs_view_model_boolean` | `scripted_boolean.riv`, `BooleanArtboard` | `BoolProp` boolean mutation |
| `script_verbs_view_model_nested` | `relative_data_binding.riv`, `main` | nested `child2/xPos` number mutation |
| `script_verbs_resize` | `artboard_width_test.riv` | `320 × 180` logical resize at DPR 2 (`640 × 360` pixels) |
| `script_verbs_merge` | `listener_view_model.riv`, `main` | equal-time input/VM ordering, resize, number, and trigger mutation |

## Verification

Green:

- prerequisite `rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/ && make fixtures`;
- `make script-verbs-test`: C++ parser self-test passed and all 12 Rust
  scripted-runner tests passed;
- golden-compare tests: 18 comparator tests + 1 generator test passed;
- focused differential for all six new entries: exact;
- `make scripted-golden-compare`: 362 entries, 331/331 exact,
  685/685 exact segments, 684/684 side-channel segments, zero divergences;
- `make silver-corpus-test`: Rust and Python suites green (one documented Rust
  test remains ignored);
- parity-scorecard checker tests: 26/26;
- runtime-frame-loop checker: 112/112 unit tests, test-correspondence check,
  and live structural check green;
- `git diff --check`.

## Review closeout

The spec review expanded view-model coverage to the complete #OR-3 mutation
family, added nested property paths, and required exact timestamp ordering plus
an executable parser gate for each runner. The standards review also prompted
complete feature metadata on every new corpus entry. Those findings are
resolved in the final tree.

The first four commits predate the commit-message review and do not carry the
milestone prefix. They were left intact rather than rewriting published lane
history; all closeout commits use `[V5]`.

The first full scripted sweep encountered the known nondeterministic pinned-C++
semantic-tree crash in `data_binding_artboards_test` and `replace_view_model`.
Crash reports created before this lane's C++ commit show the same
`SemanticManager::drainDiff` / stale nested-artboard stack, so it is not a lane
regression. The required clean corpus-wide rerun above completed successfully.

Inherited checker failures, unrelated to this lane:

- port-manifest unit tests: 19/21, with two stale `P3E` expectations after the
  existing GPUCEIL reclassification;
- B-6 audit: repository manifest pins upstream `4ac7b327`, while that checker
  currently expects `d788e8ec`;
- runtime-drawing checker: its unit suite is 7/7, but the live ledger still
  expects the removed `fn update_runtime_path_composer` anchor.

These failures were present in current `origin/main`; this lane does not alter
their manifests, ledgers, or runtime owners.
