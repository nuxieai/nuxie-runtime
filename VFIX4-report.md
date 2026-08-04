# VFIX lane 4 relaunch report

Date: 2026-08-03

Branch: `levi/vfix-comparator-effects`

Base: local `main` at `d596a19b`

Pinned C++ runtime: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Bootstrap

The required bootstrap completed before row work:

```text
rsync -a /Users/levi/dev/nuxie-runtime/fixtures/ fixtures/
make fixtures
make cpp-probe
```

## Row results

| Row | Corpus entry | Result | Commits | Evidence / remaining boundary |
| --- | --- | --- | --- | --- |
| V22 | `computed_values_test` | Artifact reclassified; remains `diverges` | `9812c53d`, `c9371a44` | Fresh ordinary runners are exact. Fresh scripting-enabled runners reproduce a 245x250 versus 490x362.5 clip. The shared corpus status cannot express ordinary exact plus scripted divergence, so it conservatively remains divergent. The register names the scripted-runner feature split rather than computed-value propagation. |
| V27 | `image_fit_alignment_2` | Artifact reclassified; remains `diverges` | `efe42657`, `a7d38d44` | Fresh ordinary runners produce byte-identical 417,801-byte streams. Scripting-enabled capture reproduces the image-buffer command-phase mismatch. The register names the runner-feature artifact. |
| V35 | `stateful_source_switch` | Artifact reclassified; remains `diverges` | `8656d759`, `078888ff` | A fresh ordinary debug C++ capture terminates with SIGSEGV. Fresh scripted capture completes and reproduces the 100-versus-75 geometry mismatch. The register separates the ordinary oracle crash from the scripted source-switch geometry residual. |
| V36 | `superbowl` | Artifact reclassified; remains `diverges` | `95895288`, `0246ae91` | Fresh ordinary runners are exact. Fresh scripting-enabled runners reproduce an empty-path versus compound-path mismatch. The register names scripted path-retention as the artifact cause. |
| V11 | `global_variables_test` | Comparator settlement fixed; draw row remains `diverges` | `95b8010d`, `aa5cf3a7`, `4d45eff6` | `ConditionComparisonSelf` now observes persistent source-cell change state through the same bind/update settlement boundary as C++ and nested rebinding no longer consumes it early. The focused C++ probe matches the two t=0 root-layer state changes. Dense draw capture still has an independent mounted-layout x-position residual (Rust 107.701172, C++ 115.701172), recorded in the register. Upstream: `transition_viewmodel_condition.cpp:49-60,1098-1108`; `state_machine_instance.cpp:2665-2697`. |
| V33 | `stateful_keyed_trigger` | Exact | `042bdb11` | Keyed trigger callbacks now fire the trigger property and retain its change until the nested comparator consumes it. Focused and full scripted comparison are exact. Upstream comparator/settlement references: `transition_viewmodel_condition.cpp:49-60,1098-1108`; `state_machine_instance.cpp:2665-2697`. |
| V25 | `group_effect` | Script invalidation fixed; row remains `diverges` | `206e283c`, `9224a0be` | A true scripted path-effect advance publishes `SCRIPT_UPDATE` at the effect dependency slot, with a unit regression test. Fresh capture shows a separate retained-chain residual: C++ builds the full chained `GroupEffect` path while Rust still builds the shortened `TargetEffect` chain. Upstream: `scripted_path_effect.cpp:111-132,199-207`; `shape_paint.cpp:115-152`. |
| V30 | `path_effect_with_feathers` | Effected feather invalidation restored; row remains `diverges` | `b2db70e0`, `bcee2637` | The effected inner feather now observes scripted path-effect invalidation. The row retains the upstream shortened effect-path residual shared with V25. Upstream: `scripted_path_effect.cpp:111-132,199-207`; `shape_paint.cpp:115-152`. |

The first artifact pass incorrectly treated unfinished runner sessions as successful captures. Each affected row was recaptured and corrected in its own follow-up commit; no false exact annotation remains.

## Verification

Focused gates:

- Each row's focused scripted corpus comparison completed successfully after its row commit.
- The final eight-row comparison completed with one exact row (`stateful_keyed_trigger`) and seven verified registered divergences.
- `cargo test -p nuxie-runtime` passed.
- The V11 C++ comparator probe passed and reported the expected two root-layer state changes.
- The V25 path-effect invalidation unit test passed.

Required final gates:

- `cargo test -p nuxie --features scripting` — passed.
- `make scripted-golden-compare` — passed on the isolated final run: 362 entries, 327 exact, 1,069 exact segments, 1,069 side-channel segments, 26 verified divergences, 9 not-yet, zero failures.
- `make runtime-frame-loop-port-check` — passed: 112 checker tests plus correspondence and live ownership checks.
- `make rust-attribution-check` — passed: 10 tests and complete in-scope Rust-source classification.
- `git diff --check main...HEAD` — passed.
- Direct `rustfmt --check` over every modified Rust file — passed. Repository-wide `cargo fmt --all -- --check` also reports pre-existing formatting drift in untouched renderer and facade files; those unrelated files were not modified.

The first full scripted comparison encountered transient SIGSEGVs from the pinned C++ runner on `data_binding_artboards_test`, `replace_view_model`, and V35. An immediate focused rerun of all three with the same binaries passed, and the subsequent isolated full run passed all 362 entries. No Rust failure or corpus reclassification was inferred from that transient capture failure.

## Review closeout

The requested Codex review helper was invoked as:

```text
/Users/levi/.codex/skills/codex-review/scripts/codex-review --mode branch --base main
```

It could not initialize because the managed workspace denies writes to the parent repository's worktree `FETCH_HEAD` and Codex app-server state. It made no changes. A manual `main...HEAD` source/test/corpus audit found no additional actionable issue. The only accepted cleanup was formatting the modified `state_machine_instance.rs` imports and blank lines (`4d45eff6`); focused formatting and diff checks pass.

`docs/v-row-triage.md` remains local and untracked as requested.
