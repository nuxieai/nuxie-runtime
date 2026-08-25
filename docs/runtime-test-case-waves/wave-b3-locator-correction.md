# Wave B3 Silver locator correction

Correction target: independent rereview receipt `1c49c3036`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Status: **corrected; pending one fresh mechanical rereview**

## Scope

This is a metadata-only correction. It changes seven `line` fields in
`wave-b3.json` so that the existing evidence paths and symbols resolve to the
unchanged tests in `tools/silver-corpus/tests/wave_b3.rs`.

| focus case | symbol | previous line | corrected line |
|---:|---|---:|---:|
| 70 | `wave_b3_focus_collapsing` | 43 | 39 |
| 71 | `wave_b3_keyboard_listener` | 49 | 43 |
| 72 | `wave_b3_keyboard_listener_keyboard_input` | 55 | 47 |
| 74 | `wave_b3_focus_traversal` | 60 | 50 |
| 75 | `wave_b3_focusable_element` | 65 | 53 |
| 78 | `wave_b3_list_focus_order` | 71 | 57 |
| 79 | `wave_b3_focus_test` | 76 | 60 |

No status, outcome, expected-red reason, adaptation, fixture, action,
assertion, evidence path, evidence symbol, Rust test, or production source was
changed. The semantic 85/85 acceptance recorded by `1c49c3036` is not
self-promoted by this correction.

## Validation

- strict Wave B3 pinned identity, name, source-line, classification,
  adaptation, ignore-reason, and evidence validation: 85/85 green;
- all seven corrected Silver symbols resolve uniquely at their corrected
  lines;
- consolidated Wave A/B1/B2/B3 pinned identity and locator validation: green;
  this resolves 258 primary Rust locators, one live differential locator, and
  three supporting Rust locators in Wave A; 70 Rust locators in Wave B1; 45 in
  Wave B2; and 82 primary plus four supporting Rust locators in Wave B3;
- Wave B3 Silver target: three pass, four ignored, zero failures;
- repository correspondence checker: 157 files and 1,404 pinned
  `TEST_CASE`s, green;
- correspondence checker unit suite: 24/24 green;
- JSON parsing and scoped `git diff --check`: green.

Wave B3 requires one fresh independent mechanical rereview before the locator
rejection can be cleared.
