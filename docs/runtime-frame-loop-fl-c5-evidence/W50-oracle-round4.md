REJECT. The O4 primitives can produce the desired order when manually driven, but the real frame-loop path does not preserve it.

## Spec/oracle

BLOCKING

- Production O4 ordering still diverges. C++ recursively notifies ancestors and executes each audio tail before returning ([state_machine_instance.cpp:3155](/Users/levi/dev/oss/rive-runtime/src/animation/state_machine_instance.cpp:3155)). Rust first advances the entire component list ([artboard.rs:5897](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/artboard.rs:5897)), collects reports ([artboard.rs:6150](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/artboard.rs:6150)), then dispatches and flushes afterward ([state_machine_instance.rs:12478](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12478)). For sibling reporters A/B, Rust can produce `leaf-A-local, leaf-B-local, root-A-local, root-A-audio, leaf-A-audio...`; C++ completes A’s entire chain before advancing B. This contradicts W48’s “before the next report batch” claim ([W48:17](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/W48-round4-corrective-report.md:17)).

- Deferred audio can become stale or undelivered. A later scripted component may return `ScriptError` ([artboard.rs:5940](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/artboard.rs:5940)); the collector then clears batches and returns without flushing prior deferred audio ([state_machine_instance.rs:12480](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12480)). C++ has already completed bubbling and audio synchronously.

NON-BLOCKING

- The isolated one- and two-ancestor primitive sequences are correct, but their tests manually perform `notify → take → notify → flush` ([three-level test](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:17690), [two-level test](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:17794)). They do not exercise production collection timing.

## Standards/spec ownership

BLOCKING

- Round 4 reintroduced substantive event/audio orchestration into [artboard.rs:10155](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/artboard.rs:10155), violating the thin-delegation boundary at [impl-spec.md:613](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-impl-spec.md:613). This regresses the earlier W40 ownership closure.

- FL-B did not leave state-machine files undisturbed. `724b4f4c`, `edddf491`, and `2e2d3c6d` modify several `state_machine/*` files, including the previously accepted FL-C3 owner [state_instance.rs:44](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/src/state_machine/state_instance.rs:44). That file is excluded by FL-B’s allowed production list ([FL-B spec:38](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-b-spec.md:38)) while remaining recorded as independently verified ([manifest:863](/Users/levi/dev/worktrees/nuxie-e2-review/file-correspondence-manifest.toml:863)).

- `cpp_probe` now requires `tools` ([Cargo.toml:32](/Users/levi/dev/worktrees/nuxie-e2-review/crates/nuxie-runtime/Cargo.toml:32)), but the binding WP6/final acceptance commands still omit that feature ([impl-spec.md:399](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-impl-spec.md:399), [impl-spec.md:568](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-impl-spec.md:568)). The mandated exact commands are no longer reproducible as written.

NON-BLOCKING

- Publication prose remains stale: closure says 59 checker tests rather than 66 ([closure:919](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-closure.md:919)), and committed E2 status still instructs landing E2 ([status:831](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-status.md:831)).

## Verified closed items

- Both pixel receipts and all other floor3 receipts carry the full `95333c41` SHA internally ([same-runner:1](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/floor3-pixel-same.log:1), [static:1](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/floor3-pixel-static.log:1)).
- W41 now labels internal closeouts as non-acceptance evidence ([W41:33](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/W41-report.md:33)); W39/W40/W45/W46/W47 are tracked ([README:44](/Users/levi/dev/worktrees/nuxie-e2-review/docs/runtime-frame-loop-fl-c5-evidence/README.md:44)).
- `95333c41` changes only the Makefile after `2e2d3c6d`; `eaf8a6f6` adds documentation/evidence only.
- No O1/O2/O3 behavioral regression was found. The post-W45 delta leaves facade preparation and semantic-dispatch production files unchanged.

A live Cargo rerun was impossible because the frozen read-only checkout rejected Cargo’s target temporary file; receipt and source verification remained available.

Axis summary: Spec/oracle 2 blocking; Standards 3 blocking and 1 non-blocking consistency group.

REJECT