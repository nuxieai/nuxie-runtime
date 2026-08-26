# Wave B4 final semantic correction

Corrected candidate: `9027983f4`

Final rejection receipt: `46dd4a97a`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Status: **CORRECTED; PENDING FRESH SIX-ROW REVIEW**

## Font owner corrections

Font cases 1, 2, 4, and 5 no longer reparse retained bytes or implement the
missing C++ Font behavior in cfg(test) methods. The synthetic test-only
`variation_coords` state has been removed from `RawTextFont` entirely.

Each corrected case now decodes its first exact pinned font through production
`RawTextFont`, preserves the complete downstream upstream assertion body, and
fails at the first precise public Font owner surface Rust does not expose:

- case 1: weight/italic inspection;
- case 2: `lineMetrics` before the retained metric and Catch Approx assertions;
- case 4: `getAxisCount` before axis enumeration, default lookup,
  `makeAtCoords`, and cumulative coordinate assertions;
- case 5: `features` before the exact count and seven OpenType tags.

The missing-owner functions return an explicit error at the named owner action;
they do not parse font bytes, synthesize values or state, scan another owner,
or substitute shaping output. Font case 3 and its accepted occurrence-local
fallback identity/cleanup adaptation are semantically unchanged.

## Silver action corrections

`global_variables_test` now owns the complete 197-action pinned stream: main
setter, the three default global create/set pairs, bind, initial advance/draw,
and all 62 frame/advance/draw iterations. Its forced replay reaches the real
first difference at frame 0 operation 49: expected `makeRenderPaint`, got
`color`.

The accidentally modified `artboard_opacity_and_transform_test` entry is
restored to its original zero-action `unsupported-feature` classification and
pointer-expression note.

The first block of `global_viewmodels_test-set_instance` now preserves the
exact owner order: create main, create global, mutate global, set main, set
global, bind. The accepted second block remains create/mutate main,
create/mutate global, set global, set main, bind. Its forced replay still
reaches the real frame 1 operation 163 difference.

## Exact census and gates

- 38/38 identities and locators are strict;
- classification: 37 direct / one Rust-safety adaptation;
- outcome: 29 pass / nine executable expected-red / zero pending;
- all 29 pass rows execute successfully; the additional Catch Approx oracle
  also passes;
- all nine expected-red rows were selected individually and fail at their
  documented concrete Font, Artboard, gamepad, or SRIV boundary;
- focused Silver: six pass / three ignored;
- repository correspondence: 157 files / 1,404 pinned cases;
- correspondence checker unit suite: 24/24;
- default and no-default production artifacts contain no Wave B4 test,
  synthetic variation-coordinate, or missing-owner helper symbols;
- JSON, TOML, strict locator/ignore-reason, and scoped diff checks pass.

No production runtime behavior was added or changed. This receipt does not
self-accept the six corrected rows; Wave B4 remains pending a fresh independent
semantic review.
