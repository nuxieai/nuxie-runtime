# Wave C13 exactness correction candidate

Status: **CORRECTION CANDIDATE; PENDING INDEPENDENT REREVIEW**

This narrow correction addresses only the two blocking omissions recorded in
independent rejection receipt `fb12699f6`. It retains the Wave C13 pin at
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`, changes no production behavior,
and does not accept Wave C13.

## Renderer case 1

`scripting_renderer_test.cpp#1` still compiles and executes the literal pinned
program through the live Luau renderer binding. Instead of calling the wrapper
that discards `ScriptedRenderer::end()`'s return value, it now calls the
existing `upstream-test-seams` balance-returning owner seam. The test unwraps
the callback result, asserts the returned balance is true, and only then calls
`afterwards`, preserving the pinned `lua_pcall`, `end`, retained-use, and exact
error assertion order.

Forcing only the balance assertion red produced the live failure
`assertion failed: !balanced`, proving the observed owner value is true rather
than a constant or proxy expectation. The row's native-scripting adaptation
remains limited to Rust's error-display prefix.

## Update-guard case 1

`scripting_update_phase_guard_test.cpp#1` retains its literal Lua generator,
real `ScriptInstance`, pinned concrete `ScriptedDrawable` occurrence, live
`Context.markNeedsUpdate`, and production `update_script_instances` owner. A
new `cfg(any(test, feature = "tools"))` read-only observer returns the retained
occurrence's actual `RuntimeComponent.in_update_phase` field. It does not
calculate, mirror, or mutate the phase.

The test now reads that field as `Some(false)` immediately before the live Lua
update and again immediately after it, before testing suppressed dirt. Each
assertion was forced red separately; both reported the real owner value
`left: Some(false), right: Some(true)`. The structured adaptation continues to
exclude only the C++ subclass dirt-call counter and protected direct
`scriptUpdate` entry. Both authoritative phase assertions are preserved.

## Gates

- focused Wave C13 evidence: all 24 declared passing rows green;
- corrected renderer suite: 1/1 green with `upstream-test-seams`;
- corrected update-guard suite: 3/3 green;
- new renderer balance and both phase assertions forced red independently at
  their documented live values;
- scripted-transition normal suite: one explicitly ignored genuine red;
- scripted-transition forced red: exact
  `frame 1, op 30 (color): expected color, got save`;
- strict C13 shard: 25/25 resolved; 20 direct / five adapted;
  24 pass / one expected-red / zero pending;
- repository correspondence: 157 files / 1,404 cases, green;
- correspondence-checker unit suite: 24/24 green;
- pinned checkout and all six source identities: green;
- scoped rustfmt, JSON parsing, locator resolution, and diff checks: green;
- default release `nuxie-runtime` LLVM IR contains neither the tools-only phase
  observer nor any Wave C13 test/expected-red symbol.

Every relied-on Cargo invocation used `CARGO_INCREMENTAL=0` and disabled
incremental compilation for the invoked test or release profile.
