# Wave C13 final independent rereview

Status: **ACCEPTED**

This receipt independently rereviews the frozen Wave C13 candidate
`6a1f2c6463f86a02e74f2d9031da7cdf3ce86809`, rejection receipt
`fb12699f6`, and correction `64ca8d6631281884df7b006aab5b3b7c221fbcdd`
against pinned upstream `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.
It changes no candidate test, ledger row, expectation, test seam, or runtime
behavior.

## Verdict

All 25 cases are accepted: 20 direct and five narrowly adapted, with 24 live
passing cases, one genuine expected-red transition case, and zero pending.
Every fixture, literal program, action order, retained owner, assertion stream,
adaptation, and evidence locator was reread against the six pinned C++ bodies.
The 23 rows accepted by the first review remain exact and were not weakened by
the correction.

The two prior omissions are closed:

- Renderer case 1 calls the literal `render` through the existing
  balance-returning owner, asserts the real `ScriptedRenderer::end()` result is
  true immediately after the callback, and only then performs the retained
  renderer misuse and exact error assertion. Independently changing only the
  balance assertion to `!balanced` failed with `assertion failed: !balanced`.
- Update-guard case 1 reads the retained `ScriptedDrawable` occurrence's
  authoritative `in_update_phase` field immediately before and after the one
  production update call, before checking suppressed dirt. Independently
  changing each assertion, one at a time, to `Some(true)` failed at its own
  position with `left: Some(false), right: Some(true)`.

The new phase observer is read-only, resolves the actual retained component,
and is limited to `cfg(any(test, feature = "tools"))`; it neither mirrors nor
reimplements phase behavior. The renderer balance seam is likewise a
feature-gated delegation to the existing live `call_draw_with_balance` owner.
Neither introduces production behavior or a test-local behavioral proxy.

## Reaudit census

- renderer: 4/4 exact, including literal oval programs, balance results,
  1/1,000-frame streams, stack/collection checks, and both pinned SRIVs;
- require: 11/11 exact, including module registration/removal, literal source,
  ordered numeric/string results, and adapted raw-stack error presentation;
- scope: 5/5 exact, including flat names, errors, all 12 rank assertions, the
  byte-identical `scope_probe.riv`, and the leak probe;
- text runs: 1/1 exact, including named artboard, binding, initial frame, seven
  mutation frames, trigger multiplicities, advances, draws, and pinned SRIV;
- transition: 1/1 exact expected-red, with the complete bind/mutation/frame
  stream and real comparator;
- update guard: 3/3 exact under the two declared native-scripting adaptations,
  including both corrected phase reads and the authoritative retained owner.

## Verification

- focused non-incremental evidence: all 24 declared pass rows green;
- renderer correction: 1/1 green, and its balance counterexample failed at the
  corrected assertion with the observed live value `true`;
- update-guard correction: 3/3 green, and its pre/post phase counterexamples
  each failed independently with the observed live value `Some(false)`;
- transition normally remains explicitly ignored; forcing it independently
  failed at `frame 1, op 30 (color): expected color, got save`;
- strict C13 shard: 25 identities and 25 locators resolved; 20 direct / five
  adapted; 24 pass / one expected-red / zero pending;
- repository correspondence: 157 files / 1,404 pinned cases, green;
- correspondence-checker unit suite: 24/24 green;
- pinned revision and all six source identities verified; relied-on RIV/SRIV
  fixtures are present and hashable, and local `scope_probe.riv` is
  byte-identical to pinned upstream;
- JSON parsing/census, scoped diff whitespace, and locator/ignore-reason checks:
  green;
- default release `nuxie-runtime` LLVM IR contains no C13 test owner, phase
  observer, test name, or expected-red message.

Every relied-on Cargo invocation disabled incremental compilation. The three
counterexample changes were made only in a detached temporary review worktree,
which was removed after observation; the frozen candidate tree remained
unchanged.
