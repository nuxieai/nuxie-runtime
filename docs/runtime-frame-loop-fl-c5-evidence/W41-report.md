# W41 FL-C5 family fix round two

Immutable combined-family production candidate:
`edddf4916e0ff0b7f55e41686704d5d988fae9f4`.

FL-C5 round-two production landed as `ea38d33b`; FL-B round-two corrections
then landed as `edddf491`. The latter is the production identity used for
every publication artifact, trace hash, and final floor.

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

Both required round-two rereviews completed with no findings.

## Final in-sandbox floors on `edddf491`

- `cargo test -p nuxie-runtime --lib`: 715 passed.
- `cargo test -p nuxie-runtime --test cpp_probe`: 815 passed.
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
- `make runtime-frame-loop-port-test`: 59/59, including the re-keyed W41
  ratchets and renamed-duplicate negatives.
- Publication validity is enforced by the final
  `make runtime-frame-loop-port-check` after trace regeneration.

## External floors on `edddf491`

- Apple: 87 green checks across `floor2-apple.log` and
  `floor2-xcframework.log`; XCFramework checksum
  `def916bee255bb3915d23ff898bb23dcd82d6c7351b10e118e8439670e5ccb7e`.
- Browser: every WebGPU-only invariant passes.
- Same-runner pixels: exact=1,468, byte-exact=1,370, diverges=0.
- Static-reference pixels: exact=1,468, byte-exact=837, diverges=0.
- Size: 8,201,448 bytes without scripting and 9,302,200 bytes with
  scripting, both below the 9 MiB budget.

All six `floor2-*` receipts in this evidence directory are copies with
trailing spaces and tabs removed.
