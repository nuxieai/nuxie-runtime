# Wave A final independent acceptance review

Reviewed commit: `e16ab6e27`

Pinned upstream: `4ac7b32798da0482e441ef09304dc3b480ed3ee5`

Reviewer: independent `runtime_test_wave_a_acceptance` lane

Verdict: **REJECTED**

## Acceptance rule

All 259 rows must be genuine executable ports and must validate from the clean
reviewed commit. A valid row must retain the pinned fixture, action order,
production owner, and assertions, or execute an approved live differential.
An expected-red row must perform the translated work before reaching one
concrete missing or divergent boundary. Its evidence locator, line, symbol,
and exact `#[ignore]` reason are part of the proof.

## Declared and certifiable census

The shard declares the required semantic census:

- status: 240 direct, four differential, 15 adapted, zero pending;
- outcome: 217 pass, 42 expected-red, zero unverified;
- body-level review: 259 genuine executable ports.

That is not the machine-certifiable census at `e16ab6e27`. Validation from a
clean detached worktree resolves 257 rows and rejects two direct/pass rows:

- status: 238 direct, four differential, 15 adapted;
- outcome: 215 pass, 42 expected-red;
- metadata-defective: two direct/pass rows.

## Blocking rows

Both blockers are stale committed line locators in
`crates/nuxie/src/lib.rs`:

- `tests/unit_tests/runtime/cdn_asset_test.cpp#1` declares
  `hosted_image_cdn_descriptor_matches_pinned_cpp` at line 10220, but the
  function starts at line 10151 in `e16ab6e27`.
- `tests/unit_tests/runtime/cdn_asset_test.cpp#2` declares
  `hosted_font_cdn_descriptor_matches_pinned_cpp` at line 10243, but the
  function starts at line 10174 in `e16ab6e27`.

The two test bodies themselves retain the pinned hosted-image and hosted-font
descriptor assertions and pass when selected in the fixture-populated primary
worktree. This rejection is only about the committed proof surface: the strict
validator correctly refuses locators that resolve only after unrelated
uncommitted line movement.

## Re-audited semantic evidence

Samples were rechecked across every one of the 25 upstream files. No body was
demoted from the 217 pass / 42 executable expected-red semantic census.

- The clip port imports `artboardclipping.riv`, advances `Center`, and reads
  the retained world-path owner before and after `frameOrigin(false)`. Its
  forced run passes the initial four points and fails only because the owner
  remains `(0,0)`, `(500,0)`, `(500,500)`, `(0,500)` instead of the pinned
  translated points.
- Component-list #16, #20, and #21 have one primary typed locator plus a
  schema-validated passing support locator. All three direct assertion bodies
  pass; #21's Silver replay passes; #16 and #20 reach only their documented
  retained-stream divergences when forced.
- The six formerly compilation-blocked `cpp_probe` rows build and run. The
  fingerprint-verified audio test for #12/#13 passes, bounds #2 passes, and
  cubic-value #1 passes. ComponentOrigin #1/#2 are discovered with their exact
  ignore reason; forced runs execute their imported-fixture controls and fail
  only at the missing immutable-object-arena insertion seam.
- All four Bezier live differentials pass, respectively executing 1,850, 15,
  74,370, and 16,539 assertions against the pinned cases and repo-owned shader
  authority.
- The 15 adaptations remain limited to 13 native-audio rows and two Rust
  safety/language call-shape rows. They do not replace a tested behavioral
  observable.
- AABB, animation-state, artboard transform, audio, binary reader, bound
  bones, bounds, color/color-glyph, command queue, component/list/origin,
  contour, data binding, dash, and blob families either pass their ordinary
  runs or discover only their exact declared expected-red tests.

No accepted row cites frozen source, an inert action list, a pending helper, or
a recording proxy for a different production owner.

## Gates

- clean-commit per-row evidence validation: 257 valid, two stale CDN locators;
- repository checker: 157 files and 1,404 pinned `TEST_CASE`s, green;
- checker unit suite: 24/24 green;
- strict shard identities: all 259 upstream paths, ordinals, lines, and names
  match the pin;
- `cpp_probe` tools-feature target and all six formerly blocked rows:
  executable as described above;
- Silver Wave A: 13 pass, 17 expected-red discovered;
- renderer Bezier Rust family: 10 pass, three expected-red discovered;
- four Bezier live differentials: green.

Wave A must not be promoted until the two CDN locators are corrected to the
lines in the committed tree and one fresh independent validator confirms
259/259 from that corrected commit. This review changes no production or test
code.
