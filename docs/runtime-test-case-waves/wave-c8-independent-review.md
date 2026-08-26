# Wave C8 independent adversarial review

Verdict: **REJECTED — four rows bypass or alter a pinned intermediate owner**

Reviewed candidate: `039916181`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

## Blocking findings

1. `reader_test.cpp#2` is not a direct safe spelling of the pinned
   `decode_string(12, bytes, end, destination)` call. The Rust test prepends a
   new varuint length byte and calls the downstream composite
   `BinaryDataReader::read_string`, then subtracts that extra read from its
   position assertion. This bypasses the exact intermediate decoder and adds
   an action that is not present upstream. Owned output bytes and overflow
   state are valid Rust safety substitutions, but the extra length decode is
   not. Demote the row to strict pending and remove the proxy test unless a
   callable direct safe decoder owner is exposed.

2. `serialized_rendering_test.cpp#1` derives its loop count from the live
   `walkAnimation->durationSeconds()` owner. The manifest replacement instead
   hardcodes `int(3.0 / 0.016)`. Matching this fixture's current count does not
   execute the causally controlling duration owner. The final SRIV mismatch is
   downstream evidence only. Demote the row or make the replay derive the loop
   count from the selected live animation.

3. `serialized_rendering_test.cpp#28` loops while the actual
   `stateMachine->advanceAndApply(0.25f)` result remains true. The manifest
   hardcodes five repeated advances and discards every return value. This
   bypasses the advance-status owner named by the test and makes the final SRIV
   mismatch a downstream proxy. Demote the row or preserve the real
   return-controlled loop.

4. `serialized_rendering_test.cpp#35` has a different action stream. Pinned C++
   evaluates `int(4.0f / 0.016f)` in `float`, producing
   `249.99998474121094` and therefore 249 loop iterations. The Python manifest
   generator evaluates binary64 literals and emits 250 frame/advance/draw
   iterations. The row cannot be credited even though its current mismatch is
   at frame zero. Correct the generator to C++ `float` arithmetic and refresh
   the manifest, or demote the row.

## Accepted review findings

- Signed-header cases 1-4, 6, and 7 use the retained `SignedContent` owner and
  preserve their literal bytes and ordered observables. Case 5 is correctly
  pending because a failed Rust parse cannot expose the pinned post-failure
  `isSigned()` state. The nine ScriptAsset rows correctly reject the local
  `ScriptAssetProbe`.
- All four ObjectStream/PODStream rows correctly remain pending; the local
  queue and native-byte reconstructions are not retained owners.
- Serialized cases 23-25 correctly remain pending because final SRIV replay
  omits their intermediate `RandomProvider::totalCalls()` assertions.
- The remaining claimed Silver streams preserve their fixtures, selectors,
  mutations, pointer actions, compile-time loop counts, frame boundaries, and
  sole final SRIV assertion. The 13 surviving red rows fail at their named
  renderer-stream difference rather than setup.
- Reader cases 1, 3, and 4 are narrow safe-owner adaptations that preserve the
  literal byte ranges, result sequence, and success/overflow observables.

## Corrected semantic ceiling

Until corrected and freshly rereviewed, Wave C8 can claim at most:

- 22 executable passes;
- 13 genuine executable expected reds;
- 27 honest pending owner/action blockers;
- 32 direct, three adapted, zero differential, and 27 pending rows.

The candidate's current ledger still overclaims the four blocked rows, so Wave
C8 remains accepted at **0/62**.

## Review evidence

All candidate, retained signed-header, shared manifest, and four pinned source
hashes match the candidate receipt. The generated and checked manifest agree
for all 38 serialized-rendering entries. Focused non-incremental execution is
green for the four Reader tests, the signed-header target, and the Silver
target (13 pass / 16 ignored); those passing mechanics do not cure the four
semantic defects above. Candidate diff checks are green. The recorded release
containment result was reused because all candidate evidence bytes are
unchanged.

This receipt changes no candidate test, ledger, manifest, generator, fixture,
or production behavior.
