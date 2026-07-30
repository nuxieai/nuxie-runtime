REJECT. FL-B production behavior remains clear, and all three original attacks are improved, but round 11 still leaves two blocking detector evasions.

## BLOCKING findings

1. **`site_hash` does not bind a site’s location.**

The exact round-11 relocation fixture trips because it changes the guarded statement: approved and relocated hashes were respectively `c82c3c83…` and `1cb3a884…`. However, moving the identical guarded statement from an approved branch to a forbidden branch under the same function produced:

```text
approved:  Approved::choose StateMachine c82c3c83…
relocated: Approved::choose StateMachine c82c3c83…
REGISTRY_KEYS_EQUAL True
```

The detector hashes only the innermost statement token stream ([main.rs](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/rust-owner-detector/src/main.rs:770)); the checker then authorizes `(anchor, guarded_name, site_hash)` without location or enclosing control-flow context ([check.py](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/check.py:1681), [check.py](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/check.py:1701)). Thus a literal same-anchor relocation consumes the original row, contrary to the requirement that *any relocation* change identity ([W89 spec](/Users/levi/dev/worktrees/nuxie-e9-verify/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W89-round11-spec.md:81)).

The permanent test misses this because its “relocation” also changes parameters and policy calls ([test_check.py](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/test_check.py:5109)).

2. **Owner-wrapper propagation is only one hop.**

The original direct `DELIVER → notify_events` attack is closed: owner audit emitted `export dispatch DELIVER`, and the non-owner call emitted a dispatch hit.

A minimal two-hop form still escapes:

```rust
pub(crate) fn BRIDGE(...) { StateMachineInstance::notify_events(...); }
pub(crate) fn DELIVER(...) { BRIDGE(...); }
```

Direct replay emitted only:

```text
export dispatch BRIDGE
```

The non-owner `state_machine_instance::DELIVER(owner)` emitted no detector output. Owner exports are collected in one scan and passed once to non-owner analysis ([check.py](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/check.py:1599)); newly discovered owner aliases are not fed back through owner analysis. The permanent negative covers only the direct wrapper ([test_check.py](/Users/levi/dev/worktrees/nuxie-e9-verify/tools/runtime-frame-loop-port/test_check.py:4799)). This leaves the neutral owner-wrapper evasion open under one ordinary helper layer.

## Original attack replay

- **Direct `DELIVER` wrapper:** trips as claimed.
- **Forged local function:** correctly anchors as
  `ArtboardInstance::dispatch_nested_text_input_at_focus::dispatch_nested_key_input_at_focus`, distinct from the blessed anchor. It therefore produces an unregistered hit and leaves the blessed row unmatched.
- **Round-11 altered-content relocation fixture:** hashes differ and fails.
- **Literal same-statement relocation:** bypasses, as described above.

## Held verification

- Detached clean publication: `a4522812`, parent candidate `14b18765`.
- `crates/` tree identity at `e729dd74`, `14b18765`, and `a4522812` is exactly `473036c1d4eb1491bbc5b10f3b5ca2241dcc5d2c`. The corrective changes only eight tools/docs paths.
- All seven FL-B regression proofs share unchanged `cpp_probe.rs` blob `4bc2467d…`: invalid interpolator, importer cursor, doomed-owner sink, negative-speed remap, signed loop override, empty-baseline reset, and NaN direct blend ([tests](/Users/levi/dev/worktrees/nuxie-e9-verify/crates/nuxie-runtime/tests/cpp_probe.rs:19594)).
- Blend1D still owns only occurrences, `from`, `to`, and its construction-time reset ([state_machine.rs](/Users/levi/dev/worktrees/nuxie-e9-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:820)); application remains reset-before-blend ([state_machine.rs](/Users/levi/dev/worktrees/nuxie-e9-verify/crates/nuxie-runtime/src/state_machine/state_machine.rs:1043)). BlendDirect retains definition handles without copied payloads ([blend_state_direct_instance.rs](/Users/levi/dev/worktrees/nuxie-e9-verify/crates/nuxie-runtime/src/state_machine/blend_state_direct_instance.rs:4)).
- Candidate-mode live checker passed with all four ownership ratchets at `0/0..0`; candidate mode is active ([ownership ledger](/Users/levi/dev/worktrees/nuxie-e9-verify/docs/runtime-frame-loop-ownership.toml:22)).
- Fingerprint independently reproduced: `7315` files, `c2447d362bd936edf49698ba9b89b25cd8078cf0c73fec1304e9bbb2b65bed82`, exactly matching the trace ([trace](/Users/levi/dev/worktrees/nuxie-e9-verify/docs/runtime-frame-loop-trace.json:7850)).
- Registry audit: 30 rows are in FL-B’s `artboard.rs`; running the round-11 detector against the pre-corrective `e729dd74` source matched all 30 exactly, with zero missing or extra rows. Representative legitimate sites include nested-event dispatch ([artboard.rs](/Users/levi/dev/worktrees/nuxie-e9-verify/crates/nuxie-runtime/src/artboard.rs:5696)) and post-animation-owner dispatch/audio unwind ([artboard.rs](/Users/levi/dev/worktrees/nuxie-e9-verify/crates/nuxie-runtime/src/artboard.rs:10297)). The other eight registry rows are in focused-input dispatch, outside FL-B’s narrow production-file list.

The read-only sandbox prevented rerunning the tempfile-based 83-test harness, but the unchanged live checker, exact cached detector, direct attacks, fingerprint, and registry comparison all ran successfully.

No NON-BLOCKING candidate findings.

**REJECT for FL-B reacceptance.**