# Wave B2 independent semantic review

Reviewed commit: `bbf1ce429e87deafb6cfb89610d29ddf2b66039f`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Reviewer: independent `runtime_test_wave_b2_review` lane

Verdict: **REJECTED**

## Acceptance rule

All 45 rows were read from the pinned C++ case and the committed Rust
evidence. A row was accepted only when its executable Rust body recreated the
pinned fixture and setup, action order, production owner, and assertion
semantics, or reached a concrete missing/divergent boundary after performing
all executable prerequisite work. A compiling entry point, a nearby facade,
test-authored arithmetic, or a different deterministic fixture did not count
as exact correspondence.

## Exact review census

| upstream file | cases | accepted pass | accepted executable expected-red | incomplete or narrower |
|---|---:|---:|---:|---:|
| `decode_ktx2_test.cpp` | 11 | 0 | 11 | 0 |
| `default_state_machine_test.cpp` | 1 | 1 | 0 | 0 |
| `distance_constraint_test.cpp` | 1 | 1 | 0 | 0 |
| `draw_order_test.cpp` | 1 | 1 | 0 | 0 |
| `elastic_easing_test.cpp` | 2 | 1 | 0 | 1 |
| `enums_test.cpp` | 17 | 0 | 0 | 17 |
| `file_test.cpp` | 12 | 7 | 3 | 2 |
| **total** | **45** | **11** | **14** | **20** |

The accepted set is 25 direct rows: 11 passing ports and 14 executable
expected-red ports. The rejected set is three declared-direct passing rows and
all 17 declared `cxx-language-only` passing adaptations. There are no
source-anchor-only or inert-action rows, and none of the expected-red rows is
rejected.

## Blocking semantic findings

### The 17 enum adaptations do not preserve the pinned test fixture or comparison

The pinned `TestUnaryEnumOp` and `TestBinaryEnumOp` helpers seed
`std::mt19937_64` with `0xf934929` and compare each typed enum operation with
the corresponding integral operation over the five/eleven basic inputs and
1,000 values from that exact generator. The Rust evidence instead implements
a different xorshift generator. It therefore does not execute the pinned
random fixture claimed by every ledger note.

More importantly, the Rust bodies replace the typed-enum-versus-integral
comparisons with algebraic identities over raw integers. Several become
tautological substitutes; `underlying_value`, for example, asserts
`value == value` and `(value as u32) == (value as u32)`. These tests cannot
detect a discrepancy in any retained Rust flag abstraction and do not prove
the behavior tested by `rive/enums.hpp`.

The C++ syntax and template identity are legitimate language-only differences,
but that does not make a different random fixture and different assertion
owner an exact passing port. These rows need either an executable comparison
against the actual Rust flag owner with the pinned input stream, or an honest
not-applicable disposition limited to the absent C++-only surface. The current
`adapted/pass` claim overstates the evidence.

### Elastic numeric assertions use different approximation semantics

`elastic_easing_test.cpp#2` uses exact equality for the two actual-amplitude
checks and pinned Catch `Approx` semantics for the three easing checks. The
Rust evidence preserves the exact checks but applies one absolute
`<= 0.0001` margin to all three approximate values. That is not Catch's
magnitude-scaled epsilon rule: it is looser for the value below one and can be
stricter for the value near fourteen. The repository already has an
`assert_upstream_approx` helper for the pinned rule. Passing under a different
tolerance is not exact assertion correspondence.

### Two file cases omit pinned behavior

`file_test.cpp#6`, **dependencies are as expected**, explicitly asserts
`artboard->graphOrder() == 0` and then compares `nodeA` with that observed
owner value. The Rust evidence never reads or asserts the Artboard graph-order
owner; it substitutes `node_a.graph_order > Some(0)`. That cannot detect an
incorrect or absent Artboard graph order and is narrower than the pinned
assertion.

`file_test.cpp#9`, **file a bad skin (no parent skinnable) doesn't crash**,
advances once, finds every `PointsPath`, calls `markPathDirty()` on each, and
then updates again. The Rust evidence merely asserts that the parsed graph has
some paths and calls `update_pass()` a second time. It never marks any retained
path dirty, so the exact crash-regression action is not exercised.

## Accepted expected-red boundary review

All 14 red rows were forced individually. Every command selected exactly one
test and failed; none passed, skipped, or selected zero.

- The 11 KTX2 cases construct the complete pinned header/index/payload stream
  for their case and stop at the concrete absent production KTX2/BC7 decoder
  seam.
- The strip-assets case imports `jellyfish_test.riv`, validates the Jellyfish
  artboard and retained image asset, and stops only when the missing
  `File::stripAssets` owner is required.
- The signed-script case imports `joel_signed.riv`, finds real ScriptAssets,
  and stops only at the discarded `verified()` state.
- The deterministic-mode Silver row replays the complete pinned bind,
  pointer, advance, frame, and draw sequence and fails at frame 0's retained
  transform signed-zero difference.

## Mechanical and execution gates

- strict shard identity, source line, exact name, evidence locator, test
  discovery, ignore-reason, and declared-census validation: 45/45 green;
- all 31 declared passing evidence rows executed successfully, including the
  17 enum targets that are rejected semantically rather than mechanically;
- all 14 expected-red rows forced individually with exactly one selected and
  the documented concrete failure boundary;
- repository correspondence checker: 157 files and 1,404 pinned
  `TEST_CASE`s, green (the independent main case ledger remains all pending);
- correspondence checker unit suite: 24/24 green;
- `git diff --check` for the reviewed commit: green;
- reviewed commit changes tests and Wave B2 documentation only, with no
  production runtime source changes.

Mechanical success does not promote the 20 semantically incomplete rows.
Wave B2 needs correction and a fresh independent review before it can be
accepted at 45/45.
