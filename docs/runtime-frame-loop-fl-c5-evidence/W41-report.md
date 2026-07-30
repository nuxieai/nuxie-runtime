# W41 FL-C5 family fix round two

Immutable combined-family production candidate:
`95333c41fe68ab6a2a5486874ffd0c59cd4381be`.

FL-C5 round-two production landed as `ea38d33b`; FL-B round-two corrections
then landed as `edddf491`. Round-four corrective production landed as
`2e2d3c6d`, and `95333c41` completed its tools-enabled probe target. The full
`95333c41` commit is the production identity used for every operative E2
artifact, trace hash, and floor.

## Binding seam repairs

- Semantic actions use an instance-owned `SemanticNodeResolver`. Production
  has no fabricated resolver or ordinal node-ID projection; without a
  resolver, node `77` returns `false`. The injected resolver proves tap,
  increase, and decrease, plus valid-resolver invalid-action no-op behavior.
  This expectation follows the recorded-seam contract in
  `W41-family-fix-round2-spec.md`; no test was weakened.
- Selected live `AudioEvent` occurrences cross an instance-owned production
  `AudioEventSeam` after bubbling. The default records count and last
  occurrence without a test-only behavior branch.
- Nested-artboard and frame-component event collection delegates through the
  state-machine instance owner. Its retained scratch allocation is reused
  across frames; a renamed duplicate orchestration helper is rejected.
- Public construction retains a machine and records terminal `script_error`
  when scripted preparation fails.
- The public inventory pins all four `RuntimeStateMachine` field types and
  generic hydration's exact `FnOnce` contract in its exhaustive digest.
- The five-pass persistent-dirt differential drives the real
  `advance_and_apply` facade and a genuinely scheduled component fixture.

Two internal subagent closeouts during W41 reported no further findings to
the writer, but they are neither archived independent verdicts nor acceptance
evidence. The independent W39/W40 and W45/W46/W47 verdicts in this directory
rejected earlier candidates. P3 addresses their round-four production and
packet findings; independent post-E2 acceptance remains separate.

## Final in-sandbox floors on `95333c41`

- `cargo test -p nuxie-runtime --lib`: 716 passed.
- `cargo test -p nuxie-runtime --features tools --test cpp_probe`: 816 passed.
- `cargo test -p nuxie --lib`: 146 passed.
- `cargo test -p nuxie-runtime --test public_api_fl_c5`: 1 passed.
- `cargo test -p nux-capi`: 3 library and 16 integration tests passed; doc
  tests passed.
- `cargo test -p nuxie --test public_api`: 14 code/API cases passed. The sole
  failure is the adapter-dependent renderer construction test because this
  sandbox exposes no suitable adapter (`metal found no adapters`). It was not
  skipped or weakened.
- Ordinary golden comparison: 317/317 exact entries and 647/647 exact
  segments; zero divergences, unsupported features, or not-yet cases.
- Scripted golden comparison with all diagnostic verifiers: 317/317 exact
  entries and 647/647 exact segments; zero divergences, unsupported features,
  or not-yet cases.
- `make runtime-frame-loop-port-test`: 66/66, including the W41 ratchets,
  renamed-duplicate negatives, round-four exact-inventory/gating/semantic
  negatives, receipt-stamp checks, and trace identity/artifact negatives.
- Publication validity is enforced by the final
  `make runtime-frame-loop-port-check` after trace regeneration.

## External floors on `95333c41`

- Apple: the operative `floor3-apple.log` is one SHA-stamped clean run whose
  product checks and XCFramework packaging pass; checksum
  `765b11cf03e3dd52b347f712cfa9411821bba01edc96a0b15c52122b450b39ef`.
  The superseded floor2 attempt-1 dirty-tree refusal and successful clean
  attempt-2 remain disclosed in README and their historical receipts.
- Browser: every WebGPU-only invariant passes.
- Same-runner pixels: exact=1,468, byte-exact=1,370, diverges=0.
- Static-reference pixels: exact=1,468, byte-exact=837, diverges=0.
- Size: 8,218,008 bytes without scripting and 9,302,232 bytes with
  scripting, both below the 9 MiB budget.

All five operative `floor3-*` receipts name the full P3 SHA and are copied
with trailing spaces and tabs removed. The floor2 set is superseded.
