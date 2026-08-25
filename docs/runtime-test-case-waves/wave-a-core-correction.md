# Wave A core-case correction receipt

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Scope: only the nine rows rejected for using synthetic, nearby, stateless, or
inert evidence:

- `bounds_test.cpp` cases 1 and 3
- `child_iterator_test.cpp` case 1
- `color_glyph_test.cpp` cases 7, 9, and 11
- `component_test.cpp` cases 5, 6, and 7

No production behavior was changed. The correction ports the pinned fixtures,
actions, concrete owners, and assertions that the Rust surface can execute.
Where a required owner or observation surface does not exist, the executable
flow is marked expected-red at that exact boundary instead of substituting a
proxy or claiming a pass.

## Outcomes

| Row | Outcome | Executable boundary |
| --- | --- | --- |
| bounds 1 | expected-red | The exact text mutation, advances, root transform, and world assertions pass; `Shape::computeLocalBounds` has no callable Rust owner. |
| bounds 3 | expected-red | All 12 required objects across shapes, text, group, image, n-slice, custom path/shape, and layouts are resolved and sent through real bounds dispatch; the generic local-bounds owner is absent. |
| child iterator 1 | expected-red | `juice.riv` imports and instantiates; Rust has no `children<T>`/`objects<T>` owner, so graph filtering is no longer accepted. |
| color glyph 7 | expected-red | Both real layer extractions and both count/size assertions execute; there is no retained font cache owner to test on call two. |
| color glyph 9 | expected-red | The font decodes and reports color glyphs; Rust has no `withOptions` variation/feature derivation owner. |
| color glyph 11 | pass | The private standalone shaping owner produces one paragraph line with at least one glyph from the pinned emoji font. |
| component 5 | expected-red | The exact artboard source sequence reaches the StrokedButton replacement, which lacks the pinned owned `strokeWidth` VMI context. |
| component 6 | expected-red | Matching/different/matching source swaps and all label writes/frame counts complete; active-versus-stateful VMI pointer identity is not observable. |
| component 7 | expected-red | Add, remove, first click, re-add, second click, and clear all execute; the surviving Gamma click remains false. |

## Verification

- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime shaping_emoji_font_produces_a_paragraph_run_and_glyph --lib`: 1 passed.
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test upstream_color_glyph`: green with the cache/options and pre-existing fallback rows ignored as expected-red.
- `CARGO_INCREMENTAL=0 cargo test -p nuxie-runtime --test upstream_wave_a_core`: green with all six unavailable/divergent rows ignored as expected-red.
- Each expected-red was also run explicitly with `--ignored --exact`; every flow reached its documented concrete boundary. Component 7 completed its later re-add, second-click, and clear actions before reporting the first non-fatal upstream `CHECK` mismatch.
- `python3 -m json.tool docs/runtime-test-case-waves/wave-a.json`: valid JSON.

This receipt is scoped evidence only. It does not certify the rest of Wave A.
