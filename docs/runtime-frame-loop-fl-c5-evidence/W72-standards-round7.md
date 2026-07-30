## Standards/structure

**BLOCKING — ownership ratchets remain structurally evadable.**

W63 requires structural detection across every non-owner file and every call form, allowing only blessed policy entries ([W63 spec](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:61)). Repository-wide globs are now correct, but the detector still uses token-shape regexes requiring canonical, adjacent `::member`/`.member` spellings ([check.py](/Users/levi/dev/worktrees/nuxie-e5-review/tools/runtime-frame-loop-port/check.py:541)); selection handling recognizes `use … as` aliases but not Rust type aliases ([check.py](/Users/levi/dev/worktrees/nuxie-e5-review/tools/runtime-frame-loop-port/check.py:602)).

Direct calls to the same detector used by the live checker produced zero hits for these new, valid Rust evasions:

```rust
// dispatch — hits=[]
let send = StateMachineInstance :: notify_events;

// collection — hits=[]
let get = StateMachineInstance::r#reported_event;

// audio — hits=[]
StateMachineInstance :: flush_deferred_owner_audio_events(owner);

// selection — hits=[]
type Anim = RuntimeNestedAnimationInstance;
if let Anim::StateMachine(owner) = animation { move_policy(owner); }

// macro-composed dispatch — hits=[]
let send = member!(StateMachineInstance, notify_events);
```

Thus all four ownership ratchets can still be bypassed. The scanner is invoked unchanged for each non-owner file at [check.py](/Users/levi/dev/worktrees/nuxie-e5-review/tools/runtime-frame-loop-port/check.py:1737). W69’s permanent negatives cover angle-bracket UFCS and `use` aliases, but not these forms ([test_check.py](/Users/levi/dev/worktrees/nuxie-e5-review/tools/runtime-frame-loop-port/test_check.py:3717)). The packet’s “alias-resistant” closure claim is therefore premature ([closure packet](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-closure.md:900)).

The original W66/W68 probes do now register hits: function-item dispatch/audio aliases, direct enum-variant aliases, angle-bracket UFCS, and `use`-based enum aliases all returned detected offsets. The blocker is specifically the newly demonstrated evasion surface.

## Evidence/packet

**NON-BLOCKING — stale publication pointer remains.**

W63 requires `Next` to be truthful at publication and contain no “publish-this” instruction ([W63 spec](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-evidence/round-specs/W63-round7-spec.md:72)). Published HEAD still says “Publish the staged E5 evidence/docs packet” ([status](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-status.md:829)), although `Current` already declares E5 operative. This is the previously identified stale-pointer nit.

Everything else requested checked out:

- HEAD is clean, detached, and exactly `3bef19da…`; candidate `171b5703…` has parents `192cbbbe…` and dated upstream boundary `afe71e30…`.
- The live checker itself exited green. Independent recomputation matched the trace exactly: 7,295 files, SHA-256 `92a2588f…`, and identical runner provenance ([trace](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-trace.json:7874)).
- The complete Make target discovered 67 tests, but this read-only sandbox prevented every temporary-directory creation: four read-only tests passed and 63 errored before assertions. I therefore do not claim a fresh 67/67 unit run; the live checker/fingerprint/provenance were independently reproduced.
- The Apple amendment accurately cites `afe71e30` on 2026-07-30, removes Apple/XCFramework from the operative floor, and retains size ([README](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-evidence/README.md:23), [implementation spec](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-impl-spec.md:606)). All six historical Apple/XCFramework receipts remain under `superseded/`; all 32 tracked floor receipts have valid, resolvable SHA stamps.
- The packet truthfully describes the boundary merge and main integration ([closure](/Users/levi/dev/worktrees/nuxie-e5-review/docs/runtime-frame-loop-fl-c5-closure.md:875)).
- The merge preserved ownership boundaries. The runtime tree, structural checker/tests, C++ probe, and Rust differential have identical Git object IDs at `192cbbbe`, `171b5703`, and publication HEAD. Only `Makefile` conflicted; its resolution retained FL-C checker/trace/scripted-probe targets while removing Apple targets.
- Previously verified behavioral items remain: report-time bubbling and narrowed bind-only settlement ([owner implementation](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/src/state_machine/state_machine_instance.rs:12931)), deep pre-mix/full-height evidence ([differential](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:84779)), nested-simple singleton/zero-overshoot proof ([differential](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:84937)), and Blend1D clone/remount ([differential](/Users/levi/dev/worktrees/nuxie-e5-review/crates/nuxie-runtime/tests/cpp_probe.rs:20275)).

REJECT