# Wave C12 scalar/blob independent review

Verdict: **ACCEPTED (6/6)**

Reviewed candidate commit `df582e7f719ac09c735eac2681943e4e7a5fa418`
against pinned upstream commit
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Counts

- 6/6 upstream identities and evidence locators are exact;
- 5/6 executable cases pass;
- 1/6 is an individually forceable expected red;
- 0 pending, proxy, synthetic-owner, or test-local-algorithm cases.

## Adversarial findings

Cases 2, 5, 6, 7, 8, and 17 preserve their pinned fixture or explicitly
documented owner adaptation, literal Luau, host/script mutation order, direct
and named reads, listener and trigger actions, asserted values, and every
ordered console line.

Case 2 uses the real `data_binding_test.riv` view-model definitions and live
`ScriptViewModel`/`ScriptedPropertyWatch` listener owner. Its direct
view-model selection is a native-scripting owner adaptation for the C++
artboard-to-default-instance setup; it does not replace the property, listener
collector, callbacks, host mutation, or trigger path. Forcing this case alone
passes the initial property/type checks, five host writes, literal script
load, both rotation reads, pre-change `calledBoth`, both listener callbacks,
and the host trigger. It then fails only at the pinned ordered-console
assertion: Rust produces `changed with context` before `changed`, while pinned
C++ requires registration order. The red is therefore a live owner divergence
and its ledger reason is accurate and sufficiently structured.

Cases 5-8 select the first authored instance from their exact upstream assets,
execute the literal scripts, preserve all host/script writes, and reproduce
all 4/4, 4/4, 4/4, and 7/7 console lines respectively.

Case 17 uses the documented Rust-safety adaptation of a real schema-backed
blob property owner. It installs exactly `{10, 20, 30}`, observes size `3` and
byte `10`, writes `abcd`, verifies the retained four bytes, and then observes
size `4` and byte `'a'`. The script and fresh-wrapper call sequence match the
pinned fixture-free case.

All six JSON rows resolve to the named test at the recorded source line. The
candidate changes no production behavior; its parent edit only includes the
test child module under the existing test/compiler cfg.

## Gates

- focused non-incremental candidate run: 5 passed, 1 ignored expected red;
- forced non-incremental case 2 run: expected failure at the exact ordered
  console assertion with the two live listener lines reversed;
- correspondence checker: passed, 157 files / 1,404 cases;
- correspondence checker unit suite: 24/24 passed;
- ledger JSON, six row identities, symbols, and source locators: passed.
