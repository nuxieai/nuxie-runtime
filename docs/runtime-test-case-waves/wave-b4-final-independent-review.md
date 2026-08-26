# Wave B4 final independent semantic review

Reviewed correction: `9027983f4`

Prior rejection: `f55de7706`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **REJECTED — 32/38 semantically accepted**

This receipt is review-only. It changes no candidate test, runtime behavior,
ledger, fixture, manifest, or tool implementation.

## Exact census

| upstream file | cases | accepted pass | accepted expected-red | rejected |
|---|---:|---:|---:|---:|
| `follow_path_constraint_test.cpp` | 8 | 8 | 0 | 0 |
| `font_test.cpp` | 5 | 1 | 0 | 4 |
| `gamepad_test.cpp` | 7 | 6 | 1 | 0 |
| `global_view_model_binding_test.cpp` | 15 | 13 | 2 | 0 |
| `global_viewmodels_test.cpp` | 3 | 1 | 0 | 2 |
| **total** | **38** | **29** | **3** | **6** |

The committed shard mechanically declares 33 pass and five expected-red. Four
declared-pass font rows and two declared-red Silver rows remain semantically
rejected, leaving 31 accepted direct rows and one accepted `rust-safety`
adaptation.

## Rejected rows

### Font cases 1, 2, 4, and 5

These four rows still do not exercise production `RawTextFont` behavior. The
correction moved the prior fresh `SkrifaFontRef` table probes into cfg(test)
inherent methods on `RawTextFont`, but those methods implement the missing
observations rather than expose behavior used by the runtime.

- Case 1 reparses retained bytes in the test and derives weight/style there;
  production `RawTextFont` still has no corresponding owner behavior.
- Case 2 reparses retained bytes and computes line metrics and scaled heights
  in the test. Its revised double-width Catch `Approx` oracle is exact, but a
  correct oracle around a test-only implementation does not certify the
  runtime owner.
- Case 4 is the clearest counterexample: `variation_coords` exists only under
  `cfg(test)`, and test-only `make_at_coords` clones and mutates that synthetic
  field. Production font instances neither retain that state nor use it for
  shaping, so this row can pass while the pinned runtime behavior is absent.
- Case 5 reparses GPOS/GSUB tables in the test and constructs the feature set
  there. It remains the same source-table question rejected previously, now
  hidden behind a test-only method name.

Font case 3 is accepted. It shapes through the live runtime owner, proves the
fallback glyph uses the retained fallback font identity, destroys paragraphs,
glyphs, and the run, then clears the occurrence-local fallback chain and proves
it is empty. That is the approved Rust-safety translation of the C++ global
fallback cleanup.

### Global-viewmodel Silver case 1

`global_viewmodels_test.cpp#1` remains inert. In the actual
`global_variables_test` manifest entry, `actions` is still empty and status is
still `unsupported-feature`. Its forced failure therefore remains the same
49-operation empty replay rejected by the prior receipt; it does not execute
the pinned main setter, three global setters, bind, initial advance/draw, and
62 frame/advance/draw iterations.

The intended 197-action stream was accidentally written to the unrelated
`artboard_opacity_and_transform_test` entry instead. That entry's status was
also changed to `diverges` while its old pointer-expression note was retained.
This is both evidence that case 1 was not corrected and an unrelated corpus
regression that must be reverted.

### Global-viewmodel Silver case 3

The second setter block is exact, and the correction restores the relative
main/global setter directions. The first block still reorders the pinned
actions. C++ creates the main handle, creates and mutates the detached global
handle, then calls the main setter followed by the global setter. The manifest
calls the main setter immediately after creating the main handle, before it
creates or mutates the global handle. Exact action-stream evidence cannot move
an owner mutation across those setup actions merely because both handles are
eventually bound. This row therefore remains rejected.

## Accepted corrections

- Follow-path cases 1-3 now preserve transform capability, the root
  `Artboard::advance(0)` settlement boundary, live world-transform retrieval,
  `Mat2D::decompose`, and the decomposed x/y assertions.
- Font case 3 now preserves live shaping, fallback identity, destruction order,
  and actual occurrence-local fallback cleanup.
- Global-binding cases 4 and 5 execute the complete exact prefix and stop at a
  narrow test-only representation of the genuinely absent Artboard main setter.
  Their failures are at that owner seam, not the unrelated global-slot setter.
- Global-binding case 6 invokes the literal
  `StateMachineInstance::bind_view_model_instance` owner.
- Global-binding case 12 retains the pre-bind `DataContext` and proves the same
  allocation gains the main instance during bind.

The 24 rows accepted by the prior receipt were rechecked and remain accepted.
Their semantic bodies and prior adjudications are unchanged.

## Tool-only forwarding seam

The state-machine main setter forwarding method is doc-hidden and guarded by
the opt-in `tools` feature. Only `tools/rust-golden-runner` and
`tools/silver-corpus` enable that feature; no non-tool crate does. Both the
default-feature and no-default-feature non-tools LLVM artifacts omit the
forwarder, Wave B4 test symbols, and the cfg(test) variation field. The seam is
therefore adequately isolated for corpus replay and is not itself a rejection.

## Mechanical and execution gates

- pinned upstream HEAD: exact;
- strict ledger identities, ordinals, source names/lines, classifications,
  evidence symbols/lines, ignore attributes, and reasons: 38/38 green;
- all 33 declared passing rows executed successfully (`21` runtime-owner,
  `6` gamepad, and `6` Silver rows); the additional Catch oracle also passed;
- all five expected-red rows were forced individually and failed inside their
  selected bodies; execution confirms the declared failures but does not repair
  the two narrowed Silver streams;
- focused Silver suite: six passed and three remained ignored;
- repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green;
- correspondence checker unit suite: 24/24 green;
- default and no-default non-tools LLVM IR exclude Wave B4, `variation_coords`,
  and `set_view_model_instance_for_command_queue` symbols;
- JSON parsing, strict manifest inspection, and candidate `git diff --check`:
  green.

Wave B4 remains rejected until the four font rows observe real production font
owner behavior, global Silver case 1 carries its own complete 197-action stream
without modifying another case, and global Silver case 3 preserves the first
block's complete action order.
