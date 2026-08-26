# Wave B4 final six-row independent review

Reviewed correction: `5a94df5a1`

Prior rejection: `46dd4a97a`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Verdict: **ACCEPTED — Wave B4 is 38/38 semantically accepted**

This receipt is review-only. It changes no candidate test, runtime behavior,
ledger, manifest, fixture, or tool implementation.

## Six corrected rows

### Font cases 1, 2, 4, and 5

All four rows now preserve the exact production decode and setup through the
first genuinely absent Rust Font owner surface.

- Case 1 decodes the first pinned style face as a production `RawTextFont`,
  verifies the retained face index, and stops at the absent weight/italic
  inspection owner. The complete five-face expectation loop remains after the
  seam.
- Case 2 decodes the first pinned metrics face and stops at the absent
  `lineMetrics` owner. All sign/order assertions, scaled `capHeight` and
  `xHeight` calls, and the exact double-width Catch `Approx` oracle remain in
  their pinned order after that seam.
- Case 4 decodes `RobotoFlex.ttf` and stops at the absent `getAxisCount` owner.
  The axis enumeration, defaults, both `makeAtCoords` calls, and cumulative
  coordinate assertions remain intact.
- Case 5 decodes `RobotoFlex.ttf` and stops at the absent `features` owner.
  The exact count and all seven tag assertions remain intact.

The narrow missing-owner functions return typed errors at those exact actions.
They do not parse retained bytes, inspect Skrifa tables, synthesize metrics,
axes, features, or variation state, traverse another owner, or panic before
production decode. The former cfg(test)-only `variation_coords` field is gone.
The failures therefore record real absent owner surfaces rather than passing
facades that can hide missing production behavior.

### Global-viewmodel Silver case 1

`global_variables_test` now owns the complete 197-action stream:

- create and set main;
- create and set `Sizes`, `Colors`, then `Labels` in file order;
- bind, advance by `0.1`, and draw;
- execute exactly 62 `frame / advance(0.016) / draw` iterations.

The forced replay executes for the full stream and reaches the real frame 0,
operation 49 divergence: expected `makeRenderPaint`, got `color`. The unrelated
`artboard_opacity_and_transform_test` entry is restored to its original source,
provenance, zero actions, `unsupported-feature` status, and pointer-expression
note. The manifest contains neither duplicate nor missing case IDs.

### Global-viewmodel Silver case 3

Both mutation blocks now preserve the complete pinned order. The first is
create main, create global, mutate global yellow, set main, set global, bind.
The second is create/mutate main, create/mutate global cyan, set global, set
main, bind. Both retain their exact advances, frames, and draws. The forced
replay reaches the genuine frame 1, operation 163 divergence (`frame` versus
`color`).

## Final census

| upstream file | cases | accepted pass | accepted expected-red | rejected |
|---|---:|---:|---:|---:|
| `follow_path_constraint_test.cpp` | 8 | 8 | 0 | 0 |
| `font_test.cpp` | 5 | 1 | 4 | 0 |
| `gamepad_test.cpp` | 7 | 6 | 1 | 0 |
| `global_view_model_binding_test.cpp` | 15 | 13 | 2 | 0 |
| `global_viewmodels_test.cpp` | 3 | 1 | 2 | 0 |
| **total** | **38** | **29** | **9** | **0** |

The final shard classification is 37 direct and one accepted `rust-safety`
adaptation: font case 3's previously adjudicated occurrence-local fallback
identity and cleanup translation.

## Execution and mechanical gates

- pinned upstream HEAD: exact;
- strict identities, ordinals, source names/lines, classifications, evidence
  symbols/lines, ignore attributes, and reasons: 38/38 green;
- all 29 passing rows executed successfully (`17` mapped runtime-owner, `6`
  gamepad, and `6` Silver); the additional Catch oracle also passed;
- all nine expected-red rows were forced individually and failed inside their
  selected bodies at the documented Font, Artboard, gamepad, or SRIV seam;
- corrected global-variables replay ran the complete 197-action stream before
  reporting its exact comparator divergence;
- focused Silver suite: six passed and three remained ignored;
- TOML inspection proved the 197-action census and both explicit setter orders,
  restored the unrelated opacity entry, and found no duplicate IDs;
- repository correspondence checker: 157 files and 1,404 pinned `TEST_CASE`s,
  green;
- correspondence checker unit suite: 24/24 green;
- default and no-default non-tools LLVM IR contain no Wave B4,
  `variation_coords`, or missing-font-owner helper symbols;
- JSON parsing and candidate `git diff --check`: green.

The correction receipt's `Corrected candidate` header still names the prior
`9027983f4` candidate. That is a clerical receipt defect, not a test, shard, or
semantic defect; this independent receipt records the actually reviewed commit
`5a94df5a1` explicitly.
